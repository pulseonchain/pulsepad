use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// LiquidityBootstrapping - Time-weighted pricing for early stages
// Prevents sniping in the early stages of a pool
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct LiquidityBootstrap {
    pub mint: Pubkey,
    pub start_time: i64,
    pub duration_seconds: u64,
    pub initial_virtual_sol: u64,
    pub current_virtual_sol: u64,
    pub phase: BootstrapPhase,
    pub bump: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum BootstrapPhase {
    EarlyStage,    // Phase 1: Reduced buying power
    GrowthStage,   // Phase 2: Gradual transition
    StableStage,   // Phase 3: Normal operation
}

impl LiquidityBootstrap {
    pub const SEED: &'static [u8] = b"bootstrap";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // start_time
        + 8   // duration_seconds
        + 8   // initial_virtual_sol
        + 8   // current_virtual_sol
        + 1   // phase
        + 1;  // bump

    /// Initialize bootstrap config
    pub fn init(&mut self, mint: &Pubkey, duration_seconds: u64, bump: u8) {
        self.mint = *mint;
        self.start_time = Clock::get().unwrap().unix_timestamp;
        self.duration_seconds = duration_seconds;
        self.initial_virtual_sol = 30_000_000_000; // 30 SOL
        self.current_virtual_sol = 30_000_000_000;
        self.phase = BootstrapPhase::EarlyStage;
        self.bump = bump;
    }

    /// Get current bootstrap phase
    pub fn get_phase(&self, now: i64) -> BootstrapPhase {
        let elapsed = now.saturating_sub(self.start_time);
        
        if elapsed >= self.duration_seconds as i64 {
            BootstrapPhase::StableStage
        } else if elapsed >= (self.duration_seconds as i64 / 3) * 2 {
            BootstrapPhase::GrowthStage
        } else {
            BootstrapPhase::EarlyStage
        }
    }

    /// Get virtual SOL for pricing in current phase
    pub fn get_pricing_virtual_sol(&self, now: i64) -> u64 {
        let phase = self.get_phase(now);
        
        match phase {
            BootstrapPhase::EarlyStage => {
                // Phase 1: Only 20% of SOL can be used for pricing
                // This prevents large buy-ins from dominating early
                (self.current_virtual_sol as u128 * 20 / 100) as u64
            }
            BootstrapPhase::GrowthStage => {
                // Phase 2: Gradual transition to full pricing
                let elapsed = now.saturating_sub(self.start_time) as u64;
                let progress = (elapsed * 10000 / self.duration_seconds).min(10000);
                
                let early_weight = (10000 - progress) * self.initial_virtual_sol as u128;
                let full_weight = progress * self.current_virtual_sol as u128;
                
                ((early_weight + full_weight) / 10000) as u64
            }
            BootstrapPhase::StableStage => {
                self.current_virtual_sol
            }
        }
    }

    /// Check if early stage penalty applies
    pub fn is_early_stage_penalty(&self, now: i64) -> bool {
        self.get_phase(now) == BootstrapPhase::EarlyStage
    }

    /// Calculate early stage buy limit
    pub fn get_buy_limit(&self, now: i64) -> u64 {
        if !self.is_early_stage_penalty(now) {
            return u64::MAX;
        }
        
        // In early stage, limit buys to 1% of current virtual reserves
        self.current_virtual_sol / 100
    }

    /// Update virtual SOL (call after trades)
    pub fn update_virtual_sol(&mut self, new_sol: u64) {
        self.current_virtual_sol = new_sol;
    }

    /// Check if bootstrap period is complete
    pub fn is_complete(&self, now: i64) -> bool {
        now >= self.start_time + self.duration_seconds as i64
    }

    /// Get remaining bootstrap time
    pub fn remaining_time(&self, now: i64) -> i64 {
        (self.start_time + self.duration_seconds as i64).saturating_sub(now)
    }
}
