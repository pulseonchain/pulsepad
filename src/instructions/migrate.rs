use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, Token, TokenAccount, Transfer},
};
use anchor_lang::solana_program::program_pack::Pack;
use spl_token::state::Account as SplTokenAccount;
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::{MigrateEvent, BuybackActivatedEvent};
use crate::state::{GlobalConfig, MigrationConfig, PoolState};

pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?;
    let pool = &mut ctx.accounts.pool_state;
    let migration_config = &ctx.accounts.migration_config;

    require!(!pool.graduated, BondingError::AlreadyGraduated);
    let threshold = pool.graduation_threshold();
    require!(pool.is_ready_to_graduate(threshold), BondingError::NotReadyToGraduate);
    require!(migration_config.mint == pool.mint, BondingError::InvalidMigrationConfig);

    let mint_key = ctx.accounts.mint.key();
    let sol_for_dex = if pool.has_partial_migration() {
        let pct = pool.partial_migration_pct as u128;
        let keep = (pool.real_sol_reserves as u128)
            .saturating_mul(pct)
            .saturating_div(100) as u64;
        pool.real_sol_reserves.saturating_sub(keep)
    } else {
        pool.real_sol_reserves
    };

    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE, mint_key.as_ref(), &[pool.bump],
    ]];
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT, mint_key.as_ref(), &[pool.fee_vault_bump],
    ]];

    // ── Create migration vault ATA if needed ──────────────────────────────────
    {
        let vata_info = ctx.accounts.migration_vault_token_account.to_account_info();
        if vata_info.data_is_empty() {
            let rent = Rent::get()?;
            let lamports = rent.minimum_balance(SplTokenAccount::LEN);
            let create_ix = anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.payer.key(), &vata_info.key(), lamports,
                SplTokenAccount::LEN as u64, &spl_token::id(),
            );
            anchor_lang::solana_program::program::invoke(
                &create_ix,
                &[ctx.accounts.payer.to_account_info(), vata_info.clone(), ctx.accounts.system_program.to_account_info()],
            )?;
            let init_ix = spl_token::instruction::initialize_account3(
                &spl_token::id(), &vata_info.key(), &ctx.accounts.mint.key(),
                &ctx.accounts.migration_vault.key(),
            )?;
            anchor_lang::solana_program::program::invoke(
                &init_ix,
                &[vata_info.clone(), ctx.accounts.mint.to_account_info(), ctx.accounts.token_program.to_account_info(), ctx.accounts.migration_vault.to_account_info()],
            )?;
            msg!("Created migration vault ATA");
        }
    }

    // ── LP tokens → DEX ───────────────────────────────────────────────────────
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.lp_reserve_account.to_account_info(),
                to: ctx.accounts.dex_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_state_seeds,
        ),
        LP_RESERVE_SUPPLY,
    )?;

    // ── Remaining pool tokens: half burn, half vault ──────────────────────────
    let remaining = ctx.accounts.pool_token_account.amount;
    let (burn_amount, vault_amount) = if remaining > 0 {
        let half = remaining / 2;
        if half > 0 {
            token::burn(CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn { mint: ctx.accounts.mint.to_account_info(), from: ctx.accounts.pool_token_account.to_account_info(), authority: pool.to_account_info() },
                pool_state_seeds,
            ), half)?;
        }
        let to_vault = remaining - half;
        if to_vault > 0 {
            token::transfer(CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer { from: ctx.accounts.pool_token_account.to_account_info(), to: ctx.accounts.migration_vault_token_account.to_account_info(), authority: pool.to_account_info() },
                pool_state_seeds,
            ), to_vault)?;
        }
        (half, to_vault)
    } else { (0, 0) };

    // ── SOL → DEX ─────────────────────────────────────────────────────────────
    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer { from: ctx.accounts.fee_vault.to_account_info(), to: ctx.accounts.dex_pool.to_account_info() },
            fee_vault_seeds,
        ),
        sol_for_dex,
    )?;

    // ── Partial migration: activate buyback ────────────────────────────────────
    if pool.has_partial_migration() {
        let pct = pool.partial_migration_pct as u128;
        let kept_sol = (pool.real_sol_reserves as u128).saturating_mul(pct).saturating_div(100) as u64;
        let kept_tokens = if remaining > 0 {
            let vault_half = remaining / 2;
            (vault_half as u128).saturating_mul(pct).saturating_div(100) as u64
        } else { 0 };
        pool.activate_buyback(kept_sol, kept_tokens);

        let now = Clock::get()?.unix_timestamp;
        emit!(BuybackActivatedEvent {
            mint: mint_key,
            kept_sol,
            kept_tokens,
            partial_migration_pct: pool.partial_migration_pct,
            timestamp: now,
        });
        msg!("Buyback activated: {} SOL + {} tokens kept", kept_sol, kept_tokens);
    }

    // ── Mark graduated ────────────────────────────────────────────────────────
    pool.graduated = true;
    pool.dex_pool = Some(ctx.accounts.dex_pool.key());
    pool.real_sol_reserves = 0;
    pool.real_token_reserves = 0;
    pool.reserve_tokens_remaining = 0;

    let now = Clock::get()?.unix_timestamp;
    emit!(MigrateEvent {
        mint: mint_key,
        migration_target: pool.migration_target.clone(),
        sol_deposited: sol_for_dex,
        tokens_deposited: LP_RESERVE_SUPPLY,
        tokens_burned: burn_amount,
        tokens_to_migration_vault: vault_amount,
        dex_pool: ctx.accounts.dex_pool.key(),
        timestamp: now,
    });

    msg!("🎓 MIGRATED {} | SOL={} | LP={} | Burn={} | Vault={} | Tier={:?}", mint_key, sol_for_dex, LP_RESERVE_SUPPLY, burn_amount, vault_amount, pool.graduation_tier);
    Ok(())
}

#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Box<Account<'info, GlobalConfig>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(seeds = [SEED_MIGRATION_CONFIG, mint.key().as_ref()], bump = migration_config.bump)]
    pub migration_config: Box<Account<'info, MigrationConfig>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(mut, seeds = [SEED_POOL_TOKENS, mint.key().as_ref()], bump = pool_state.pool_tokens_bump, token::mint = mint, token::authority = pool_state)]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [SEED_LP_RESERVE, mint.key().as_ref()], bump = pool_state.lp_reserve_bump, token::mint = mint, token::authority = pool_state)]
    pub lp_reserve_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub migration_vault_token_account: UncheckedAccount<'info>,

    #[account(seeds = [SEED_MIGRATION_VAULT, mint.key().as_ref()], bump = pool_state.migration_vault_bump)]
    pub migration_vault: UncheckedAccount<'info>,

    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump = pool_state.fee_vault_bump)]
    pub fee_vault: UncheckedAccount<'info>,

    #[account(mut, address = migration_config.dex_pool)]
    pub dex_pool: UncheckedAccount<'info>,

    #[account(mut, address = migration_config.dex_token_account)]
    pub dex_token_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}