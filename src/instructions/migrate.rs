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
use crate::events::MigrateEvent;
use crate::state::{GlobalConfig, MigrationConfig, MigrationTarget, PoolState};

// ─────────────────────────────────────────────────────────────────────────────
// migrate() — permissionless. Anyone can call once the 85 SOL threshold is met.
//
// Uses the pre-configured MigrationConfig PDA so no DEX accounts need to be
// passed by the caller. This makes migration fully permissionless and safe.
//
// Token flow at migration:
//   0. Create migration vault ATA if it doesn't exist
//   1. LP reserve (300M tokens) → DEX token account (from MigrationConfig)
//   2. Remaining pool tokens (bonding leftovers + reserve leftovers):
//      a. Half → BURN (removed from supply forever)
//      b. Half → migration_vault ATA (program-controlled, claimable by creator)
//   3. All SOL in fee_vault → DEX pool (from MigrationConfig)
//   4. Mark graduated
// ─────────────────────────────────────────────────────────────────────────────
pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid
    let pool   = &mut ctx.accounts.pool_state;
    let migration_config = &ctx.accounts.migration_config;

    require!(!pool.graduated, BondingError::AlreadyGraduated);
    require!(
        pool.is_ready_to_graduate(config.graduation_sol_threshold),
        BondingError::NotReadyToGraduate
    );

    // Verify migration_config matches this pool
    require!(
        migration_config.mint == pool.mint,
        BondingError::InvalidMigrationConfig
    );

    let mint_key       = ctx.accounts.mint.key();
    let sol_to_deposit = pool.real_sol_reserves;

    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE,
        mint_key.as_ref(),
        &[pool.bump],
    ]];
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT,
        mint_key.as_ref(),
        &[pool.fee_vault_bump],
    ]];

    // ── 0. Create migration vault ATA if it doesn't exist ─────────────────────
    {
        let vata_info = ctx.accounts.migration_vault_token_account.to_account_info();
        if vata_info.data_is_empty() {
            let payer_info = ctx.accounts.payer.to_account_info();
            let mint_info = ctx.accounts.mint.to_account_info();
            let migration_vault_info = ctx.accounts.migration_vault.to_account_info();
            let token_program_info = ctx.accounts.token_program.to_account_info();
            let system_program_info = ctx.accounts.system_program.to_account_info();

            let rent = Rent::get()?;
            let token_account_len = SplTokenAccount::LEN;
            let lamports = rent.minimum_balance(token_account_len);

            let create_ix = anchor_lang::solana_program::system_instruction::create_account(
                &payer_info.key(),
                &vata_info.key(),
                lamports,
                token_account_len as u64,
                &spl_token::id(),
            );
            anchor_lang::solana_program::program::invoke(
                &create_ix,
                &[payer_info.clone(), vata_info.clone(), system_program_info.clone()],
            )?;

            let init_ix = spl_token::instruction::initialize_account3(
                &spl_token::id(),
                &vata_info.key(),
                &mint_info.key(),
                &migration_vault_info.key(),
            )?;
            anchor_lang::solana_program::program::invoke(
                &init_ix,
                &[
                    vata_info.clone(),
                    mint_info.clone(),
                    token_program_info.clone(),
                    migration_vault_info.clone(),
                ],
            )?;

            msg!("Created migration vault ATA");
        }
    }

    // ── 1. Send 300M LP tokens from lp_reserve → DEX token account ────────────
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

    // ── 2. Handle remaining pool tokens: half burn, half to migration vault ───
    // This is the key logic: at migration, ALL remaining tokens in the pool
    // (bonding leftovers + reserve leftovers) are split 50/50:
    //   - Half burned (permanently removed from supply)
    //   - Half sent to migration_vault (claimable by creator)
    let remaining = ctx.accounts.pool_token_account.amount;
    let (burn_amount, vault_amount) = if remaining > 0 {
        let half = remaining / 2;
        if half > 0 {
            token::burn(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.mint.to_account_info(),
                        from: ctx.accounts.pool_token_account.to_account_info(),
                        authority: pool.to_account_info(),
                    },
                    pool_state_seeds,
                ),
                half,
            )?;
        }
        let to_vault = remaining - half;
        if to_vault > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.pool_token_account.to_account_info(),
                        to: ctx.accounts.migration_vault_token_account.to_account_info(),
                        authority: pool.to_account_info(),
                    },
                    pool_state_seeds,
                ),
                to_vault,
            )?;
        }
        (half, to_vault)
    } else {
        (0, 0)
    };

    msg!("Migration token split: {} burned, {} to migration vault", burn_amount, vault_amount);

    // ── 3. Send all SOL from fee_vault → DEX pool ─────────────────────────────
    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.fee_vault.to_account_info(),
                to: ctx.accounts.dex_pool.to_account_info(),
            },
            fee_vault_seeds,
        ),
        sol_to_deposit,
    )?;

    // ── 4. Mark graduated ──────────────────────────────────────────────────────
    pool.graduated = true;
    pool.dex_pool = Some(ctx.accounts.dex_pool.key());
    pool.real_sol_reserves = 0;
    pool.real_token_reserves = 0;
    pool.reserve_tokens_remaining = 0;

    let now = Clock::get()?.unix_timestamp;
    emit!(MigrateEvent {
        mint: mint_key,
        migration_target: pool.migration_target.clone(),
        sol_deposited: sol_to_deposit,
        tokens_deposited: LP_RESERVE_SUPPLY,
        tokens_burned: burn_amount,
        tokens_to_migration_vault: vault_amount,
        dex_pool: ctx.accounts.dex_pool.key(),
        timestamp: now,
    });

    msg!(
        "🎓 MIGRATED {} | SOL={} | LP={} | Burned={} | Vault={} | DEX={}",
        mint_key, sol_to_deposit, LP_RESERVE_SUPPLY, burn_amount, vault_amount, ctx.accounts.dex_pool.key()
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Accounts
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Box<Account<'info, GlobalConfig>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// Pre-configured migration parameters (created during token creation)
    #[account(
        seeds = [SEED_MIGRATION_CONFIG, mint.key().as_ref()],
        bump = migration_config.bump,
    )]
    pub migration_config: Box<Account<'info, MigrationConfig>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_POOL_TOKENS, mint.key().as_ref()],
        bump = pool_state.pool_tokens_bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [SEED_LP_RESERVE, mint.key().as_ref()],
        bump = pool_state.lp_reserve_bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub lp_reserve_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: migration vault ATA — created via CPI if needed
    #[account(mut)]
    pub migration_vault_token_account: UncheckedAccount<'info>,

    /// CHECK: migration_vault PDA — program-controlled
    #[account(seeds = [SEED_MIGRATION_VAULT, mint.key().as_ref()], bump = pool_state.migration_vault_bump)]
    pub migration_vault: UncheckedAccount<'info>,

    /// CHECK: fee_vault PDA — holds the bonding curve SOL to send to DEX
    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump = pool_state.fee_vault_bump)]
    pub fee_vault: UncheckedAccount<'info>,

    /// DEX pool account — validated against MigrationConfig
    #[account(mut, address = migration_config.dex_pool)]
    pub dex_pool: UncheckedAccount<'info>,

    /// DEX token account — validated against MigrationConfig
    #[account(mut, address = migration_config.dex_token_account)]
    pub dex_token_account: UncheckedAccount<'info>,

    /// permissionless — anyone can trigger graduation
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
