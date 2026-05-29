use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// MetricsTracker - Track protocol metrics in real-time
// Seeds: [b"metrics", mint]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct Metrics {
    pub mint: Pubkey,
    pub total_buys: u64,
    pub total_sells: u64,
    pub total_volume_sol: u64,
    pub total_volume_tokens: u64,
    pub total_fees_platform: u64,
    pub total_fees_creator: u64,
    pub unique_buyers: u64,
    pub unique_sellers: u64,
    pub last_trade_timestamp: i64,
    pub peak_sol_reserves: u64,
    pub peak_sol_timestamp: i64,
    pub total_migrations: u32,
    pub total_stakers: u32,
    pub total_lp_fees_claimed: u64,
    pub total_staker_rewards: u64,
    pub bump: u8,
}

impl Metrics {
    pub const SEED: &'static [u8] = b"metrics";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // total_buys
        + 8   // total_sells
        + 8   // total_volume_sol
        + 8   // total_volume_tokens
        + 8   // total_fees_platform
        + 8   // total_fees_creator
        + 8   // unique_buyers
        + 8   // unique_sellers
        + 8   // last_trade_timestamp
        + 8   // peak_sol_reserves
        + 8   // peak_sol_timestamp
        + 4   // total_migrations
        + 4   // total_stakers
        + 8   // total_lp_fees_claimed
        + 8   // total_staker_rewards
        + 1;  // bump

    /// Initialize metrics
    pub fn init(&mut self, mint: &Pubkey, bump: u8) {
        self.mint = *mint;
        self.total_buys = 0;
        self.total_sells = 0;
        self.total_volume_sol = 0;
        self.total_volume_tokens = 0;
        self.total_fees_platform = 0;
        self.total_fees_creator = 0;
        self.unique_buyers = 0;
        self.unique_sellers = 0;
        self.last_trade_timestamp = 0;
        self.peak_sol_reserves = 0;
        self.peak_sol_timestamp = 0;
        self.total_migrations = 0;
        self.total_stakers = 0;
        self.total_lp_fees_claimed = 0;
        self.total_staker_rewards = 0;
        self.bump = bump;
    }

    /// Record a buy trade
    pub fn record_buy(
        &mut self,
        volume_sol: u64,
        volume_tokens: u64,
        fees_platform: u64,
        fees_creator: u64,
        current_sol_reserves: u64,
        buyer: &Pubkey,
        now: i64,
    ) {
        self.total_buys = self.total_buys.saturating_add(1);
        self.total_volume_sol = self.total_volume_sol.saturating_add(volume_sol);
        self.total_volume_tokens = self.total_volume_tokens.saturating_add(volume_tokens);
        self.total_fees_platform = self.total_fees_platform.saturating_add(fees_platform);
        self.total_fees_creator = self.total_fees_creator.saturating_add(fees_creator);
        self.last_trade_timestamp = now;

        if current_sol_reserves > self.peak_sol_reserves {
            self.peak_sol_reserves = current_sol_reserves;
            self.peak_sol_timestamp = now;
        }

        // Track unique buyer (simplified - in production use bloom filter)
        self.unique_buyers = self.unique_buyers.saturating_add(1);
    }

    /// Record a sell trade
    pub fn record_sell(
        &mut self,
        volume_sol: u64,
        volume_tokens: u64,
        fees_platform: u64,
        fees_creator: u64,
        seller: &Pubkey,
    ) {
        self.total_sells = self.total_sells.saturating_add(1);
        self.total_volume_sol = self.total_volume_sol.saturating_add(volume_sol);
        self.total_volume_tokens = self.total_volume_tokens.saturating_add(volume_tokens);
        self.total_fees_platform = self.total_fees_platform.saturating_add(fees_platform);
        self.total_fees_creator = self.total_fees_creator.saturating_add(fees_creator);
        self.unique_sellers = self.unique_sellers.saturating_add(1);
    }

    /// Record migration
    pub fn record_migration(&mut self) {
        self.total_migrations = self.total_migrations.saturating_add(1);
    }

    /// Record staker
    pub fn record_staker(&mut self) {
        self.total_stakers = self.total_stakers.saturating_add(1);
    }

    /// Record LP fee claim
    pub fn record_lp_fee_claim(&mut self, amount: u64) {
        self.total_lp_fees_claimed = self.total_lp_fees_claimed.saturating_add(amount);
    }

    /// Record staker reward
    pub fn record_staker_reward(&mut self, amount: u64) {
        self.total_staker_rewards = self.total_staker_rewards.saturating_add(amount);
    }

    /// Get total volume
    pub fn total_volume(&self) -> u64 {
        self.total_volume_sol.saturating_add(self.total_volume_tokens)
    }

    /// Get total fees
    pub fn total_fees(&self) -> u64 {
        self.total_fees_platform.saturating_add(self.total_fees_creator)
    }

    /// Get net volume (buys - sells)
    pub fn net_volume(&self) -> i64 {
        self.total_volume_sol as i64 - self.total_volume_tokens as i64
    }
}
