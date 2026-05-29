use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// RateLimiter - Prevents spam/abuse by tracking user activity
// Seeds: [b"rate_limiter", user, mint]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct RateLimiter {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub window_start: i64,
    pub window_count: u64,
    pub window_volume: u64,
    pub daily_count: u64,
    pub daily_volume: u64,
    pub bump: u8,
}

impl RateLimiter {
    pub const SEED: &'static [u8] = b"rate_limiter";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // user
        + 32  // mint
        + 8   // window_start
        + 8   // window_count
        + 8   // window_volume
        + 8   // daily_count
        + 8   // daily_volume
        + 1;  // bump

    /// Initialize or reset rate limiter
    pub fn init(&mut self, user: Pubkey, mint: Pubkey, bump: u8) {
        self.user = user;
        self.mint = mint;
        self.window_start = Clock::get().unwrap().unix_timestamp;
        self.window_count = 0;
        self.window_volume = 0;
        self.daily_count = 0;
        self.daily_volume = 0;
        self.bump = bump;
    }

    /// Check if operation is allowed
    pub fn check_and_update(
        &mut self,
        volume: u64,
        window_seconds: i64,
        max_window_volume: u64,
        max_daily_volume: u64,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        
        // Reset window if expired
        if now >= self.window_start + window_seconds {
            self.window_start = now;
            self.window_count = 0;
            self.window_volume = 0;
        }

        // Check window limits
        require!(
            self.window_volume.saturating_add(volume) <= max_window_volume,
            BondingError::RateLimitExceeded
        );

        // Check daily limits
        require!(
            self.daily_volume.saturating_add(volume) <= max_daily_volume,
            BondingError::DailyLimitExceeded
        );

        // Update counters
        self.window_count = self.window_count.saturating_add(1);
        self.window_volume = self.window_volume.saturating_add(volume);
        self.daily_count = self.daily_count.saturating_add(1);
        self.daily_volume = self.daily_volume.saturating_add(volume);

        Ok(())
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self, max_window: u64, max_daily: u64) -> (u64, u64) {
        (
            max_window.saturating_sub(self.window_volume),
            max_daily.saturating_sub(self.daily_volume),
        )
    }
}

/// Add rate limit errors

