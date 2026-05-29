use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// UserStats - Track individual user trading activity
// Seeds: [b"user_stats", user, mint]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct UserStats {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub total_buys: u64,
    pub total_sells: u64,
    pub total_volume_bought: u64,
    pub total_volume_sold: u64,
    pub first_trade_timestamp: i64,
    pub last_trade_timestamp: i64,
    pub total_fees_paid: u64,
    pub net_profit: i64,
    pub total_trades: u64,
    pub stake_count: u32,
    pub referral_count: u32,
    pub bump: u8,
}

impl UserStats {
    pub const SEED: &'static [u8] = b"user_stats";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // user
        + 32  // mint
        + 8   // total_buys
        + 8   // total_sells
        + 8   // total_volume_bought
        + 8   // total_volume_sold
        + 8   // first_trade_timestamp
        + 8   // last_trade_timestamp
        + 8   // total_fees_paid
        + 8   // net_profit
        + 8   // total_trades
        + 4   // stake_count
        + 4   // referral_count
        + 1;  // bump

    /// Initialize user stats
    pub fn init(&mut self, user: &Pubkey, mint: &Pubkey, bump: u8) {
        self.user = *user;
        self.mint = *mint;
        self.total_buys = 0;
        self.total_sells = 0;
        self.total_volume_bought = 0;
        self.total_volume_sold = 0;
        self.first_trade_timestamp = Clock::get().unwrap().unix_timestamp;
        self.last_trade_timestamp = 0;
        self.total_fees_paid = 0;
        self.net_profit = 0;
        self.total_trades = 0;
        self.stake_count = 0;
        self.referral_count = 0;
        self.bump = bump;
    }

    /// Record a buy
    pub fn record_buy(&mut self, volume: u64, fees: u64, now: i64) {
        self.total_buys = self.total_buys.saturating_add(1);
        self.total_volume_bought = self.total_volume_bought.saturating_add(volume);
        self.total_fees_paid = self.total_fees_paid.saturating_add(fees);
        self.last_trade_timestamp = now;
        self.total_trades = self.total_trades.saturating_add(1);
    }

    /// Record a sell
    pub fn record_sell(&mut self, volume: u64, fees: u64, now: i64) {
        self.total_sells = self.total_sells.saturating_add(1);
        self.total_volume_sold = self.total_volume_sold.saturating_add(volume);
        self.total_fees_paid = self.total_fees_paid.saturating_add(fees);
        self.last_trade_timestamp = now;
        self.total_trades = self.total_trades.saturating_add(1);

        // Calculate net profit (simplified)
        let gross_profit = self.total_volume_sold as i64 - self.total_volume_bought as i64;
        self.net_profit = gross_profit.saturating_sub(self.total_fees_paid as i64);
    }

    /// Record stake
    pub fn record_stake(&mut self) {
        self.stake_count = self.stake_count.saturating_add(1);
    }

    /// Record referral
    pub fn record_referral(&mut self) {
        self.referral_count = self.referral_count.saturating_add(1);
    }

    /// Check if user is active
    pub fn is_active(&self, now: i64, window_seconds: i64) -> bool {
        now >= self.last_trade_timestamp && now < self.last_trade_timestamp + window_seconds
    }

    /// Get trading streak
    pub fn trading_streak(&self, now: i64, day_seconds: i64) -> u64 {
        if now < self.last_trade_timestamp {
            return 0;
        }
        (now - self.last_trade_timestamp) / day_seconds
    }
}
