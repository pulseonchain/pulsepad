use anchor_lang::prelude::*;

pub mod consts;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;
pub mod security;
pub mod invariants;
pub mod analytics;
pub mod economics;
pub mod upgrades;
pub mod compliance;
pub mod utils;

use instructions::*;
use state::MigrationTarget;
use crate::upgrades::Feature;
use consts::GraduationTier;

// Replace with your actual program ID after `anchor build`
declare_id!("5NLh9rQPR4EAZZpZfAJ3ujszffKjMJJCEGXxCBf4CRea");

#[program]
pub mod cto_bonding {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════
    //  CORE INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    // ─── Admin ────────────────────────────────────────────────────────────────
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    // ─── Token Creation (3-step to stay within SBF stack limits) ──────────────
    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        migration_target: MigrationTarget,
    ) -> Result<()> {
        instructions::create_token::create_token(ctx, name, symbol, uri, migration_target)
    }

    pub fn create_token_accounts(ctx: Context<CreateTokenAccounts>) -> Result<()> {
        instructions::create_token::create_token_accounts(ctx)
    }

    pub fn create_staker_vault(ctx: Context<CreateStakerVault>) -> Result<()> {
        instructions::create_token::create_staker_vault(ctx)
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        initial_sol_deposit: u64,
    ) -> Result<()> {
        instructions::create_token::initialize_pool(ctx, initial_sol_deposit)
    }

    // ─── Trading ──────────────────────────────────────────────────────────────
    pub fn buy(
        ctx: Context<Buy>,
        sol_amount: u64,
        min_tokens_out: u64,
    ) -> Result<()> {
        instructions::buy::buy(ctx, sol_amount, min_tokens_out)
    }

    pub fn sell(
        ctx: Context<Sell>,
        token_amount: u64,
        min_sol_out: u64,
    ) -> Result<()> {
        instructions::sell::sell(ctx, token_amount, min_sol_out)
    }

    // ─── Graduation ───────────────────────────────────────────────────────────
    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        instructions::migrate::migrate(ctx)
    }

    // ─── Creator Fee Management ───────────────────────────────────────────────
    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        instructions::claim_fees::claim_fees(ctx)
    }

    pub fn transfer_authority(
        ctx: Context<TransferAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::transfer_authority::transfer_authority(ctx, new_authority)
    }

    pub fn claim_lp_fees(ctx: Context<ClaimLpFees>) -> Result<()> {
        instructions::claim_lp_fees::claim_lp_fees(ctx)
    }

    pub fn claim_migration_vault(ctx: Context<ClaimMigrationVault>) -> Result<()> {
        instructions::claim_migration_vault::claim_migration_vault(ctx)
    }

    // ─── Staking ──────────────────────────────────────────────────────────────
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake::stake(ctx, amount)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        instructions::unstake::unstake(ctx, amount)
    }

    pub fn claim_staker_rewards(ctx: Context<ClaimStakerRewards>) -> Result<()> {
        instructions::claim_staker_rewards::claim_staker_rewards(ctx)
    }

    // ─── Admin Extensions ──────────────────────────────────────────────────────
    pub fn update_global_config(
        ctx: Context<UpdateGlobalConfig>,
        params: instructions::update_global_config::UpdateGlobalConfigParams,
    ) -> Result<()> {
        instructions::update_global_config::update_global_config(ctx, params)
    }

    pub fn close_pool(ctx: Context<ClosePool>) -> Result<()> {
        instructions::close_pool::close_pool(ctx)
    }

    // ─── Whitelist Presale ──────────────────────────────────────────────────────
    pub fn whitelist_buy(
        ctx: Context<WhitelistBuy>,
        sol_amount: u64,
        min_tokens_out: u64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::whitelist_buy::whitelist_buy(ctx, sol_amount, min_tokens_out, merkle_proof)
    }

    // ─── Referral System ────────────────────────────────────────────────────────
    pub fn create_referral_config(
        ctx: Context<CreateReferralConfig>,
        referral_share_bps: u16,
    ) -> Result<()> {
        instructions::referral::create_referral_config(ctx, referral_share_bps)
    }

    pub fn set_referral_active(
        ctx: Context<SetReferralActive>,
        active: bool,
    ) -> Result<()> {
        instructions::referral::set_referral_active(ctx, active)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  SECURITY MODULE INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn init_reentrancy_guard(ctx: Context<InitReentrancyGuard>) -> Result<()> {
        instructions::init_reentrancy_guard::init_reentrancy_guard(ctx)
    }

    pub fn enter_reentrancy_guard(ctx: Context<EnterReentrancyGuard>) -> Result<()> {
        instructions::enter_reentrancy_guard::enter_reentrancy_guard(ctx)
    }

    pub fn exit_reentrancy_guard(ctx: Context<ExitReentrancyGuard>) -> Result<()> {
        instructions::exit_reentrancy_guard::exit_reentrancy_guard(ctx)
    }

    pub fn init_circuit_breaker(ctx: Context<InitCircuitBreaker>) -> Result<()> {
        instructions::init_circuit_breaker::init_circuit_breaker(ctx)
    }

    pub fn trigger_circuit_breaker(
        ctx: Context<TriggerCircuitBreaker>,
        duration_seconds: i64,
    ) -> Result<()> {
        instructions::trigger_circuit_breaker::trigger_circuit_breaker(ctx, duration_seconds)
    }

    pub fn reset_circuit_breaker(ctx: Context<ResetCircuitBreaker>) -> Result<()> {
        instructions::reset_circuit_breaker::reset_circuit_breaker(ctx)
    }

    pub fn init_rate_limiter(ctx: Context<InitRateLimiter>) -> Result<()> {
        instructions::init_rate_limiter::init_rate_limiter(ctx)
    }

    pub fn check_rate_limit(
        ctx: Context<CheckRateLimit>,
        sol_amount: u64,
    ) -> Result<()> {
        instructions::check_rate_limit::check_rate_limit(ctx, sol_amount)
    }

    pub fn init_address_filter(
        ctx: Context<InitAddressFilter>,
        filter_type: u8,
    ) -> Result<()> {
        instructions::init_address_filter::init_address_filter(ctx, filter_type)
    }

    pub fn remove_address_filter(ctx: Context<RemoveAddressFilter>) -> Result<()> {
        instructions::remove_address_filter::remove_address_filter(ctx)
    }

    pub fn init_flash_loan_detector(ctx: Context<InitFlashLoanDetector>) -> Result<()> {
        instructions::init_flash_loan_detector::init_flash_loan_detector(ctx)
    }

    pub fn record_flash_loan_check(
        ctx: Context<RecordFlashLoanCheck>,
        volume: u64,
    ) -> Result<()> {
        instructions::record_flash_loan_check::record_flash_loan_check(ctx, volume)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  ANALYTICS MODULE INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn init_metrics(ctx: Context<InitMetrics>) -> Result<()> {
        instructions::init_metrics::init_metrics(ctx)
    }

    pub fn record_buy_metrics(
        ctx: Context<RecordBuyMetrics>,
        sol_amount: u64,
        tokens_out: u64,
        platform_fee: u64,
        creator_fee: u64,
    ) -> Result<()> {
        instructions::record_buy_metrics::record_buy_metrics(ctx, sol_amount, tokens_out, platform_fee, creator_fee)
    }

    pub fn record_sell_metrics(
        ctx: Context<RecordSellMetrics>,
        sol_amount: u64,
        tokens_in: u64,
        platform_fee: u64,
        creator_fee: u64,
    ) -> Result<()> {
        instructions::record_sell_metrics::record_sell_metrics(ctx, sol_amount, tokens_in, platform_fee, creator_fee)
    }

    pub fn init_pool_health(ctx: Context<InitPoolHealth>) -> Result<()> {
        instructions::init_pool_health::init_pool_health(ctx)
    }

    pub fn update_pool_health(
        ctx: Context<UpdatePoolHealth>,
    ) -> Result<()> {
        instructions::update_pool_health::update_pool_health(ctx)
    }

    pub fn init_user_stats(ctx: Context<InitUserStats>) -> Result<()> {
        instructions::init_user_stats::init_user_stats(ctx)
    }

    pub fn record_user_buy(
        ctx: Context<RecordUserBuy>,
        volume: u64,
        fees: u64,
    ) -> Result<()> {
        instructions::record_user_buy::record_user_buy(ctx, volume, fees)
    }

    pub fn record_user_sell(
        ctx: Context<RecordUserSell>,
        volume: u64,
        fees: u64,
    ) -> Result<()> {
        instructions::record_user_sell::record_user_sell(ctx, volume, fees)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  ECONOMICS MODULE INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn init_dynamic_fees(ctx: Context<InitDynamicFees>) -> Result<()> {
        instructions::init_dynamic_fees::init_dynamic_fees(ctx)
    }

    pub fn update_dynamic_fee(
        ctx: Context<UpdateDynamicFee>,
        sol_reserves: u64,
        token_reserves: u64,
        volatility_bps: u64,
    ) -> Result<()> {
        instructions::update_dynamic_fee::update_dynamic_fee(ctx, sol_reserves, token_reserves, volatility_bps)
    }

    pub fn init_liquidity_bootstrap(
        ctx: Context<InitLiquidityBootstrap>,
        duration_seconds: u64,
    ) -> Result<()> {
        instructions::init_liquidity_bootstrap::init_liquidity_bootstrap(ctx, duration_seconds)
    }

    pub fn get_bootstrap_pricing(
        ctx: Context<GetBootstrapPricing>,
    ) -> Result<()> {
        instructions::get_bootstrap_pricing::get_bootstrap_pricing(ctx)
    }

    pub fn init_whale_protection(ctx: Context<InitWhaleProtection>) -> Result<()> {
        instructions::init_whale_protection::init_whale_protection(ctx)
    }

    pub fn calculate_whale_fee(
        ctx: Context<CalculateWhaleFee>,
        trade_volume: u64,
    ) -> Result<()> {
        instructions::calculate_whale_fee::calculate_whale_fee(ctx, trade_volume)
    }

    pub fn init_fee_redistribution(ctx: Context<InitFeeRedistribution>) -> Result<()> {
        instructions::init_fee_redistribution::init_fee_redistribution(ctx)
    }

    pub fn redistribute_fees(
        ctx: Context<RedistributeFees>,
        total_fees: u64,
    ) -> Result<()> {
        instructions::redistribute_fees::redistribute_fees(ctx, total_fees)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  UPGRADES MODULE INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn init_upgradeable(ctx: Context<InitUpgradeable>) -> Result<()> {
        instructions::init_upgradeable::init_upgradeable(ctx)
    }

    pub fn schedule_upgrade(
        ctx: Context<ScheduleUpgrade>,
        next_version: u32,
        delay_seconds: u64,
    ) -> Result<()> {
        instructions::schedule_upgrade::schedule_upgrade(ctx, next_version, delay_seconds)
    }

    pub fn complete_upgrade(ctx: Context<CompleteUpgrade>) -> Result<()> {
        instructions::complete_upgrade::complete_upgrade(ctx)
    }

    pub fn cancel_upgrade(ctx: Context<CancelUpgrade>) -> Result<()> {
        instructions::cancel_upgrade::cancel_upgrade(ctx)
    }

    pub fn init_feature_flags(ctx: Context<InitFeatureFlags>) -> Result<()> {
        instructions::init_feature_flags::init_feature_flags(ctx)
    }

    // Note: toggle_feature removed - use set_feature_bitmap for feature control

    pub fn set_feature_bitmap(
        ctx: Context<SetFeatureBitmap>,
        bitmap: u128,
    ) -> Result<()> {
        instructions::set_feature_bitmap::set_feature_bitmap(ctx, bitmap)
    }

    pub fn init_plugin(
        ctx: Context<InitPlugin>,
        plugin_type: u8,
        config_data: [u8; 256],
    ) -> Result<()> {
        instructions::init_plugin::init_plugin(ctx, plugin_type, config_data)
    }

    pub fn execute_plugin(
        ctx: Context<ExecutePlugin>,
    ) -> Result<()> {
        instructions::execute_plugin::execute_plugin(ctx)
    }

    pub fn init_upgrade_path(ctx: Context<InitUpgradePath>) -> Result<()> {
        instructions::init_upgrade_path::init_upgrade_path(ctx)
    }

    pub fn schedule_upgrade_path(
        ctx: Context<ScheduleUpgradePath>,
        version: u32,
        scheduled_at: i64,
        deadline: i64,
        delay_seconds: u64,
    ) -> Result<()> {
        instructions::schedule_upgrade_path::schedule_upgrade_path(ctx, version, scheduled_at, deadline, delay_seconds)
    }

    pub fn execute_upgrade_path(
        ctx: Context<ExecuteUpgradePath>,
        version: u32,
    ) -> Result<()> {
        instructions::execute_upgrade_path::execute_upgrade_path(ctx, version)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  COMPLIANCE MODULE INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn init_compliance(
        ctx: Context<InitCompliance>,
        kyc_required: bool,
    ) -> Result<()> {
        instructions::init_compliance::init_compliance(ctx, kyc_required)
    }

    pub fn check_compliance(
        ctx: Context<CheckCompliance>,
        kyc_expiry: i64,
    ) -> Result<()> {
        instructions::check_compliance::check_compliance(ctx, kyc_expiry)
    }

    pub fn init_audit_log(ctx: Context<InitAuditLog>) -> Result<()> {
        instructions::init_audit_log::init_audit_log(ctx)
    }

    pub fn log_audit_event(
        ctx: Context<LogAuditEvent>,
        log_type: u8,
        details: [u8; 64],
    ) -> Result<()> {
        instructions::log_audit_event::log_audit_event(ctx, log_type, details)
    }

    pub fn verify_whitelist_proof(
        ctx: Context<VerifyWhitelistProof>,
        inclusion_proof: Vec<u8>,
        root_hash: [u8; 32],
    ) -> Result<()> {
        instructions::verify_whitelist_proof::verify_whitelist_proof(ctx, inclusion_proof, root_hash)
    }

    pub fn verify_referral_proof(
        ctx: Context<VerifyReferralProof>,
        referral_proof: Vec<u8>,
        root_hash: [u8; 32],
    ) -> Result<()> {
        instructions::verify_referral_proof::verify_referral_proof(ctx, referral_proof, root_hash)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  INVARIANT CHECKER INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn verify_pool_invariants(
        ctx: Context<VerifyPoolInvariants>,
    ) -> Result<()> {
        instructions::verify_pool_invariants::verify_pool_invariants(ctx)
    }

    pub fn verify_math_operations(
        ctx: Context<VerifyMathOps>,
        before: u64,
        after: u64,
        change: u64,
        operation: u8,
    ) -> Result<()> {
        instructions::verify_math_ops_instruction::verify_math_operations(ctx, before, after, change, operation)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  TIME-LOCKED AUTHORITY INSTRUCTION
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn schedule_authority_transfer(
        ctx: Context<ScheduleAuthorityTransfer>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::schedule_authority_transfer::schedule_authority_transfer(ctx, new_authority)
    }

    pub fn execute_authority_transfer(
        ctx: Context<ExecuteAuthorityTransfer>,
    ) -> Result<()> {
        instructions::execute_authority_transfer::execute_authority_transfer(ctx)
    }

    pub fn cancel_authority_transfer(
        ctx: Context<CancelAuthorityTransfer>,
    ) -> Result<()> {
        instructions::cancel_authority_transfer::cancel_authority_transfer(ctx)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    //  AGENTIC / PREBOND INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn init_prebond(
        ctx: Context<InitPrebond>,
        graduation_tier: GraduationTier,
        total_fee_bps: u64,
        fees_to_agent: bool,
        agent_name: String,
        anti_snipe_enabled: bool,
        partial_migration_pct: u8,
    ) -> Result<()> {
        instructions::prebond::init_prebond(ctx, graduation_tier, total_fee_bps, fees_to_agent, agent_name, anti_snipe_enabled, partial_migration_pct)
    }

    pub fn agent_claim(ctx: Context<AgentClaim>) -> Result<()> {
        instructions::agent::agent_claim::agent_claim(ctx)
    }

    pub fn agent_buyback(ctx: Context<AgentBuyback>, sol_to_spend: u64, burn_pct: u8) -> Result<()> {
        instructions::agent::agent_buyback::agent_buyback(ctx, sol_to_spend, burn_pct)
    }

    pub fn agent_transfer(ctx: Context<AgentTransfer>, amount: u64) -> Result<()> {
        instructions::agent::agent_transfer::agent_transfer(ctx, amount)
    }

    pub fn claim_vault_capped(ctx: Context<ClaimVaultCapped>, amount: u64) -> Result<()> {
        instructions::prebond::claim_vault_capped(ctx, amount)
    }
}
