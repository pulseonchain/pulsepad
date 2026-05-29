use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::PoolClosedEvent;
use crate::state::{GlobalConfig, PoolState};

// ─────────────────────────────────────────────────────────────────────────────
// close_pool() — Admin + creator co-signed.
//
// Allows a pool that has NOT yet graduated to be closed and its rent/SOL
// reclaimed. This handles the real-world case of:
//   1. Creator abandons a token (rug hygiene — clean up dead pools)
//   2. Platform wants to delist a malicious or prohibited token
//   3. Creator wants to restart with different migration params
//
// Requirements:
//   - Pool must NOT be graduated (can't close a graduated DEX pool)
//   - Requires BOTH the platform authority AND the pool's current_authority
//     to co-sign — prevents either party acting unilaterally
//   - All tokens in pool_token_account and lp_reserve_account are BURNED
//   - Any remaining SOL in fee_vault is returned to the creator
//
// After this instruction:
//   - pool_state is closed (rent returned to payer)
//   - All token accounts are emptied and closed
//   - A PoolClosedEvent is emitted for indexers
// ─────────────────────────────────────────────────────────────────────────────

pub fn close_pool(ctx: Context<ClosePool>) -> Result<()> {
    let pool = &ctx.accounts.pool_state;
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid

    // Cannot close a graduated pool — that's a DEX position now
    require!(!pool.graduated, BondingError::AlreadyGraduated);

    // Verify both parties have signed
    require!(
        ctx.accounts.creator_authority.key() == pool.current_authority,
        BondingError::Unauthorized
    );
    require!(
        ctx.accounts.platform_authority.key() == ctx.accounts.global_config.authority,
        BondingError::Unauthorized
    );

    let mint_key = ctx.accounts.mint.key();
    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE,
        mint_key.as_ref(),
        &[pool.bump],
    ]];

    // ── 1. Burn all tokens in pool_token_account ──────────────────────────────
    let pool_token_balance = ctx.accounts.pool_token_account.amount;
    if pool_token_balance > 0 {
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                pool_state_seeds,
            ),
            pool_token_balance,
        )?;
        msg!("Burned {} pool tokens", pool_token_balance);
    }

    // ── 2. Burn all tokens in lp_reserve_account ─────────────────────────────
    let lp_balance = ctx.accounts.lp_reserve_account.amount;
    if lp_balance > 0 {
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.lp_reserve_account.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                pool_state_seeds,
            ),
            lp_balance,
        )?;
        msg!("Burned {} LP reserve tokens", lp_balance);
    }

    // ── 3. Return any creator fee SOL to creator ──────────────────────────────
    let fee_recipient_balance = ctx.accounts.fee_recipient.lamports();
    let fee_vault_balance = ctx.accounts.fee_vault.lamports();

    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT,
        mint_key.as_ref(),
        &[pool.fee_vault_bump],
    ]];
    let fee_recipient_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_RECIPIENT,
        mint_key.as_ref(),
        &[pool.fee_recipient_bump],
    ]];

    // Return fee_recipient balance to creator (their earned but unclaimed fees)
    if fee_recipient_balance > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.fee_recipient.to_account_info(),
                    to: ctx.accounts.creator_authority.to_account_info(),
                },
                fee_recipient_seeds,
            ),
            fee_recipient_balance,
        )?;
    }

    // Return fee_vault balance (contains the bonding curve SOL) to creator
    if fee_vault_balance > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.creator_authority.to_account_info(),
                },
                fee_vault_seeds,
            ),
            fee_vault_balance,
        )?;
    }

    let now = Clock::get()?.unix_timestamp;
    emit!(PoolClosedEvent {
        mint: mint_key,
        creator: pool.creator,
        closed_by_platform: ctx.accounts.platform_authority.key(),
        pool_tokens_burned: pool_token_balance,
        lp_tokens_burned: lp_balance,
        sol_returned: fee_recipient_balance.saturating_add(fee_vault_balance),
        timestamp: now,
    });

    msg!(
        "🗑️  Pool closed: {} | burned={} pool + {} lp | SOL returned={}",
        mint_key, pool_token_balance, lp_balance,
        fee_recipient_balance.saturating_add(fee_vault_balance)
    );
    Ok(())
}

#[derive(Accounts)]
pub struct ClosePool<'info> {
    #[account(
        seeds = [SEED_GLOBAL_CONFIG],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// Pool to close — returned to payer
    #[account(
        mut,
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
        close = creator_authority,
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_POOL_TOKENS, mint.key().as_ref()],
        bump = pool_state.pool_tokens_bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [SEED_LP_RESERVE, mint.key().as_ref()],
        bump = pool_state.lp_reserve_bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub lp_reserve_account: Account<'info, TokenAccount>,

    /// CHECK: fee vault PDA — SOL returned to creator
    #[account(
        mut,
        seeds = [SEED_FEE_VAULT, mint.key().as_ref()],
        bump = pool_state.fee_vault_bump,
    )]
    pub fee_vault: UncheckedAccount<'info>,

    /// CHECK: creator fee recipient PDA
    #[account(
        mut,
        seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()],
        bump = pool_state.fee_recipient_bump,
    )]
    pub fee_recipient: UncheckedAccount<'info>,

    /// Must be the pool's current_authority — must sign
    #[account(mut)]
    pub creator_authority: Signer<'info>,

    /// Must be the GlobalConfig authority — must sign
    #[account(mut)]
    pub platform_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
