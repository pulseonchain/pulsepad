use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// FlashLoanDetector - Detects flash loan arbitrage attempts
// ─────────────────────────────────────────────────────────────────────────────

/// Track transaction characteristics that indicate flash loan activity
#[account]
pub struct FlashLoanDetector {
    pub mint: Pubkey,
    pub block_timestamp: i64,
    pub trade_count: u32,
    pub total_volume: u64,
    pub is_suspicious: bool,
    pub bump: u8,
}

impl FlashLoanDetector {
    pub const SEED: &'static [u8] = b"flash_loan_detector";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // block_timestamp
        + 4   // trade_count
        + 8   // total_volume
        + 1   // is_suspicious
        + 1;  // bump

    /// Initialize flash loan detection for a pool
    pub fn init(&mut self, mint: &Pubkey, bump: u8) {
        self.mint = *mint;
        self.block_timestamp = Clock::get().unwrap().unix_timestamp;
        self.trade_count = 0;
        self.total_volume = 0;
        self.is_suspicious = false;
        self.bump = bump;
    }

    /// Record a trade and check for flash loan patterns
    pub fn record_trade(
        &mut self,
        volume: u64,
        time_window_seconds: i64,
        max_trades_per_window: u32,
        max_volume_per_window: u64,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        // Reset if outside time window
        if now >= self.block_timestamp + time_window_seconds {
            self.block_timestamp = now;
            self.trade_count = 0;
            self.total_volume = 0;
            self.is_suspicious = false;
        }

        // Update counters
        self.trade_count = self.trade_count.saturating_add(1);
        self.total_volume = self.total_volume.saturating_add(volume);

        // Check for suspicious patterns
        if self.trade_count > max_trades_per_window {
            self.is_suspicious = true;
        }
        if self.total_volume > max_volume_per_window {
            self.is_suspicious = true;
        }

        // Check for rapid consecutive trades
        if self.trade_count >= 2 {
            // Check if previous trade was very recent (potential flash loan)
            // In practice, you'd store previous trade timestamps
        }

        Ok(())
    }

    /// Check if current activity is suspicious
    pub fn is_suspicious(&self) -> bool {
        self.is_suspicious
    }

    /// Get current trade stats
    pub fn get_stats(&self) -> (u32, u64) {
        (self.trade_count, self.total_volume)
    }
}


