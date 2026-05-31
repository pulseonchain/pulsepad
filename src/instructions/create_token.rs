use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, SetAuthority, Token, TokenAccount},
    metadata::{
        create_metadata_accounts_v3,
        CreateMetadataAccountsV3,
        Metadata,
    },
};
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;

use crate::consts::*;
use crate::errors::BondingError;
use crate::events::TokenCreatedEvent;
use crate::state::{GlobalConfig, MigrationConfig, MigrationTarget, PoolState, StakerVault};

pub const MIN_INITIAL_DEPOSIT: u64 = 20_000_000; // 0.02 SOL

/// Step 1: Create mint, metadata, and pool state.
pub fn create_token(
    ctx: Context<CreateToken>,
    name: String,
    symbol: String,
    uri: String,
    migration_target: MigrationTarget,
) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?;
    require!(!config.paused, BondingError::Paused);
    require!(name.len() >= 1 && name.len() <= 32,  BondingError::NameTooLong);
    require!(name.is_ascii(), BondingError::InvalidName);
    require!(symbol.len() >= 1 && symbol.len() <= 10, BondingError::SymbolTooLong);
    require!(symbol.is_ascii(), BondingError::InvalidSymbol);
    require!(uri.len() >= 1 && uri.len() <= 200, BondingError::UriTooLong);
    require!(uri.is_ascii(), BondingError::InvalidUri);
    require!(uri.starts_with("https://") || uri.starts_with("http://"), BondingError::InvalidUri);
    migration_target.validate()?;

    let mint_key    = ctx.accounts.mint.key();
    let creator_key = ctx.accounts.creator.key();

    // ── Create Metaplex metadata ──────────────────────────────────────────
    let metaplex_id: Pubkey = METAPLEX_PROGRAM_ID.parse().unwrap();
    if ctx.accounts.metadata_program.key() == metaplex_id {
        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata:         ctx.accounts.metadata.to_account_info(),
                    mint:             ctx.accounts.mint.to_account_info(),
                    mint_authority:   ctx.accounts.creator.to_account_info(),
                    payer:            ctx.accounts.creator.to_account_info(),
                    update_authority: ctx.accounts.creator.to_account_info(),
                    system_program:   ctx.accounts.system_program.to_account_info(),
                    rent:             ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name:                    name.clone(),
                symbol:                  symbol.clone(),
                uri:                     uri.clone(),
                seller_fee_basis_points: 0,
                creators:                None,
                collection:              None,
                uses:                    None,
            },
            false,
            true,
            None,
        )?;
    }

    // ── Compute bumps ─────────────────────────────────────────────────────
    let (_, fee_vault_bump) = Pubkey::find_program_address(&[SEED_FEE_VAULT, mint_key.as_ref()], &crate::ID);
    let (_, fee_recipient_bump) = Pubkey::find_program_address(&[SEED_FEE_RECIPIENT, mint_key.as_ref()], &crate::ID);
    let (_, lp_reserve_bump) = Pubkey::find_program_address(&[SEED_LP_RESERVE, mint_key.as_ref()], &crate::ID);
    let (_, pool_tokens_bump) = Pubkey::find_program_address(&[SEED_POOL_TOKENS, mint_key.as_ref()], &crate::ID);
    let (_, migration_vault_bump) = Pubkey::find_program_address(&[SEED_MIGRATION_VAULT, mint_key.as_ref()], &crate::ID);

    // ── Init pool_state (new params for prebond/agent/anti-snipe) ─────────
    let now = Clock::get()?.unix_timestamp;
    let pool = &mut ctx.accounts.pool_state;
    pool.init(
        mint_key,
        creator_key,
        migration_target.clone(),
        GraduationTier::Standard,
        true,  // anti_snipe_enabled
        0,     // partial_migration_pct
        Pubkey::default(),
        false, // fees_to_agent
        100,   // pool_fee_bps
        ctx.bumps.pool_state,
        fee_vault_bump,
        fee_recipient_bump,
        lp_reserve_bump,
        pool_tokens_bump,
        migration_vault_bump,
        now,
    );

    emit!(TokenCreatedEvent {
        mint:             mint_key,
        creator:          creator_key,
        name,
        symbol,
        uri,
        migration_target,
        graduation_tier:  "Standard".into(),
        anti_snipe_enabled: true,
        partial_migration_pct: 0,
        fees_to_agent:    false,
        agent_name:       String::new(),
        pool_fee_bps:     100,
        timestamp:        now,
    });

    msg!("Token created: {}", mint_key);
    Ok(())
}

/// Step 1b: Create pool token account and LP reserve account.
pub fn create_token_accounts(ctx: Context<CreateTokenAccounts>) -> Result<()> {
    msg!("Token accounts created for {}", ctx.accounts.mint.key());
    Ok(())
}

