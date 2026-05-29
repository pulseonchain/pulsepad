use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// WhaleProtection - Progressive fees for large trades to prevent manipulation
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct WhaleProtection {
    pub mint: Pubkey,
    pub baseline_fee_bps: u64,
    pub whale_threshold: u64,
    pub super_whale_threshold: u64,
    pub baseline_multiplier: u64,  // 1000 = 1x
    pub whale_multiplier: u64,     // 1500 = 1.5x
    pub super_whale_multiplier: u64, // 3000 = 3x
    pub bump: u8,
}

impl WhaleProtection {
    pub const SEED: &'static [u8] = b"whale_protection";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // baseline_fee_bps
        + 8   // whale_threshold
        + 8   // super_whale_threshold
        + 8   // baseline_multiplier
        + 8   // whale_multiplier
        + 8   // super_whale_multiplier
        + 1;  // bump

    /// Initialize whale protection
    pub fn init(
        &mut self,
        mint: &Pubkey,
        baseline_fee_bps: u64,
        whale_threshold: u64,
        super_whale_threshold: u64,
        bump: u8,
    ) {
        self.mint = *mint;
        self.baseline_fee_bps = baseline_fee_bps;
        self.whale_threshold = whale_threshold;
        self.super_whale_threshold = super_whale_threshold;
        self.baseline_multiplier = 1000;
        self.whale_multiplier = 1500;
        self.super_whale_multiplier = 3000;
        self.bump = bump;
    }

    /// Calculate effective fee for a trade
    pub fn calculate_effective_fee(&self, trade_volume: u64) -> u64 {
        let multiplier = self.get_multiplier(trade_volume);
        self.baseline_fee_bps
            .saturating_mul(multiplier)
            .saturating_div(1000)
    }

    /// Get the appropriate fee multiplier for a trade size
    pub fn get_multiplier(&self, trade_volume: u64) -> u64 {
        if trade_volume >= self.super_whale_threshold {
            self.super_whale_multiplier
        } else if trade_volume >= self.whale_threshold {
            self.whale_multiplier
        } else {
            self.baseline_multiplier
        }
    }

    /// Calculate progressive fee
    pub fn calculate_progressive_fee(
        &self,
        trade_volume: u64,
    ) -> (u64, u64) {
        // Returns (progressive_fee, total_fee)
        let multiplier = self.get_multiplier(trade_volume);
        let progressive_fee = self.baseline_fee_bps
            .saturating_mul(multiplier)
            .saturating_div(1000)
            .saturating_sub(self.baseline_fee_bps);
        
        let total_fee = self.baseline_fee_bps
            .saturating_mul(multiplier)
            .saturating_div(1000);
        
        (progressive_fee, total_fee)
    }

    /// Check if trade qualifies as whale
    pub fn is_whale_trade(&self, trade_volume: u64) -> bool {
        trade_volume >= self.whale_threshold
    }

    /// Check if trade qualifies as super whale
    pub fn is_super_whale_trade(&self, trade_volume: u64) -> bool {
        trade_volume >= self.super_whale_threshold
    }

    /// Get whale status for a trade
    pub fn get_whale_status(&self, trade_volume: u64) -> WhaleStatus {
        if trade_volume >= self.super_whale_threshold {
            WhaleStatus::SuperWhale
        } else if trade_volume >= self.whale_threshold {
            WhaleStatus::Whale
        } else {
            WhaleStatus::Normal
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WhaleStatus {
    Normal,
    Whale,
    SuperWhale,
}
