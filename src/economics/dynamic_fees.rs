use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// DynamicFees - Adjust fees based on market conditions
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct DynamicFeeConfig {
    pub mint: Pubkey,
    pub base_fee_bps: u64,
    pub volatility_fee_multiplier: u64, // 1000 = 1x, 2000 = 2x
    pub liquidity_fee_multiplier: u64,  // 1000 = 1x, 2000 = 2x
    pub current_fee_bps: u64,
    pub last_update: i64,
    pub bump: u8,
}

impl DynamicFeeConfig {
    pub const SEED: &'static [u8] = b"dynamic_fee";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // base_fee_bps
        + 8   // volatility_fee_multiplier
        + 8   // liquidity_fee_multiplier
        + 8   // current_fee_bps
        + 8   // last_update
        + 1;  // bump

    /// Initialize dynamic fee config
    pub fn init(&mut self, mint: &Pubkey, base_fee_bps: u64, bump: u8) {
        self.mint = *mint;
        self.base_fee_bps = base_fee_bps;
        self.volatility_fee_multiplier = 1000;
        self.liquidity_fee_multiplier = 1000;
        self.current_fee_bps = base_fee_bps;
        self.last_update = Clock::get().unwrap().unix_timestamp;
        self.bump = bump;
    }

    /// Calculate dynamic fees based on market conditions
    pub fn calculate_fee(
        &mut self,
        sol_reserves: u64,
        token_reserves: u64,
        recent_volatility_bps: u64,
        now: i64,
    ) -> u64 {
        // Update multipliers if enough time has passed
        if now >= self.last_update + 3600 { // Update every hour
            self.update_multipliers(sol_reserves, token_reserves, recent_volatility_bps);
            self.last_update = now;
        }

        // Calculate effective fee
        let volatility_multiplier = self.volatility_fee_multiplier.max(1000);
        let liquidity_multiplier = self.liquidity_fee_multiplier.max(1000);
        
        let dynamic_fee = self.base_fee_bps
            .saturating_mul(volatility_multiplier)
            .saturating_div(1000)
            .saturating_mul(liquidity_multiplier)
            .saturating_div(1000);

        self.current_fee_bps = dynamic_fee;
        dynamic_fee
    }

    /// Update volatility multiplier based on recent price action
    pub fn update_volatility_multiplier(&mut self, volatility_bps: u64) {
        // Higher volatility = higher fees to discourage rapid trading
        // Volatility 0-100 bps -> multiplier 1000-1500
        self.volatility_fee_multiplier = 1000 + (volatility_bps / 20).min(500);
    }

    /// Update liquidity multiplier based on pool depth
    pub fn update_liquidity_multiplier(&mut self, sol_reserves: u64, threshold: u64) {
        // Lower liquidity = higher fees to preserve capital
        if sol_reserves < threshold / 2 {
            self.liquidity_fee_multiplier = 2000; // Double fees
        } else if sol_reserves < threshold {
            self.liquidity_fee_multiplier = 1500; // 1.5x fees
        } else {
            self.liquidity_fee_multiplier = 1000; // Normal fees
        }
    }

    /// Calculate effective fee multiplier
    pub fn effective_multiplier(&self) -> u64 {
        self.volatility_fee_multiplier
            .saturating_mul(self.liquidity_fee_multiplier)
            .saturating_div(1000)
    }

    /// Reset to base fees
    pub fn reset_to_base(&mut self) {
        self.volatility_fee_multiplier = 1000;
        self.liquidity_fee_multiplier = 1000;
        self.current_fee_bps = self.base_fee_bps;
    }
}

/// Add dynamic fee errors