/// Step 1c: Create staker vault, fee vault, fee recipient, migration vault, and migration config.
pub fn create_staker_vault(ctx: Context<CreateStakerVault>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let sv = &mut ctx.accounts.staker_vault;
    sv.mint = mint_key;
    sv.total_staked = 0;
    sv.accumulated_reward_per_token = 0;
    sv.total_distributed = 0;
    sv.bump = ctx.bumps.staker_vault;

    let mc = &mut ctx.accounts.migration_config;
    mc.init(
        mint_key,
        ctx.accounts.pool_state.migration_target.clone(),
        ctx.accounts.dex_program_id.key(),
        None,
        ctx.accounts.dex_pool.key(),
        ctx.accounts.dex_token_account.key(),
        ctx.accounts.fee_recipient.key(),
        ctx.bumps.migration_config,
    );
    msg!("Staker vault + MigrationConfig created for {}", mint_key);
    Ok(())
}

/// Step 2: Mint tokens, revoke authorities, deposit initial SOL.
pub fn initialize_pool(ctx: Context<InitializePool>, initial_sol_deposit: u64) -> Result<()> {
    require!(initial_sol_deposit >= MIN_INITIAL_DEPOSIT, BondingError::ZeroSolAmount);
    let mint_key = ctx.accounts.mint.key();

    token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), MintTo { mint: ctx.accounts.mint.to_account_info(), to: ctx.accounts.pool_token_account.to_account_info(), authority: ctx.accounts.creator.to_account_info() }), BONDING_SUPPLY)?;
    token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), MintTo { mint: ctx.accounts.mint.to_account_info(), to: ctx.accounts.pool_token_account.to_account_info(), authority: ctx.accounts.creator.to_account_info() }), RESERVE_SUPPLY)?;
    token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), MintTo { mint: ctx.accounts.mint.to_account_info(), to: ctx.accounts.lp_reserve_account.to_account_info(), authority: ctx.accounts.creator.to_account_info() }), LP_RESERVE_SUPPLY)?;

    token::set_authority(CpiContext::new(ctx.accounts.token_program.to_account_info(), SetAuthority { account_or_mint: ctx.accounts.mint.to_account_info(), current_authority: ctx.accounts.creator.to_account_info() }), token::spl_token::instruction::AuthorityType::MintTokens, None)?;
    token::set_authority(CpiContext::new(ctx.accounts.token_program.to_account_info(), SetAuthority { account_or_mint: ctx.accounts.mint.to_account_info(), current_authority: ctx.accounts.creator.to_account_info() }), token::spl_token::instruction::AuthorityType::FreezeAccount, None)?;

    system_program::transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(), system_program::Transfer { from: ctx.accounts.creator.to_account_info(), to: ctx.accounts.fee_vault.to_account_info() }), initial_sol_deposit)?;

    // Set pool_init_at for anti-snipe
    let now = Clock::get()?.unix_timestamp;
    ctx.accounts.pool_state.pool_init_at = now;

    msg!("Pool initialized: {} | SOL={}", mint_key, initial_sol_deposit);
    Ok(())
}

// ─── Account Structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    #[account(init, payer = creator, mint::decimals = TOKEN_DECIMALS, mint::authority = creator, mint::freeze_authority = creator)]
    pub mint: Box<Account<'info, Mint>>,
    /// CHECK: Metaplex PDA
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(init, payer = creator, space = PoolState::ACCOUNT_SIZE, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump)]
    pub pool_state: Box<Account<'info, PoolState>>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: UncheckedAccount<'info>,
    pub rent: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CreateTokenAccounts<'info> {
    #[account(seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,
    /// CHECK: mint pubkey
    pub mint: UncheckedAccount<'info>,
    #[account(init, payer = creator, seeds = [SEED_POOL_TOKENS, mint.key().as_ref()], bump, token::mint = mint, token::authority = pool_state)]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = creator, seeds = [SEED_LP_RESERVE, mint.key().as_ref()], bump, token::mint = mint, token::authority = pool_state)]
    pub lp_reserve_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateStakerVault<'info> {
    /// CHECK: mint pubkey
    pub mint: UncheckedAccount<'info>,
    #[account(init, payer = creator, space = StakerVault::ACCOUNT_SIZE, seeds = [StakerVault::SEED, mint.key().as_ref()], bump)]
    pub staker_vault: Box<Account<'info, StakerVault>>,
    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump)]
    pub fee_vault: UncheckedAccount<'info>,
    #[account(mut, seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()], bump)]
    pub fee_recipient: UncheckedAccount<'info>,
    #[account(mut, seeds = [SEED_MIGRATION_VAULT, mint.key().as_ref()], bump)]
    pub migration_vault: UncheckedAccount<'info>,
    #[account(init, payer = creator, space = MigrationConfig::ACCOUNT_SIZE, seeds = [SEED_MIGRATION_CONFIG, mint.key().as_ref()], bump)]
    pub migration_config: Box<Account<'info, MigrationConfig>>,
    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump)]
    pub pool_state: Account<'info, PoolState>,
    pub migration_target: UncheckedAccount<'info>,
    pub dex_program_id: UncheckedAccount<'info>,
    pub dex_pool: UncheckedAccount<'info>,
    pub dex_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut, seeds = [SEED_POOL_TOKENS, mint.key().as_ref()], bump = pool_state.pool_tokens_bump, token::mint = mint, token::authority = pool_state)]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [SEED_LP_RESERVE, mint.key().as_ref()], bump = pool_state.lp_reserve_bump, token::mint = mint, token::authority = pool_state)]
    pub lp_reserve_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump = pool_state.fee_vault_bump)]
    pub fee_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
