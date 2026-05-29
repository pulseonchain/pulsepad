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

// ─── Reserve Tokens ──────────────────────────────────────────────────────────
// 97M tokens set aside to guarantee the last buyer always gets their full
// token amount even when the bonding curve is near graduation and the
// asymptotic CP price would otherwise make the buy fail.
// These sit in pool_token_account alongside bonding tokens but are only
// drawn down when real_token_reserves (bonding) runs out.
// At migration: half of remaining reserve is burned, half goes to migration_vault.

// ─── Graduation ───────────────────────────────────────────────────────────────
pub const GRADUATION_SOL_THRESHOLD: u64 = 85_000_000_000;  // 85 SOL in lamports

// ─── Fees ─────────────────────────────────────────────────────────────────────
pub const TOTAL_FEE_BPS: u64 = 100;        // 1 %   of every trade's SOL volume
pub const PLATFORM_SHARE_BPS: u64 = 75;   // 0.75% → platform wallet (immediate)
pub const CREATOR_SHARE_BPS: u64 = 25;    // 0.25% → fee_recipient PDA (claimable)
pub const MAX_FEE_BPS: u64 = 500;         // 5% max fee (hard cap)

// ─── Wallet 2 minimum reserve ─────────────────────────────────────────────────
pub const MIN_CREATOR_RESERVE: u64 = 5_000_000; // 0.005 SOL — gas for future claims
pub const MAX_TRADE_SOL: u64 = 10_000_000_000; // 10 SOL max trade size
pub const MAX_TRADE_TOKENS: u64 = 1_000_000_000_000_000; // 1T tokens max trade size

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

// ─── External Programs ────────────────────────────────────────────────────────
pub const METAPLEX_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
pub const METEORA_DAMM_PROGRAM_ID: &str  = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EkSCmrAP";
pub const METEORA_DLMM_PROGRAM_ID: &str  = "LBUZKhRxPF3XUpBCjp4YzTKgLLjggiJmV1fTTCkUscX";
pub const PUMP_SWAP_PROGRAM_ID: &str     = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
