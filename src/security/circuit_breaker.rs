use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// CircuitBreaker - Emergency pause with auto-recovery
// Seeds: [b"circuit_breaker"]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct CircuitBreaker {
    pub is_paused: bool,
    pub pause_start: i64,
    pub pause_duration_seconds: i64, // 0 = indefinite
    pub paused_by: Pubkey,
    pub bump: u8,
}

impl CircuitBreaker {
    pub const SEED: &'static [u8] = b"circuit_breaker";

    pub const ACCOUNT_SIZE: usize = 8
        + 1   // is_paused
        + 8   // pause_start
        + 8   // pause_duration_seconds
        + 32  // paused_by
        + 1;  // bump

    /// Pause the protocol
    pub fn pause(&mut self, authority: Pubkey) -> Result<()> {
        require!(!self.is_paused, BondingError::AlreadyPaused);
        self.is_paused = true;
        self.pause_start = Clock::get()?.unix_timestamp;
        self.paused_by = authority;
        Ok(())
    }

    /// Unpause the protocol
    pub fn unpause(&mut self, authority: Pubkey) -> Result<()> {
        require!(self.is_paused, BondingError::NotPaused);
        require!(authority == self.paused_by, BondingError::Unauthorized);
        self.is_paused = false;
        Ok(())
    }

    /// Check if should auto-resume
    pub fn should_auto_resume(&self) -> bool {
        if !self.is_paused { return false; }
        if self.pause_duration_seconds == 0 { return false; } // indefinite pause
        
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.pause_start + self.pause_duration_seconds
    }

    /// Get remaining pause time
    pub fn remaining_pause_time(&self) -> i64 {
        if !self.is_paused { return 0; }
        let now = Clock::get().unwrap().unix_timestamp;
        let end_time = self.pause_start + self.pause_duration_seconds;
        end_time.saturating_sub(now)
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
}

/// Add already_paused and not_paused errors

