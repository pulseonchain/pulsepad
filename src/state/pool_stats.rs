use anchor_lang::prelude::*;

// ─── Pool Statistics ──────────────────────────────────────────────────────────
//
// A dedicated analytics PDA for each pool. Decoupled from PoolState so that
// the hot trading path (buy/sell) doesn't bloat the core account with counters
// that indexers care about but the program doesn't need for pricing logic.
//
// Seeds: [b"pool_stats", mint]
//
// Updated by: buy, sell, migrate, stake, unstake, claim_staker_rewards
// Read by: SDK analytics, frontends, ranking systems
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct PoolStats {
    pub mint: Pubkey,

    // ── Trade Counts ──────────────────────────────────────────────────────────
    /// Total number of buy transactions
    pub total_buys: u64,
    /// Total number of sell transactions
    pub total_sells: u64,
    /// Number of unique wallets that have ever bought (approximation via bloom
    /// filter or exact counter — using exact counter here for simplicity)
    pub unique_buyers: u64,

    // ── Volume ────────────────────────────────────────────────────────────────
    /// Cumulative SOL volume spent on buys (lamports)
    pub total_sol_volume_buy: u64,
    /// Cumulative SOL volume received from sells (lamports)
    pub total_sol_volume_sell: u64,
    /// Cumulative tokens bought (raw units)
    pub total_tokens_bought: u64,
    /// Cumulative tokens sold (raw units)
    pub total_tokens_sold: u64,

    // ── Fee Tracking ──────────────────────────────────────────────────────────
    /// Total platform fees collected in lamports (all-time)
    pub total_platform_fees_collected: u64,
    /// Total creator fees collected in lamports (all-time)
    pub total_creator_fees_collected: u64,
    /// Total LP fees claimed post-graduation (lamports)
    pub total_lp_fees_claimed: u64,
    /// Total staker rewards distributed (lamports)
    pub total_staker_rewards_distributed: u64,

    // ── Peak / ATH Tracking ───────────────────────────────────────────────────
    /// Peak real_sol_reserves ever reached (lamports) — helpful for ATH price computation
    pub peak_sol_reserves: u64,
    /// Unix timestamp when peak_sol_reserves was reached
    pub peak_sol_timestamp: i64,

    // ── Migration ─────────────────────────────────────────────────────────────
    /// Unix timestamp of graduation (0 if not yet graduated)
    pub graduated_at: i64,
    /// SOL deposited to DEX at migration (lamports)
    pub sol_at_graduation: u64,
    /// Tokens burned at graduation (raw units)
    pub tokens_burned_at_graduation: u64,

    // ── Staking ───────────────────────────────────────────────────────────────
    /// Number of wallets currently staking (net stake/unstake counter)
    pub total_stakers: u32,

    // ── Padding for future fields ──────────────────────────────────────────────
    pub bump: u8,
    pub _padding: [u8; 3],
}

pub const SEED_POOL_STATS: &[u8] = b"pool_stats";

impl PoolStats {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // total_buys
        + 8   // total_sells
        + 8   // unique_buyers
        + 8   // total_sol_volume_buy
        + 8   // total_sol_volume_sell
        + 8   // total_tokens_bought
        + 8   // total_tokens_sold
        + 8   // total_platform_fees_collected
        + 8   // total_creator_fees_collected
        + 8   // total_lp_fees_claimed
        + 8   // total_staker_rewards_distributed
        + 8   // peak_sol_reserves
        + 8   // peak_sol_timestamp
        + 8   // graduated_at
        + 8   // sol_at_graduation
        + 8   // tokens_burned_at_graduation
        + 4   // total_stakers
        + 1   // bump
        + 3;  // padding

    pub fn init(&mut self, mint: Pubkey, bump: u8) {
        self.mint = mint;
        self.total_buys = 0;
        self.total_sells = 0;
        self.unique_buyers = 0;
        self.total_sol_volume_buy = 0;
        self.total_sol_volume_sell = 0;
        self.total_tokens_bought = 0;
        self.total_tokens_sold = 0;
        self.total_platform_fees_collected = 0;
        self.total_creator_fees_collected = 0;
        self.total_lp_fees_claimed = 0;
        self.total_staker_rewards_distributed = 0;
        self.peak_sol_reserves = 0;
        self.peak_sol_timestamp = 0;
        self.graduated_at = 0;
        self.sol_at_graduation = 0;
        self.tokens_burned_at_graduation = 0;
        self.total_stakers = 0;
        self.bump = bump;
        self._padding = [0; 3];
    }

    /// Called after every buy to update stats.
    pub fn record_buy(
        &mut self,
        sol_amount: u64,
        tokens_out: u64,
        platform_fee: u64,
        creator_fee: u64,
        current_sol_reserves: u64,
        now: i64,
    ) {
        self.total_buys = self.total_buys.saturating_add(1);
        self.total_sol_volume_buy = self.total_sol_volume_buy.saturating_add(sol_amount);
        self.total_tokens_bought = self.total_tokens_bought.saturating_add(tokens_out);
        self.total_platform_fees_collected = self.total_platform_fees_collected.saturating_add(platform_fee);
        self.total_creator_fees_collected = self.total_creator_fees_collected.saturating_add(creator_fee);

        if current_sol_reserves > self.peak_sol_reserves {
            self.peak_sol_reserves = current_sol_reserves;
            self.peak_sol_timestamp = now;
        }
    }

    /// Called after every sell to update stats.
    pub fn record_sell(
        &mut self,
        sol_out_gross: u64,
        tokens_in: u64,
        platform_fee: u64,
        creator_fee: u64,
    ) {
        self.total_sells = self.total_sells.saturating_add(1);
        self.total_sol_volume_sell = self.total_sol_volume_sell.saturating_add(sol_out_gross);
        self.total_tokens_sold = self.total_tokens_sold.saturating_add(tokens_in);
        self.total_platform_fees_collected = self.total_platform_fees_collected.saturating_add(platform_fee);
        self.total_creator_fees_collected = self.total_creator_fees_collected.saturating_add(creator_fee);
    }

    /// Called at migration to record graduation data.
    pub fn record_graduation(
        &mut self,
        sol_deposited: u64,
        tokens_burned: u64,
        now: i64,
    ) {
        self.graduated_at = now;
        self.sol_at_graduation = sol_deposited;
        self.tokens_burned_at_graduation = tokens_burned;
    }

    /// Returns total trade count (buys + sells).
    pub fn total_trades(&self) -> u64 {
        self.total_buys.saturating_add(self.total_sells)
    }

    /// Returns net SOL volume (buy - sell) in lamports.
    pub fn net_sol_volume(&self) -> i64 {
        self.total_sol_volume_buy as i64 - self.total_sol_volume_sell as i64
    }
}
