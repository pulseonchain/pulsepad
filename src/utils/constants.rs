use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Constants - All protocol constants in one place
// ─────────────────────────────────────────────────────────────────────────────

// ─── Platform ────────────────────────────────────────────────────────────────
pub const PLATFORM_WALLET: &str = "EobUZD7H6TQRYfzqKsYEYekpKoFinKW1UWA4TsHidTqj";

// ─── Token Supply ─────────────────────────────────────────────────────────────
pub const TOKEN_DECIMALS: u8 = 6;
pub const TOTAL_SUPPLY: u64 = 1_097_052_391_304_347;   // ~1.097B × 10^6
pub const BONDING_SUPPLY: u64 = 700_000_000_000_000;   //   700 000 000 × 10^6
pub const RESERVE_SUPPLY: u64 =   97_052_391_304_347;  //    97 052 391 × 10^6
pub const LP_RESERVE_SUPPLY: u64 = 300_000_000_000_000; //   300 000 000 × 10^6

// ─── Bonding Curve (constant-product with virtual reserves) ──────────────────
// Mirrors PumpFun's approach: price = virtual_sol / virtual_tokens
// Initial price ≈ 30 SOL / 1.073B tokens ≈ 0.00003 SOL per token
pub const INITIAL_VIRTUAL_SOL: u64 = 30_000_000_000;       // 30 SOL in lamports
pub const INITIAL_VIRTUAL_TOKEN: u64 = 1_073_000_000_000_000; // ~1.073B tokens

// ─── Graduation ───────────────────────────────────────────────────────────────
pub const GRADUATION_SOL_THRESHOLD: u64 = 85_000_000_000;  // 85 SOL in lamports

// ─── Fees ─────────────────────────────────────────────────────────────────────
pub const TOTAL_FEE_BPS: u64 = 100;        // 1 %   of every trade's SOL volume
pub const PLATFORM_SHARE_BPS: u64 = 75;   // 0.75% → platform wallet (immediate)
pub const CREATOR_SHARE_BPS: u64 = 25;    // 0.25% → fee_recipient PDA (claimable)
pub const MAX_FEE_BPS: u64 = 500;         // 5% max fee (hard cap)

// ─── Wallet 2 minimum reserve ─────────────────────────────────────────────────
pub const MIN_CREATOR_RESERVE: u64 = 5_000_000; // 0.005 SOL — gas for future claims

// ─── Trade Limits ─────────────────────────────────────────────────────────────
pub const MAX_TRADE_SOL: u64 = 10_000_000_000; // 10 SOL max trade size
pub const MAX_TRADE_TOKENS: u64 = 1_000_000_000_000_000; // 1T tokens max trade size

// ─── Price Impact ─────────────────────────────────────────────────────────────
pub const MAX_PRICE_IMPACT_BPS: u64 = 1000; // 10% max price impact
pub const DEFAULT_PRICE_IMPACT_BPS: u64 = 500; // 5% default

// ─── Time-based constants ─────────────────────────────────────────────────────
pub const DAY_SECONDS: i64 = 86_400;
pub const WEEK_SECONDS: i64 = 604_800;
pub const MONTH_SECONDS: i64 = 2_592_000;

// ─── Rate Limiting ────────────────────────────────────────────────────────────
pub const RATE_LIMIT_WINDOW_SECONDS: i64 = 3600; // 1 hour
pub const MAX_TRADES_PER_WINDOW: u64 = 100;
pub const MAX_VOLUME_PER_WINDOW: u64 = 100_000_000_000; // 100 SOL

// ─── Whale Protection ────────────────────────────────────────────────────────
pub const WHALE_THRESHOLD: u64 = 1_000_000_000_000; // 1000 SOL
pub const SUPER_WHALE_THRESHOLD: u64 = 5_000_000_000_000; // 5000 SOL

// ─── Bootstrap Period ────────────────────────────────────────────────────────
pub const BOOTSTRAP_DURATION_SECONDS: u64 = 604_800; // 7 days

// ─── Staking ──────────────────────────────────────────────────────────────────
pub const REWARD_PRECISION: u128 = 1_000_000_000_000;

// ─── Whitelist ────────────────────────────────────────────────────────────────
pub const WHITELIST_MAX_SOL_PER_WALLET: u64 = 50_000_000_000; // 50 SOL

// ─── Referral ─────────────────────────────────────────────────────────────────
pub const MAX_REFERRAL_SHARE_BPS: u16 = 10_000; // 100% of creator's share

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
pub const SEED_STAKER_VAULT: &[u8] = b"staker_vault";
pub const SEED_RATE_LIMITER: &[u8] = b"rate_limiter";
pub const SEED_REENTRANCY_GUARD: &[u8] = b"reentrancy_guard";
pub const SEED_CIRCUIT_BREAKER: &[u8] = b"circuit_breaker";
pub const SEED_ADDRESS_FILTER: &[u8] = b"address_filter";
pub const SEED_FLASH_LOAN_DETECTOR: &[u8] = b"flash_loan_detector";
pub const SEED_METRICS: &[u8] = b"metrics";
pub const SEED_POOL_HEALTH: &[u8] = b"pool_health";
pub const SEED_USER_STATS: &[u8] = b"user_stats";
pub const SEED_DYNAMIC_FEE: &[u8] = b"dynamic_fee";
pub const SEED_BOOTSTRAP: &[u8] = b"bootstrap";
pub const SEED_WHALE_PROTECTION: &[u8] = b"whale_protection";
pub const SEED_FEE_REDISTRIBUTION: &[u8] = b"fee_redistribution";
pub const SEED_UPGRADEABLE_PROGRAM: &[u8] = b"upgradeable_program";
pub const SEED_FEATURE_FLAGS: &[u8] = b"feature_flags";
pub const SEED_PLUGIN: &[u8] = b"plugin";
pub const SEED_UPGRADE_PATH: &[u8] = b"upgrade_path";
pub const SEED_COMPLIANCE: &[u8] = b"compliance";
pub const SEED_AUDIT_LOG: &[u8] = b"audit_log";
pub const SEED_REFERRAL_CONFIG: &[u8] = b"referral_config";
pub const SEED_REFERRAL_RECORD: &[u8] = b"referral_record";

// ─── External Programs ────────────────────────────────────────────────────────
pub const METAPLEX_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
pub const METEORA_DAMM_PROGRAM_ID: &str  = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EkSCmrAP";
pub const METEORA_DLMM_PROGRAM_ID: &str  = "LBUZKhRxPF3XUpBCjp4YzTKgLLjggiJmV1fTTCkUscX";
pub const PUMP_SWAP_PROGRAM_ID: &str     = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
