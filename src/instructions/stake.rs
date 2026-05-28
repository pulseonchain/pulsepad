use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::StakeEvent;
use crate::state::{PoolState, StakeAccount, StakerVault};

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, BondingError::ZeroStakeAmount);

    let mint_key = ctx.accounts.mint.key();
    let now = Clock::get()?.unix_timestamp;

    // ── Snapshot checkpoint before changing stake ─────────────────────────────
    let new_reward_debt;
    {
        let vault = &ctx.accounts.staker_vault;
        let _acc   = &ctx.accounts.stake_account;
        new_reward_debt = vault.accumulated_reward_per_token;
    }

    // ── Transfer tokens: user_token_account → stake_token_vault ──────────────
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.stake_token_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    // ── Update StakeAccount ───────────────────────────────────────────────────
    let acc = &mut ctx.accounts.stake_account;
    if acc.amount_staked == 0 {
        acc.owner       = ctx.accounts.user.key();
        acc.mint        = mint_key;
        acc.staked_at   = now;
        acc.last_claimed = now;
        acc.bump        = ctx.bumps.stake_account;
    }
    acc.amount_staked = acc.amount_staked.saturating_add(amount);
    acc.reward_debt   = new_reward_debt; // reset checkpoint to current

    // ── Update StakerVault total ──────────────────────────────────────────────
    let vault = &mut ctx.accounts.staker_vault;
    vault.total_staked = vault.total_staked.saturating_add(amount);

    emit!(StakeEvent {
        mint: mint_key,
        staker: ctx.accounts.user.key(),
        amount,
        timestamp: now,
    });

    msg!("Staked {} tokens for {}", amount, ctx.accounts.user.key());
    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = user,
        space = StakeAccount::ACCOUNT_SIZE,
        seeds = [SEED_STAKE, mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    #[account(
        mut,
        seeds = [StakerVault::SEED, mint.key().as_ref()],
        bump = staker_vault.bump,
    )]
    pub staker_vault: Box<Account<'info, StakerVault>>,

    // Program-owned token account that physically holds the staked tokens
    #[account(
        mut,
        seeds = [b"stake_token_vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub stake_token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
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
