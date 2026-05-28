use anchor_lang::prelude::*;

pub mod consts;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;
use state::MigrationTarget;

// Replace with your actual program ID after `anchor build`
declare_id!("5NLh9rQPR4EAZZpZfAJ3ujszffKjMJJCEGXxCBf4CRea");

#[program]
pub mod cto_bonding {
    use super::*;

    // ─── Admin ────────────────────────────────────────────────────────────────
    /// One-time setup: creates the GlobalConfig PDA.
    /// Must be called by the platform wallet before any tokens can be created.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    // ─── Token Creation (3-step to stay within SBF stack limits) ──────────────
    /// Step 1: Creates mint, metadata, and pool state.
    /// Call create_token_accounts (step 1b) and initialize_pool (step 2) after.
    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        migration_target: MigrationTarget,
    ) -> Result<()> {
        instructions::create_token::create_token(ctx, name, symbol, uri, migration_target)
    }

    /// Step 1b: Creates pool token account and LP reserve account.
    /// Must be called after create_token and before create_staker_vault.
    pub fn create_token_accounts(ctx: Context<CreateTokenAccounts>) -> Result<()> {
        instructions::create_token::create_token_accounts(ctx)
    }

    /// Step 1c: Creates staker vault, fee vault, fee recipient, and migration vault.
    /// Must be called after create_token_accounts and before initialize_pool.
    pub fn create_staker_vault(ctx: Context<CreateStakerVault>) -> Result<()> {
        instructions::create_token::create_staker_vault(ctx)
    }

    /// Step 2: Mint 700M bonding + 97M reserve + 300M LP tokens, revoke mint authority,
    /// and deposit initial SOL. Must be called after create_staker_vault.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        initial_sol_deposit: u64,
    ) -> Result<()> {
        instructions::create_token::initialize_pool(ctx, initial_sol_deposit)
    }

    // ─── Trading ──────────────────────────────────────────────────────────────
    /// Buy tokens from the bonding curve.
    /// Takes 1% fee from sol_amount: 0.75% to platform, 0.25% to creator.
    /// Remaining 99% executes the constant-product swap.
    pub fn buy(
        ctx: Context<Buy>,
        sol_amount: u64,
        min_tokens_out: u64,
    ) -> Result<()> {
        instructions::buy::buy(ctx, sol_amount, min_tokens_out)
    }

    /// Sell tokens back to the bonding curve.
    /// Takes 1% fee from gross SOL output: 0.75% to platform, 0.25% to creator.
    pub fn sell(
        ctx: Context<Sell>,
        token_amount: u64,
        min_sol_out: u64,
    ) -> Result<()> {
        instructions::sell::sell(ctx, token_amount, min_sol_out)
    }

    // ─── Graduation ───────────────────────────────────────────────────────────
    /// Permissionless: graduates the token to the chosen DEX once 85 SOL is raised.
    /// Sends 300M LP tokens + all SOL to the DEX pool.
    /// Remaining bonding+reserve tokens: half burned, half sent to migration_vault.
    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        instructions::migrate::migrate(ctx)
    }

    // ─── Creator Fee Management ───────────────────────────────────────────────
    /// Claim accumulated creator fees from fee_recipient (Wallet 2).
    /// Leaves 0.005 SOL minimum for future gas. Signer must be current_authority.
    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        instructions::claim_fees::claim_fees(ctx)
    }

    /// Permanently transfer fee-claiming authority to a new wallet.
    /// Old wallet loses all access. New wallet takes full control immediately.
    pub fn transfer_authority(
        ctx: Context<TransferAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::transfer_authority::transfer_authority(ctx, new_authority)
    }

    // ─── Post-Graduation LP Fee Claiming ─────────────────────────────────────
    /// Permissionless crank: claims LP fees from Raydium or PumpSwap (Hold LP mode).
    /// Splits claimed SOL 0.75% to platform, 0.25% to creator fee_recipient.
    pub fn claim_lp_fees(ctx: Context<ClaimLpFees>) -> Result<()> {
        instructions::claim_lp_fees::claim_lp_fees(ctx)
    }

    // ─── Migration Vault ─────────────────────────────────────────────────────
    /// Claim tokens from the migration vault (half of remaining tokens at migration).
    /// Only callable by current_authority after graduation.
    pub fn claim_migration_vault(ctx: Context<ClaimMigrationVault>) -> Result<()> {
        instructions::claim_migration_vault::claim_migration_vault(ctx)
    }

    // ─── Staking ──────────────────────────────────────────────────────────────
    /// Stake tokens to earn a share of post-graduation creator fees.
    /// Only relevant for Meteora targets with staker_share > 0.
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake::stake(ctx, amount)
    }

    /// Unstake tokens. Snapshots any pending rewards before reducing stake.
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        instructions::unstake::unstake(ctx, amount)
    }

    /// Claim pending staker SOL rewards.
    pub fn claim_staker_rewards(ctx: Context<ClaimStakerRewards>) -> Result<()> {
        instructions::claim_staker_rewards::claim_staker_rewards(ctx)
    }

    // ─── Admin Extensions ──────────────────────────────────────────────────────
    /// Update global config parameters (admin only).
    /// Every field is optional — pass None to keep the existing value.
    pub fn update_global_config(
        ctx: Context<UpdateGlobalConfig>,
        params: instructions::update_global_config::UpdateGlobalConfigParams,
    ) -> Result<()> {
        instructions::update_global_config::update_global_config(ctx, params)
    }

    /// Close a non-graduated pool. Burns all tokens, returns SOL to creator.
    /// Requires both platform authority AND creator to co-sign.
    pub fn close_pool(ctx: Context<ClosePool>) -> Result<()> {
        instructions::close_pool::close_pool(ctx)
    }

    // ─── Whitelist Presale ──────────────────────────────────────────────────────
    /// Whitelist-gated buy during the presale phase.
    /// Requires a valid Merkle proof that the buyer is on the whitelist.
    /// Supports per-wallet caps and optional discounted fees.
    pub fn whitelist_buy(
        ctx: Context<WhitelistBuy>,
        sol_amount: u64,
        min_tokens_out: u64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::whitelist_buy::whitelist_buy(ctx, sol_amount, min_tokens_out, merkle_proof)
    }

    // ─── Referral System ────────────────────────────────────────────────────────
    /// Create a referral config for a partner wallet (platform admin only).
    pub fn create_referral_config(
        ctx: Context<CreateReferralConfig>,
        referral_share_bps: u16,
    ) -> Result<()> {
        instructions::referral::create_referral_config(ctx, referral_share_bps)
    }

    /// Enable or disable a referral config (platform admin only).
    pub fn set_referral_active(
        ctx: Context<SetReferralActive>,
        active: bool,
    ) -> Result<()> {
        instructions::referral::set_referral_active(ctx, active)
    }
}
