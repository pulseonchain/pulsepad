use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::UnstakeEvent;
use crate::state::{PoolState, StakeAccount, StakerVault};

pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    require!(amount > 0, BondingError::ZeroStakeAmount);
    require!(
        ctx.accounts.stake_account.amount_staked >= amount,
        BondingError::InsufficientStake
    );

    let mint_key = ctx.accounts.mint.key();
    let now = Clock::get()?.unix_timestamp;

    // ── Snapshot new reward_debt before reducing stake ────────────────────────
    let new_reward_debt = ctx.accounts.staker_vault.accumulated_reward_per_token;

    // ── Transfer tokens: stake_token_vault → user ─────────────────────────────
    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE,
        mint_key.as_ref(),
        &[ctx.accounts.pool_state.bump],
    ]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.stake_token_vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.pool_state.to_account_info(),
            },
            pool_state_seeds,
        ),
        amount,
    )?;

    // ── Update accounts ───────────────────────────────────────────────────────
    let acc = &mut ctx.accounts.stake_account;
    acc.amount_staked = acc.amount_staked.saturating_sub(amount);
    acc.reward_debt   = new_reward_debt; // reset checkpoint

    let vault = &mut ctx.accounts.staker_vault;
    vault.total_staked = vault.total_staked.saturating_sub(amount);

    emit!(UnstakeEvent {
        mint: mint_key,
        staker: ctx.accounts.user.key(),
        amount,
        timestamp: now,
    });

    msg!("Unstaked {} tokens for {}", amount, ctx.accounts.user.key());
    Ok(())
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_STAKE, mint.key().as_ref(), user.key().as_ref()],
        bump = stake_account.bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [StakerVault::SEED, mint.key().as_ref()],
        bump = staker_vault.bump,
    )]
    pub staker_vault: Account<'info, StakerVault>,

    #[account(
        mut,
        seeds = [b"stake_token_vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub stake_token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
