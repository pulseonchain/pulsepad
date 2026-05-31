// ─── Platform ────────────────────────────────────────────────────────────────
pub const PLATFORM_WALLET: &str = "EobUZD7H6TQRYfzqKsYEYekpKoFinKW1UWA4TsHidTqj";

// ─── Token Supply ─────────────────────────────────────────────────────────────
pub const TOKEN_DECIMALS: u8 = 6;
pub const TOTAL_SUPPLY: u64 = 1_097_052_391_304_347;
pub const BONDING_SUPPLY: u64 = 700_000_000_000_000;
pub const RESERVE_SUPPLY: u64 =   97_052_391_304_347;
pub const LP_RESERVE_SUPPLY: u64 = 300_000_000_000_000;

// ─── Bonding Curve ────────────────────────────────────────────────────────────
pub const INITIAL_VIRTUAL_SOL: u64 = 30_000_000_000;
pub const INITIAL_VIRTUAL_TOKEN: u64 = 1_073_000_000_000_000;

// ─── Graduation Tiers ─────────────────────────────────────────────────────────
pub const GRADUATION_TIER_FAST_SOL: u64     = 80_000_000_000;
pub const GRADUATION_TIER_STANDARD_SOL: u64 = 150_000_000_000;
pub const GRADUATION_TIER_STABLE_SOL: u64   = 240_000_000_000;
pub const GRADUATION_SOL_THRESHOLD: u64 = 85_000_000_000; // backwards-compat

use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Default)]
pub enum GraduationTier {
    Fast,
    #[default]
    Standard,
    Stable,
}

impl GraduationTier {
    pub fn threshold_sol(&self) -> u64 {
        match self {
            GraduationTier::Fast     => GRADUATION_TIER_FAST_SOL,
            GraduationTier::Standard => GRADUATION_TIER_STANDARD_SOL,
            GraduationTier::Stable   => GRADUATION_TIER_STABLE_SOL,
        }
    }

    pub fn from_threshold(threshold: u64) -> Self {
        if threshold <= GRADUATION_TIER_FAST_SOL { GraduationTier::Fast }
        else if threshold <= GRADUATION_TIER_STANDARD_SOL { GraduationTier::Standard }
        else { GraduationTier::Stable }
    }
}

// ─── Anti-Snipe ───────────────────────────────────────────────────────────────
pub const ANTI_SNIPE_WINDOW_SECS: i64   = 180;
pub const ANTI_SNIPE_MULTIPLIER_BASIS: u64 = 300;

// ─── Agent / Vault ────────────────────────────────────────────────────────────
pub const MAX_VAULT_CLAIM_PER_24H: u64 = 500_000_000_000;
pub const VAULT_CLAIM_COOLDOWN_SECS: i64 = 24 * 3600;

// ─── Partial Migration ────────────────────────────────────────────────────────
pub const PARTIAL_MIGRATION_OPTIONS: [u8; 4] = [0, 10, 20, 30];

// ─── Fees ─────────────────────────────────────────────────────────────────────
pub const TOTAL_FEE_BPS: u64 = 100;
pub const PLATFORM_SHARE_BPS: u64 = 75;
pub const CREATOR_SHARE_BPS: u64 = 25;
pub const MAX_FEE_BPS: u64 = 500;
pub const PLATFORM_FRACTION: u64 = 75;
pub const MIN_CREATOR_FEE_BPS: u64 = 100;
pub const MAX_CREATOR_FEE_BPS: u64 = 500;

// ─── Limits ───────────────────────────────────────────────────────────────────
pub const MIN_CREATOR_RESERVE: u64 = 5_000_000;
pub const MAX_TRADE_SOL: u64 = 10_000_000_000;
pub const MAX_TRADE_TOKENS: u64 = 1_000_000_000_000_000;

// ─── PDA Seeds ────────────────────────────────────────────────────────────────
pub const SEED_GLOBAL_CONFIG: &[u8] = b"global_config";
pub const SEED_POOL_STATE: &[u8]    = b"pool_state";
pub const SEED_FEE_VAULT: &[u8]     = b"fee_vault";
pub const SEED_FEE_RECIPIENT: &[u8] = b"fee_recipient";
pub const SEED_POOL_TOKENS: &[u8]   = b"pool_tokens";
pub const SEED_LP_RESERVE: &[u8]    = b"lp_reserve";
pub const SEED_STAKE: &[u8]         = b"stake";
pub const SEED_LP_TOKEN_VAULT: &[u8]= b"lp_token_vault";
pub const SEED_MIGRATION_VAULT: &[u8] = b"migration_vault";
pub const SEED_MIGRATION_CONFIG: &[u8] = b"migration_config";
pub const SEED_AGENT: &[u8]       = b"agent";
pub const SEED_BUYBACK: &[u8]       = b"buyback";
pub const AGENT_NAME_PREFIX: &str = "Agent ";

// ─── External Programs ────────────────────────────────────────────────────────
pub const METAPLEX_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
pub const METEORA_DAMM_PROGRAM_ID: &str  = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EkSCmrAP";
pub const METEORA_DLMM_PROGRAM_ID: &str  = "LBUZKhRxPF3XUpBCjp4YzTKgLLjggiJmV1fTTCkUscX";
pub const PUMP_SWAP_PROGRAM_ID: &str     = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
