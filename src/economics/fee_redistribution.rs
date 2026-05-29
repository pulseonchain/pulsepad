use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// FeeRedistribution - Smart fee distribution based on pool state
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct FeeRedistributionConfig {
    pub mint: Pubkey,
    pub platform_share_bps: u64,
    pub creator_share_bps: u64,
    pub staker_share_bps: u64,
    pub liquidity_pool_share_bps: u64,
    pub reserve_share_bps: u64,
    pub last_distribution: i64,
    pub bump: u8,
}

impl FeeRedistributionConfig {
    pub const SEED: &'static [u8] = b"fee_redistribution";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // platform_share_bps
        + 8   // creator_share_bps
        + 8   // staker_share_bps
        + 8   // liquidity_pool_share_bps
        + 8   // reserve_share_bps
        + 8   // last_distribution
        + 1;  // bump

    /// Initialize fee redistribution config
    pub fn init(
        &mut self,
        mint: &Pubkey,
        platform_share_bps: u64,
        creator_share_bps: u64,
        staker_share_bps: u64,
        bump: u8,
    ) {
        self.mint = *mint;
        self.platform_share_bps = platform_share_bps;
        self.creator_share_bps = creator_share_bps;
        self.staker_share_bps = staker_share_bps;
        self.liquidity_pool_share_bps = 0;
        self.reserve_share_bps = 0;
        self.last_distribution = Clock::get().unwrap().unix_timestamp;
        self.bump = bump;
    }

    /// Distribute fees based on current pool state
    pub fn distribute_fees(
        &mut self,
        total_fees: u64,
        pool_state: &PoolState,
        total_staked: u64,
    ) -> (u64, u64, u64, u64, u64) {
        // Platform fee
        let platform_fee = total_fees
            .saturating_mul(self.platform_share_bps)
            .saturating_div(100);

        // Creator fee
        let creator_fee = total_fees
            .saturating_mul(self.creator_share_bps)
            .saturating_div(100);

        // Staker reward
        let staker_reward = total_fees
            .saturating_mul(self.staker_share_bps)
            .saturating_div(100);

        // Liquidity pool contribution (during bootstrap)
        let lp_contribution = if pool_state.real_sol_reserves < 85_000_000_000 {
            total_fees
                .saturating_mul(self.liquidity_pool_share_bps)
                .saturating_div(100)
        } else {
            0
        };

        // Reserve contribution
        let reserve_contribution = total_fees
            .saturating_mul(self.reserve_share_bps)
            .saturating_div(100);

        (platform_fee, creator_fee, staker_reward, lp_contribution, reserve_contribution)
    }

    /// Adjust shares based on pool health
    pub fn adjust_for_health(&mut self, health_score: u16) {
        // When pool is unhealthy, reduce staker rewards
        if health_score < 5000 {
            let reduction = 10000 - health_score; // 0-5000
            self.staker_share_bps = self.staker_share_bps.saturating_sub(reduction / 100);
            self.liquidity_pool_share_bps = self.liquidity_pool_share_bps.saturating_add(reduction / 100);
        }
    }

    /// Reset to default distribution
    pub fn reset_to_defaults(&mut self, platform: u64, creator: u64, staker: u64) {
        self.platform_share_bps = platform;
        self.creator_share_bps = creator;
        self.staker_share_bps = staker;
    }
}

/// Add fee redistribution errors

