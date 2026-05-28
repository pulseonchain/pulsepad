use anchor_lang::prelude::*;
use crate::state::MigrationTarget;

#[event]
pub struct TokenCreatedEvent {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub migration_target: MigrationTarget,
    pub timestamp: i64,
}

#[event]
pub struct BuyEvent {
    pub mint: Pubkey,
    pub buyer: Pubkey,
    pub sol_amount: u64,
    pub tokens_out: u64,
    pub from_bonding: u64,
    pub from_reserve: u64,
    pub platform_fee: u64,
    pub creator_fee: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub timestamp: i64,
}

#[event]
pub struct SellEvent {
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub token_amount: u64,
    pub sol_out: u64,
    pub platform_fee: u64,
    pub creator_fee: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub timestamp: i64,
}

#[event]
pub struct MigrateEvent {
    pub mint: Pubkey,
    pub migration_target: MigrationTarget,
    pub sol_deposited: u64,
    pub tokens_deposited: u64,
    pub tokens_burned: u64,
    pub tokens_to_migration_vault: u64,
    pub dex_pool: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct FeeClaimedEvent {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct AuthorityTransferredEvent {
    pub mint: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct LpFeeClaimedEvent {
    pub mint: Pubkey,
    pub platform_amount: u64,
    pub creator_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct StakeEvent {
    pub mint: Pubkey,
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct UnstakeEvent {
    pub mint: Pubkey,
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct StakeRewardClaimedEvent {
    pub mint: Pubkey,
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct GraduationReadyEvent {
    pub mint: Pubkey,
    pub real_sol_reserves: u64,
    pub threshold: u64,
    pub timestamp: i64,
}

// ─── New Events ────────────────────────────────────────────────────────────────────────

#[event]
pub struct PoolClosedEvent {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub closed_by_platform: Pubkey,
    pub pool_tokens_burned: u64,
    pub lp_tokens_burned: u64,
    pub sol_returned: u64,
    pub timestamp: i64,
}

#[event]
pub struct WhitelistBuyEvent {
    pub mint: Pubkey,
    pub buyer: Pubkey,
    pub sol_amount: u64,
    pub tokens_out: u64,
    pub platform_fee: u64,
    pub creator_fee: u64,
    pub discount_applied: bool,
    pub timestamp: i64,
}

#[event]
pub struct WhitelistConfigCreatedEvent {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub whitelist_end_time: i64,
    pub max_sol_per_wallet: u64,
    pub discounted_fee_bps: u16,
    pub timestamp: i64,
}

#[event]
pub struct ReferralRewardEvent {
    pub mint: Pubkey,
    pub referrer: Pubkey,
    pub buyer_or_seller: Pubkey,
    pub referral_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct GlobalConfigUpdatedEvent {
    pub updated_by: Pubkey,
    pub field: String,
    pub timestamp: i64,
}

#[event]
pub struct MigrationConfigUpdatedEvent {
    pub mint: Pubkey,
    pub updated_by: Pubkey,
    pub new_dex_pool: Pubkey,
    pub timestamp: i64,
}
