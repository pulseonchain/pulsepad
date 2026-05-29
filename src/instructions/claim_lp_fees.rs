use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::LpFeeClaimedEvent;
use crate::state::{GlobalConfig, MigrationTarget, PoolState, StakerVault};

// ─────────────────────────────────────────────────────────────────────────────
// claim_lp_fees — permissionless crank.
// Valid only for: RaydiumCpmm, PumpSwapHoldLp
// Calls the DEX program to claim accumulated LP fees into fee_vault,
// then splits them 0.75% platform / 0.25% fee_recipient.
// ─────────────────────────────────────────────────────────────────────────────
pub fn claim_lp_fees(ctx: Context<ClaimLpFees>) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid
    let pool = &ctx.accounts.pool_state;

    require!(pool.graduated, BondingError::NotReadyToGraduate);

    match &pool.migration_target {
        MigrationTarget::RaydiumCpmm | MigrationTarget::PumpSwapHoldLp => {}
        _ => return err!(BondingError::InvalidMigrationConfig),
    }

    let mint_key = ctx.accounts.mint.key();
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT,
        mint_key.as_ref(),
        &[pool.fee_vault_bump],
    ]];

    // ── Record fee_vault balance before claim ─────────────────────────────────
    let balance_before = ctx.accounts.fee_vault.lamports();

    // [INTEGRATION NOTE] ──────────────────────────────────────────────────────
    // Here we CPI into the DEX program to claim LP fees.
    //
    // Raydium CPMM: call collect_protocol_fee / withdraw
    //   The fee_vault PDA is the LP holder — it signs via PDA seeds.
    //   Raydium sends accumulated fees as SOL/wrapped SOL to fee_vault.
    //
    // PumpSwap: call withdraw (fee_vault holds LP tokens)
    //   fee_vault signs as LP holder, PumpSwap sends fees back.
    //
    // After the CPI, fee_vault's lamport balance increases by claimed_amount.
    // We read the delta and split it below.
    // ─────────────────────────────────────────────────────────────────────────

    // Read delta after DEX CPI (in production this follows the actual CPI)
    let balance_after = ctx.accounts.fee_vault.lamports();
    let claimed = balance_after.saturating_sub(balance_before);

    if claimed == 0 {
        msg!("No LP fees available to claim right now");
        return Ok(());
    }

    // ── Split claimed SOL: 0.75% → platform, 0.25% → fee_recipient ───────────
    let (_, platform_fee, creator_fee) = config.calc_fees(claimed);

    if platform_fee > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.platform_wallet.to_account_info(),
                },
                fee_vault_seeds,
            ),
            platform_fee,
        )?;
    }

    if creator_fee > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.fee_recipient.to_account_info(),
                },
                fee_vault_seeds,
            ),
            creator_fee,
        )?;
    }

    // ── Credit staker vault for Meteora targets with staker_share ──────────────
    let staker_share_bps = match &pool.migration_target {
        MigrationTarget::MeteoraDammV1 { staker_share, .. } => *staker_share as u64,
        MigrationTarget::MeteoraDlmm { staker_share, .. } => *staker_share as u64,
        _ => 0,
    };
    if staker_share_bps > 0 {
        if let Some(ref mut sv) = ctx.accounts.staker_vault {
            let staker_amount = claimed
                .checked_mul(staker_share_bps)
                .unwrap_or(0)
                .checked_div(100)
                .unwrap_or(0);
            if staker_amount > 0 && sv.total_staked > 0 {
                system_program::transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.system_program.to_account_info(),
                        system_program::Transfer {
                            from: ctx.accounts.fee_vault.to_account_info(),
                            to: sv.to_account_info(),
                        },
                        fee_vault_seeds,
                    ),
                    staker_amount,
                )?;
                sv.add_rewards(staker_amount);
            }
        }
    }

    let now = Clock::get()?.unix_timestamp;
    emit!(LpFeeClaimedEvent {
        mint: mint_key,
        platform_amount: platform_fee,
        creator_amount: creator_fee,
        timestamp: now,
    });

    msg!(
        "LP fees claimed: {} total | {} platform | {} creator",
        claimed, platform_fee, creator_fee
    );
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimLpFees<'info> {
    #[account(
        seeds = [SEED_GLOBAL_CONFIG],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    pub mint: Account<'info, Mint>,

    /// CHECK: fee_vault PDA — LP holder, signs DEX claim CPI
    #[account(
        mut,
        seeds = [SEED_FEE_VAULT, mint.key().as_ref()],
        bump = pool_state.fee_vault_bump,
    )]
    pub fee_vault: SystemAccount<'info>,

    /// CHECK: fee_recipient PDA — receives creator's 0.25%
    #[account(
        mut,
        seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()],
        bump = pool_state.fee_recipient_bump,
    )]
    pub fee_recipient: SystemAccount<'info>,

    /// Staker vault — receives staker's share of LP fees for Meteora targets
    #[account(
        mut,
        seeds = [StakerVault::SEED, mint.key().as_ref()],
        bump = staker_vault.bump,
    )]
    pub staker_vault: Option<Account<'info, StakerVault>>,

    /// CHECK: validated against global_config.platform_wallet
    #[account(
        mut,
        address = global_config.platform_wallet,
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// CHECK: DEX pool account
    #[account(mut)]
    pub dex_pool: UncheckedAccount<'info>,

    /// CHECK: LP token mint (Raydium or PumpSwap LP mint)
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    /// CHECK: fee_vault's LP token account
    #[account(mut)]
    pub fee_vault_lp_account: Account<'info, TokenAccount>,

    /// CHECK: DEX program (Raydium or PumpSwap)
    pub dex_program: UncheckedAccount<'info>,

    /// permissionless — anyone can trigger (backend crank)
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
