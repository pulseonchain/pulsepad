use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::SellEvent;
use crate::state::{GlobalConfig, PoolState, AgentWallet};
use crate::math;

pub fn sell(ctx: Context<Sell>, token_amount: u64, min_sol_out: u64) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?;
    require!(!config.paused, BondingError::Paused);
    
    let pool = &mut ctx.accounts.pool_state;
    require!(!pool.graduated, BondingError::AlreadyGraduated);
    require!(pool.virtual_sol_reserves > 0 && pool.virtual_token_reserves > 0, BondingError::InvalidPoolState);
    require!(pool.real_sol_reserves > 0, BondingError::InsufficientPoolSol);
    require!(token_amount > 0 && token_amount <= MAX_TRADE_TOKENS, BondingError::ZeroTokenAmount);

    // ── Calculate gross SOL out ──────────────────────────────────────────────
    let sol_out_gross = pool.calc_sell(token_amount)?;
    let impact_bps = math::sell_price_impact_bps(
        pool.virtual_sol_reserves, pool.virtual_token_reserves, token_amount, sol_out_gross,
    );
    require!(impact_bps <= config.max_price_impact_bps, BondingError::PriceImpactTooHigh);

    // ── Pool-level fees ──────────────────────────────────────────────────────
    let fee_bps = pool.pool_fee_bps;
    let total_fee = sol_out_gross.checked_mul(fee_bps).ok_or(BondingError::MathOverflow)?
        .checked_div(10_000).ok_or(BondingError::MathOverflow)?;
    let platform_fee = total_fee.checked_mul(PLATFORM_FRACTION).ok_or(BondingError::MathOverflow)?
        .checked_div(100).ok_or(BondingError::MathOverflow)?;
    let creator_or_agent_fee = total_fee.saturating_sub(platform_fee);
    let net_sol_to_user = sol_out_gross.checked_sub(total_fee).ok_or(BondingError::MathOverflow)?;
    require!(net_sol_to_user >= min_sol_out, BondingError::SlippageExceeded);

    let mint_key = ctx.accounts.mint.key();
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT, mint_key.as_ref(), &[pool.fee_vault_bump],
    ]];

    // ── Tokens: user → pool ──────────────────────────────────────────────────
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        token_amount,
    )?;

    // ── SOL: fee_vault → user ────────────────────────────────────────────────
    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.fee_vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            fee_vault_seeds,
        ),
        net_sol_to_user,
    )?;

    // ── Platform fee ─────────────────────────────────────────────────────────
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

    // ── Creator/Agent fee → fee_recipient ────────────────────────────────────
    if creator_or_agent_fee > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.fee_recipient.to_account_info(),
                },
                fee_vault_seeds,
            ),
            creator_or_agent_fee,
        )?;
    }

    // ── Update reserves ─────────────────────────────────────────────────────
    pool.apply_sell(token_amount, sol_out_gross);

    let now = Clock::get()?.unix_timestamp;
    emit!(SellEvent {
        mint: mint_key,
        seller: ctx.accounts.user.key(),
        token_amount,
        sol_out: net_sol_to_user,
        platform_fee,
        creator_fee: creator_or_agent_fee,
        virtual_sol_reserves: pool.virtual_sol_reserves,
        virtual_token_reserves: pool.virtual_token_reserves,
        real_sol_reserves: pool.real_sol_reserves,
        timestamp: now,
    });

    msg!("Sell: {} tokens → {} lamports | fee_bps={}", token_amount, net_sol_to_user, fee_bps);
    Ok(())
}

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Box<Account<'info, GlobalConfig>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(mut, seeds = [SEED_POOL_TOKENS, mint.key().as_ref()], bump = pool_state.pool_tokens_bump, token::mint = mint, token::authority = pool_state)]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = global_config.platform_wallet)]
    pub platform_wallet: UncheckedAccount<'info>,

    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump = pool_state.fee_vault_bump)]
    pub fee_vault: UncheckedAccount<'info>,

    #[account(mut, seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()], bump = pool_state.fee_recipient_bump)]
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(mut, seeds = [SEED_AGENT, mint.key().as_ref()], bump)]
    pub agent_wallet: Option<Box<Account<'info, AgentWallet>>>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = user)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}