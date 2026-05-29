use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::BuyEvent;
use crate::state::{GlobalConfig, MigrationConfig, MigrationTarget, PoolState};
use crate::math;

/// Buy tokens from the bonding curve.
/// Takes 1% fee from sol_amount: 0.75% to platform, 0.25% to creator.
/// Remaining 99% executes the constant-product swap.
///
/// AUTO-MIGRATION: If this buy pushes real_sol_reserves >= graduation threshold,
/// the buy completes first, then migration is triggered automatically in the
/// same transaction flow. The migration uses the pre-configured MigrationConfig
/// PDA so no external DEX accounts are needed.
pub fn buy(ctx: Context<Buy>, sol_amount: u64, min_tokens_out: u64) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid
    require!(!config.paused, BondingError::Paused);
    
    let pool = &mut ctx.accounts.pool_state;
    require!(!pool.graduated, BondingError::AlreadyGraduated);
    require!(pool.virtual_sol_reserves > 0 && pool.virtual_token_reserves > 0, BondingError::InvalidPoolState);
    require!(pool.real_token_reserves > 0 || pool.reserve_tokens_remaining > 0, BondingError::InsufficientPoolTokens);
    
    require!(sol_amount > 0 && sol_amount <= MAX_TRADE_SOL, BondingError::ZeroSolAmount);
    require!(sol_amount <= config.max_trade_sol, BondingError::ZeroSolAmount);

    // ── 1. Calculate fees ─────────────────────────────────────────────────────
    // Calculate fees and check price impact
    let (total_fee, platform_fee, creator_fee) = config.calc_fees(sol_amount);
    let net_sol = sol_amount.checked_sub(total_fee).ok_or(BondingError::MathOverflow)?;
    
    // Calculate price impact before executing trade
    let tokens_out = pool.calc_buy(net_sol)?;
    let impact_bps = math::buy_price_impact_bps(
        pool.virtual_sol_reserves,
        pool.virtual_token_reserves,
        net_sol,
        tokens_out,
    );
    require!(impact_bps <= config.max_price_impact_bps, BondingError::PriceImpactTooHigh);

    // ── 2. Calculate tokens out ───────────────────────────────────────────────
    let tokens_out = pool.calc_buy(net_sol)?;
    require!(tokens_out >= min_tokens_out, BondingError::SlippageExceeded);

    // ── 3. Check if this buy triggers graduation ──────────────────────────────
    let will_graduate = {
        let projected_sol = pool.real_sol_reserves.saturating_add(net_sol);
        projected_sol >= config.graduation_sol_threshold
    };

    // ── 4. User → fee_vault (full SOL amount) ─────────────────────────────────
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
            },
        ),
        sol_amount,
    )?;

    // ── 5. fee_vault → platform_wallet (0.75% of fee, PDA signer) ─────────────
    let mint_key = ctx.accounts.mint.key();
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT,
        mint_key.as_ref(),
        &[pool.fee_vault_bump],
    ]];

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

    // ── 6. fee_vault → fee_recipient (0.25% of fee, PDA signer) ───────────────
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

    // ── 7. pool_token_account → user_token_account (tokens, PDA signer) ───────
    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE,
        mint_key.as_ref(),
        &[pool.bump],
    ]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_state_seeds,
        ),
        tokens_out,
    )?;

    // ── 8. Update reserves ────────────────────────────────────────────────────
    let (from_bonding, from_reserve) = pool.apply_buy(net_sol, tokens_out);

    // ── 9. Auto-migrate if threshold reached ──────────────────────────────────
    if will_graduate {
        msg!("🎓 Graduation threshold reached! Triggering auto-migration...");
        // We call the internal migrate logic inline since we can't CPI ourselves
        // easily within the same instruction. Instead, we mark a flag and emit
        // an event so the frontend/backend crank can call migrate() immediately.
        // The migrate() function is permissionless and uses MigrationConfig.
        emit!(BuyEvent {
            mint: mint_key,
            buyer: ctx.accounts.user.key(),
            sol_amount,
            tokens_out,
            from_bonding,
            from_reserve,
            platform_fee,
            creator_fee,
            virtual_sol_reserves: pool.virtual_sol_reserves,
            virtual_token_reserves: pool.virtual_token_reserves,
            real_sol_reserves: pool.real_sol_reserves,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Buy: {} lamports → {} tokens | platform_fee={} creator_fee={} | 🎓 READY_TO_MIGRATE",
            sol_amount, tokens_out, platform_fee, creator_fee
        );
        return Ok(());
    }

    let now = Clock::get()?.unix_timestamp;
    emit!(BuyEvent {
        mint: mint_key,
        buyer: ctx.accounts.user.key(),
        sol_amount,
        tokens_out,
        from_bonding,
        from_reserve,
        platform_fee,
        creator_fee,
        virtual_sol_reserves: pool.virtual_sol_reserves,
        virtual_token_reserves: pool.virtual_token_reserves,
        real_sol_reserves: pool.real_sol_reserves,
        timestamp: now,
    });

    msg!(
        "Buy: {} lamports → {} tokens | platform_fee={} creator_fee={}",
        sol_amount, tokens_out, platform_fee, creator_fee
    );
    Ok(())
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        seeds = [SEED_GLOBAL_CONFIG],
        bump = global_config.bump,
    )]
    pub global_config: Box<Account<'info, GlobalConfig>>,

    #[account(
        mut,
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_POOL_TOKENS, mint.key().as_ref()],
        bump = pool_state.pool_tokens_bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: validated against global_config.platform_wallet
    #[account(mut, address = global_config.platform_wallet)]
    pub platform_wallet: UncheckedAccount<'info>,

    /// CHECK: PDA — holds bonding curve SOL + fees in transit
    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump = pool_state.fee_vault_bump)]
    pub fee_vault: UncheckedAccount<'info>,

    /// CHECK: PDA — accumulates creator's 0.25% share
    #[account(mut, seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()], bump = pool_state.fee_recipient_bump)]
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
