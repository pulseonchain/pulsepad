use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// ReentrancyGuard - Prevents reentrant calls in critical functions
// Seeds: [b"reentrancy_guard", mint]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct ReentrancyGuard {
    pub mint: Pubkey,
    pub locked: bool,
    pub bump: u8,
}

impl ReentrancyGuard {
    pub const SEED: &'static [u8] = b"reentrancy_guard";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 1   // locked
        + 1;  // bump (padding)

    /// Enter a critical section, fails if already locked
    pub fn enter(&mut self) -> Result<()> {
        require!(!self.locked, BondingError::ReentrancyDetected);
        self.locked = true;
        Ok(())
    }

    /// Exit a critical section
    pub fn exit(&mut self) {
        self.locked = false;
    }

    /// Try to enter without error (returns bool)
    pub fn try_enter(&mut self) -> bool {
        if self.locked {
            return false;
        }
        self.locked = true;
        true
    }

    /// Check if currently locked
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

/// Error code for reentrancy detection

