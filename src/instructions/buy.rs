use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::BuyEvent;
use crate::state::{GlobalConfig, PoolState, AgentWallet};
use crate::math;

/// Buy tokens from the bonding curve.
/// Fee: pool-level fee_bps (1%-5%), platform always 3/4.
/// Anti-snipe: first 3 min, effective virtual SOL = 3x → 3x more expensive.
pub fn buy(ctx: Context<Buy>, sol_amount: u64, min_tokens_out: u64) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?;
    require!(!config.paused, BondingError::Paused);
    
    let pool = &mut ctx.accounts.pool_state;
    let now = Clock::get()?.unix_timestamp;
    require!(!pool.graduated, BondingError::AlreadyGraduated);
    require!(pool.virtual_sol_reserves > 0 && pool.virtual_token_reserves > 0, BondingError::InvalidPoolState);
    require!(pool.real_token_reserves > 0 || pool.reserve_tokens_remaining > 0, BondingError::InsufficientPoolTokens);
    
    require!(sol_amount > 0 && sol_amount <= MAX_TRADE_SOL, BondingError::ZeroSolAmount);

    // ── Anti-snipe check ────────────────────────────────────────────────────
    require!(!pool.is_anti_snipe_active(now), BondingError::AntiSnipeActive);

    // ── Fee calculation (pool-level fee, platform always 3/4) ────────────────
    let fee_bps = pool.pool_fee_bps;
    let total_fee = sol_amount.checked_mul(fee_bps).ok_or(BondingError::MathOverflow)?
        .checked_div(10_000).ok_or(BondingError::MathOverflow)?;
    let platform_fee = total_fee.checked_mul(PLATFORM_FRACTION).ok_or(BondingError::MathOverflow)?
        .checked_div(100).ok_or(BondingError::MathOverflow)?;
    let creator_or_agent_fee = total_fee.saturating_sub(platform_fee);
    let net_sol = sol_amount.checked_sub(total_fee).ok_or(BondingError::MathOverflow)?;
    
    // ── Calculate tokens out (with anti-snipe virtual reserves) ─────────────
    let tokens_out = pool.calc_buy_at(net_sol, now)?;
    let impact_bps = math::buy_price_impact_bps(
        pool.effective_virtual_sol(now),
        pool.virtual_token_reserves,
        net_sol,
        tokens_out,
    );
    require!(impact_bps <= config.max_price_impact_bps, BondingError::PriceImpactTooHigh);
    require!(tokens_out >= min_tokens_out, BondingError::SlippageExceeded);

    // ── Check graduation ────────────────────────────────────────────────────
    let threshold = pool.graduation_threshold();
    let will_graduate = pool.real_sol_reserves.saturating_add(net_sol) >= threshold;

    // ── Transfer SOL: user → fee_vault ──────────────────────────────────────
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

    let mint_key = ctx.accounts.mint.key();
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT,
        mint_key.as_ref(),
        &[pool.fee_vault_bump],
    ]];

    // ── Platform fee → platform wallet ──────────────────────────────────────
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

    // ── Creator/Agent fee → fee_recipient or agent_wallet ───────────────────
    let fee_dest = if pool.fees_to_agent && pool.agent_wallet != Pubkey::default() {
        pool.agent_wallet
    } else {
        ctx.accounts.fee_recipient.key()
    };

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

    // ── Transfer tokens: pool → user ────────────────────────────────────────
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

    // ── Update reserves ────────────────────────────────────────────────────
    let (from_bonding, from_reserve) = pool.apply_buy(net_sol, tokens_out);

    emit!(BuyEvent {
        mint: mint_key,
        buyer: ctx.accounts.user.key(),
        sol_amount,
        tokens_out,
        from_bonding,
        from_reserve,
        platform_fee,
        creator_fee: creator_or_agent_fee,
        virtual_sol_reserves: pool.virtual_sol_reserves,
        virtual_token_reserves: pool.virtual_token_reserves,
        real_sol_reserves: pool.real_sol_reserves,
        timestamp: now,
    });

    if will_graduate {
        msg!("🎓 Graduation threshold reached! real_sol={} threshold={}", pool.real_sol_reserves, threshold);
    }

    msg!(
        "Buy: {} lamports → {} tokens | fee={} platform={} | tier={:?}",
        sol_amount, tokens_out, total_fee, platform_fee, pool.graduation_tier
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

    /// CHECK: PDA — accumulates creator's 0.25% share (or agent wallet if fees_to_agent)
    #[account(mut, seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()], bump = pool_state.fee_recipient_bump)]
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: Agent wallet PDA (optional — validated via pool_state.agent_wallet)
    #[account(mut, seeds = [SEED_AGENT, mint.key().as_ref()], bump)]
    pub agent_wallet: Option<Box<Account<'info, AgentWallet>>>,

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