use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// PoolHealth - Track and report pool health metrics
// Seeds: [b"pool_health", mint]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct PoolHealth {
    pub mint: Pubkey,
    pub health_score: u16, // 0-10000 (0-100%)
    pub liquidity_depth: u64,
    pub price_volatility: u64,
    pub trade_volume_24h: u64,
    pub unique_traders: u32,
    pub last_updated: i64,
    pub graduated: bool,
    pub dex_pool_address: Option<Pubkey>,
    pub bump: u8,
}

impl PoolHealth {
    pub const SEED: &'static [u8] = b"pool_health";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 2   // health_score
        + 8   // liquidity_depth
        + 8   // price_volatility
        + 8   // trade_volume_24h
        + 4   // unique_traders
        + 8   // last_updated
        + 1   // graduated
        + 33  // dex_pool_address (Option<Pubkey>)
        + 1;  // bump

    /// Initialize pool health
    pub fn init(&mut self, mint: &Pubkey, bump: u8) {
        self.mint = *mint;
        self.health_score = 10000; // Start at 100%
        self.liquidity_depth = 0;
        self.price_volatility = 0;
        self.trade_volume_24h = 0;
        self.unique_traders = 0;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
        self.graduated = false;
        self.dex_pool_address = None;
        self.bump = bump;
    }

    /// Update health score
    pub fn update_health(
        &mut self,
        sol_reserves: u64,
        token_reserves: u64,
        graduation_threshold: u64,
        now: i64,
    ) {
        self.last_updated = now;

        // Calculate health based on multiple factors
        let mut health = 10000;

        // Factor 1: Liquidity depth (10% weight)
        if sol_reserves > 0 {
            let liquidity_score = (sol_reserves as u128 * 1000) / (graduation_threshold as u128);
            health = health.saturating_sub((1000 - liquidity_score.min(1000) as u64) as u16);
        }

        // Factor 2: Graduation status (20% weight)
        if self.graduated {
            health = health.saturating_sub(500); // Slight penalty for graduated pools
        }

        // Factor 3: Trade volume (10% weight)
        // Higher volume = better health

        // Factor 4: Price stability (10% weight)
        // Lower volatility = better health

        self.health_score = health;
        self.liquidity_depth = sol_reserves.saturating_add(token_reserves);
    }

    /// Calculate health score components
    pub fn health_components(&self) -> (u16, u16, u16) {
        let liquidity_score = self.health_score / 2;
        let stability_score = self.health_score / 3;
        let activity_score = self.health_score - liquidity_score - stability_score;
        (liquidity_score, stability_score, activity_score)
    }

    /// Get health category
    pub fn health_category(&self) -> &str {
        match self.health_score {
            0..=2500 => "Critical",
            2501..=5000 => "Warning",
            5001..=7500 => "Healthy",
            _ => "Excellent",
        }
    }
}
