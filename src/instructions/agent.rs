use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::*;
use crate::state::*;

// ─── AGENT CLAIM ──────────────────────────────────────────────────────────

pub fn agent_claim(ctx: Context<AgentClaim>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let agent = &mut ctx.accounts.agent_wallet;
    let amount = agent.claim(now)?;
    require!(amount > 0, BondingError::NoRewardsToClaim);

    let bump = ctx.bumps.agent_wallet;
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[&[u8]]] = &[&[SEED_AGENT, mint_key.as_ref(), &[bump]]];

    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.agent_wallet.to_account_info(),
                to: ctx.accounts.agent_authority.to_account_info(),
            },
            seeds,
        ),
        amount,
    )?;

    emit!(AgentClaimedEvent {
        mint: mint_key,
        agent_wallet: ctx.accounts.agent_wallet.key(),
        agent_name: agent.agent_name.clone(),
        amount,
        timestamp: now,
    });

    msg!("Agent claimed {} lamports", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct AgentClaim<'info> {
    pub mint: Box<Account<'info, anchor_spl::token::Mint>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(mut, seeds = [SEED_AGENT, mint.key().as_ref()], bump)]
    pub agent_wallet: Box<Account<'info, AgentWallet>>,

    /// CHECK: PDA
    #[account(mut)]
    pub agent_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

// ─── AGENT BUYBACK ────────────────────────────────────────────────────────

pub fn agent_buyback(ctx: Context<AgentBuyback>, sol_to_spend: u64, burn_pct: u8) -> Result<()> {
    let pool = &mut ctx.accounts.pool_state;
    require!(pool.buyback_active, BondingError::BuybackFundEmpty);
    require!(sol_to_spend <= pool.buyback_sol_reserves, BondingError::BuybackFundEmpty);
    require!(burn_pct <= 100, BondingError::InvalidConfig);

    let now = Clock::get()?.unix_timestamp;
    require!(
        now.saturating_sub(pool.last_buyback_at) >= 3600,
        BondingError::BuybackAlreadyExecuted
    );

    let vt = pool.buyback_virtual_token_reserves as u128;
    let vs = pool.buyback_virtual_sol_reserves as u128;
    let s = sol_to_spend as u128;
    let tokens_out = vt.checked_mul(s).unwrap_or(0)
        .checked_div(vs.checked_add(s).unwrap_or(1)).unwrap_or(0) as u64;
    require!(tokens_out > 0 && tokens_out <= pool.buyback_token_reserves, BondingError::BuybackFundEmpty);

    pool.buyback_virtual_sol_reserves = pool.buyback_virtual_sol_reserves.saturating_add(sol_to_spend);
    pool.buyback_virtual_token_reserves = pool.buyback_virtual_token_reserves.saturating_sub(tokens_out);
    pool.buyback_sol_reserves = pool.buyback_sol_reserves.saturating_sub(sol_to_spend);
    pool.buyback_token_reserves = pool.buyback_token_reserves.saturating_sub(tokens_out);
    pool.last_buyback_at = now;

    let burn_amount = (tokens_out as u128).checked_mul(burn_pct as u128).unwrap_or(0)
        .checked_div(100).unwrap_or(0) as u64;
    let _treasury_amount = tokens_out.saturating_sub(burn_amount);

    if burn_amount > 0 {
        let pool_state_seeds: &[&[&[u8]]] = &[&[
            SEED_POOL_STATE, ctx.accounts.mint.key().as_ref(), &[pool.bump],
        ]];
        anchor_spl::token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                pool_state_seeds,
            ),
            burn_amount,
        )?;
    }

    emit!(BuybackExecutedEvent {
        mint: ctx.accounts.mint.key(),
        sol_spent: sol_to_spend,
        tokens_burned: burn_amount,
        tokens_to_treasury: _treasury_amount,
        timestamp: now,
    });

    msg!("Buyback: {} SOL spent -> {} tokens ({} burned)", sol_to_spend, tokens_out, burn_amount);
    Ok(())
}

#[derive(Accounts)]
pub struct AgentBuyback<'info> {
    #[account(mut)]
    pub mint: Box<Account<'info, anchor_spl::token::Mint>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(mut, seeds = [SEED_POOL_TOKENS, mint.key().as_ref()], bump = pool_state.pool_tokens_bump)]
    pub pool_token_account: Box<Account<'info, anchor_spl::token::TokenAccount>>,

    /// CHECK: PDA
    #[account(seeds = [SEED_AGENT, mint.key().as_ref()], bump)]
    pub agent_wallet: UncheckedAccount<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub system_program: Program<'info, System>,
}

// ─── AGENT TRANSFER ───────────────────────────────────────────────────────

pub fn agent_transfer(ctx: Context<AgentTransfer>, amount: u64) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let bump = ctx.bumps.agent_wallet;
    let seeds: &[&[&[u8]]] = &[&[SEED_AGENT, mint_key.as_ref(), &[bump]]];
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.agent_wallet.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
            },
            seeds,
        ),
        amount,
    )?;
    msg!("Agent transferred {} to {}", amount, ctx.accounts.destination.key());
    Ok(())
}

#[derive(Accounts)]
pub struct AgentTransfer<'info> {
    pub mint: Box<Account<'info, anchor_spl::token::Mint>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(mut, seeds = [SEED_AGENT, mint.key().as_ref()], bump)]
    pub agent_wallet: UncheckedAccount<'info>,

    /// CHECK: destination
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
