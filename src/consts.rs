// ─── Platform ────────────────────────────────────────────────────────────────
pub const PLATFORM_WALLET: &str = "EobUZD7H6TQRYfzqKsYEYekpKoFinKW1UWA4TsHidTqj";

// ─── Token Supply ─────────────────────────────────────────────────────────────
pub const TOKEN_DECIMALS: u8 = 6;
pub const TOTAL_SUPPLY: u64 = 1_097_052_391_304_347;   // ~1.097B × 10^6
pub const BONDING_SUPPLY: u64 = 700_000_000_000_000;   //   700 000 000 × 10^6
pub const RESERVE_SUPPLY: u64 =   97_052_391_304_347;  //    97 052 391 × 10^6
pub const LP_RESERVE_SUPPLY: u64 = 300_000_000_000_000; //   300 000 000 × 10^6

// ─── Bonding Curve (constant-product with virtual reserves) ──────────────────
pub const INITIAL_VIRTUAL_SOL: u64 = 30_000_000_000;       // 30 SOL in lamports
pub const INITIAL_VIRTUAL_TOKEN: u64 = 1_073_000_000_000_000; // ~1.073B tokens

// ─── Graduation Tiers (configurable per-pool) ─────────────────────────────────
// Creators pick ONE tier at pool creation. Threshold stored in PoolState.
// Tier names: Fast | Standard | Stable
// SOL:    80 SOL = ~$6.5K  |  150 SOL = ~$12.2K  |  240 SOL = ~$19.6K
// BNB:    15 BNB = ~$10.7K |  35 BNB  = ~$24.9K  |  50 BNB  = ~$35.6K
// ETH:    9 ETH  = ~$18K   |  16 ETH  = ~$32K    |  30 ETH  = ~$60K
pub const GRADUATION_TIER_FAST_SOL: u64     = 80_000_000_000;
pub const GRADUATION_TIER_STANDARD_SOL: u64 = 150_000_000_000;
pub const GRADUATION_TIER_STABLE_SOL: u64   = 240_000_000_000;

// Default threshold (backwards-compatible with existing pools)
pub const GRADUATION_SOL_THRESHOLD: u64 = 85_000_000_000;

// ─── Graduation Tier Enum ─────────────────────────────────────────────────────
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum GraduationTier {
    Fast,      // 80 SOL (cheapest, fastest path to DEX)
    Standard,  // 150 SOL (balanced)
    Stable,    // 240 SOL (deep liquidity, most price discovery)
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
        if threshold <= GRADUATION_TIER_FAST_SOL {
            GraduationTier::Fast
        } else if threshold <= GRADUATION_TIER_STANDARD_SOL {
            GraduationTier::Standard
        } else {
            GraduationTier::Stable
        }
    }
}

// ─── Anti-Snipe Protection ────────────────────────────────────────────────────
// First 3 minutes after pool init: virtual SOL is 3x higher.
// This means price = (3 * virtual_sol) / virtual_tokens → 3x more expensive.
// Snipers get wrecked. Normal buyers wait 3 minutes.
pub const ANTI_SNIPE_WINDOW_SECS: i64   = 180;     // 3 minutes
pub const ANTI_SNIPE_MULTIPLIER_BASIS: u64 = 300;   // 3.00x (300 basis points in hundredths)

// ─── Agent / Creator Vault Release ────────────────────────────────────────────
pub const MAX_VAULT_CLAIM_PER_24H: u64 = 500_000_000_000; // 500K tokens per 24h
pub const VAULT_CLAIM_COOLDOWN_SECS: i64 = 24 * 3600;      // 24 hours between claims

// ─── Partial Migration ────────────────────────────────────────────────────────
// Percentage of SOL/tokens kept in bonding curve as buyback fund.
// Must be 0 (full migration), 10, 20, or 30.
pub const PARTIAL_MIGRATION_OPTIONS: [u8; 4] = [0, 10, 20, 30];

// ─── Fees ─────────────────────────────────────────────────────────────────────
pub const TOTAL_FEE_BPS: u64 = 100;        // 1 %   (default)
pub const PLATFORM_SHARE_BPS: u64 = 75;   // 0.75% → platform wallet (immediate)
pub const CREATOR_SHARE_BPS: u64 = 25;    // 0.25% → fee_recipient PDA (claimable)
pub const MAX_FEE_BPS: u64 = 500;         // 5% max fee (hard cap)

// Platform always gets 3/4 of fees. Creator share is remainder.
// e.g. at 5% fee: platform = 3.75%, creator = 1.25%
pub const PLATFORM_FRACTION: u64 = 75;     // Always 3/4 of whatever the fee is

// ─── Fee Range for Creator Config ─────────────────────────────────────────────
pub const MIN_CREATOR_FEE_BPS: u64 = 100;  // 1% minimum
pub const MAX_CREATOR_FEE_BPS: u64 = 500;  // 5% maximum

// ─── Wallet minimum reserve ───────────────────────────────────────────────────
pub const MIN_CREATOR_RESERVE: u64 = 5_000_000; // 0.005 SOL — gas for future claims
pub const MAX_TRADE_SOL: u64 = 10_000_000_000; // 10 SOL max trade size
pub const MAX_TRADE_TOKENS: u64 = 1_000_000_000_000_000; // 1T tokens max trade size

// ─── Agent Wallet ─────────────────────────────────────────────────────────────
// Agent PDA seeds: [b"agent", mint]
pub const SEED_AGENT: &[u8] = b"agent";
pub const AGENT_NAME_PREFIX: &str = "Agent ";

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
pub const SEED_BUYBACK: &[u8]       = b"buyback";

// ─── External Programs ────────────────────────────────────────────────────────
pub const METAPLEX_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
pub const METEORA_DAMM_PROGRAM_ID: &str  = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EkSCmrAP";
pub const METEORA_DLMM_PROGRAM_ID: &str  = "LBUZKhRxPF3XUpBCjp4YzTKgLLjggiJmV1fTTCkUscX";
pub const PUMP_SWAP_PROGRAM_ID: &str     = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
