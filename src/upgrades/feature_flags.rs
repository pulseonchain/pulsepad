use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// FeatureFlags - Enable/disable features per pool
// Seeds: [b"feature_flags", mint]
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct FeatureFlags {
    pub mint: Pubkey,
    // Trading features
    pub buy_enabled: bool,
    pub sell_enabled: bool,
    pub whitelist_enabled: bool,
    pub referral_enabled: bool,
    
    // Staking features
    pub staking_enabled: bool,
    pub lp_fee_claims_enabled: bool,
    
    // Migration features
    pub migration_enabled: bool,
    pub auto_migration_enabled: bool,
    
    // Admin features
    pub fee_updates_enabled: bool,
    pub pause_enabled: bool,
    
    // Advanced features
    pub flash_loan_detection_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub dynamic_fees_enabled: bool,
    pub whale_protection_enabled: bool,
    
    pub bump: u8,
}

impl FeatureFlags {
    pub const SEED: &'static [u8] = b"feature_flags";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 1   // buy_enabled
        + 1   // sell_enabled
        + 1   // whitelist_enabled
        + 1   // referral_enabled
        + 1   // staking_enabled
        + 1   // lp_fee_claims_enabled
        + 1   // migration_enabled
        + 1   // auto_migration_enabled
        + 1   // fee_updates_enabled
        + 1   // pause_enabled
        + 1   // flash_loan_detection_enabled
        + 1   // rate_limiting_enabled
        + 1   // dynamic_fees_enabled
        + 1   // whale_protection_enabled
        + 1;  // bump (padding)

    /// Initialize feature flags with defaults
    pub fn init(&mut self, mint: &Pubkey, bump: u8) {
        self.mint = *mint;
        self.buy_enabled = true;
        self.sell_enabled = true;
        self.whitelist_enabled = false;
        self.referral_enabled = false;
        self.staking_enabled = true;
        self.lp_fee_claims_enabled = true;
        self.migration_enabled = true;
        self.auto_migration_enabled = true;
        self.fee_updates_enabled = true;
        self.pause_enabled = true;
        self.flash_loan_detection_enabled = true;
        self.rate_limiting_enabled = false;
        self.dynamic_fees_enabled = false;
        self.whale_protection_enabled = false;
        self.bump = bump;
    }

    /// Toggle a feature
    pub fn toggle_feature(&mut self, feature: Feature) -> Result<()> {
        match feature {
            Feature::Buy => self.buy_enabled = !self.buy_enabled,
            Feature::Sell => self.sell_enabled = !self.sell_enabled,
            Feature::Whitelist => self.whitelist_enabled = !self.whitelist_enabled,
            Feature::Referral => self.referral_enabled = !self.referral_enabled,
            Feature::Staking => self.staking_enabled = !self.staking_enabled,
            Feature::LpClaims => self.lp_fee_claims_enabled = !self.lp_fee_claims_enabled,
            Feature::Migration => self.migration_enabled = !self.migration_enabled,
            Feature::AutoMigration => self.auto_migration_enabled = !self.auto_migration_enabled,
            Feature::FeeUpdates => self.fee_updates_enabled = !self.fee_updates_enabled,
            Feature::Pause => self.pause_enabled = !self.pause_enabled,
            Feature::FlashLoanDetection => self.flash_loan_detection_enabled = !self.flash_loan_detection_enabled,
            Feature::RateLimiting => self.rate_limiting_enabled = !self.rate_limiting_enabled,
            Feature::DynamicFees => self.dynamic_fees_enabled = !self.dynamic_fees_enabled,
            Feature::WhaleProtection => self.whale_protection_enabled = !self.whale_protection_enabled,
        }
        Ok(())
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::Buy => self.buy_enabled,
            Feature::Sell => self.sell_enabled,
            Feature::Whitelist => self.whitelist_enabled,
            Feature::Referral => self.referral_enabled,
            Feature::Staking => self.staking_enabled,
            Feature::LpClaims => self.lp_fee_claims_enabled,
            Feature::Migration => self.migration_enabled,
            Feature::AutoMigration => self.auto_migration_enabled,
            Feature::FeeUpdates => self.fee_updates_enabled,
            Feature::Pause => self.pause_enabled,
            Feature::FlashLoanDetection => self.flash_loan_detection_enabled,
            Feature::RateLimiting => self.rate_limiting_enabled,
            Feature::DynamicFees => self.dynamic_fees_enabled,
            Feature::WhaleProtection => self.whale_protection_enabled,
        }
    }

    /// Get all features as a bitmap
    pub fn to_bitmap(&self) -> u128 {
        let mut bitmap: u128 = 0;
        
        if self.buy_enabled { bitmap |= 1 << 0; }
        if self.sell_enabled { bitmap |= 1 << 1; }
        if self.whitelist_enabled { bitmap |= 1 << 2; }
        if self.referral_enabled { bitmap |= 1 << 3; }
        if self.staking_enabled { bitmap |= 1 << 4; }
        if self.lp_fee_claims_enabled { bitmap |= 1 << 5; }
        if self.migration_enabled { bitmap |= 1 << 6; }
        if self.auto_migration_enabled { bitmap |= 1 << 7; }
        if self.fee_updates_enabled { bitmap |= 1 << 8; }
        if self.pause_enabled { bitmap |= 1 << 9; }
        if self.flash_loan_detection_enabled { bitmap |= 1 << 10; }
        if self.rate_limiting_enabled { bitmap |= 1 << 11; }
        if self.dynamic_fees_enabled { bitmap |= 1 << 12; }
        if self.whale_protection_enabled { bitmap |= 1 << 13; }
        
        bitmap
    }

    /// Set features from a bitmap
    pub fn from_bitmap(&mut self, bitmap: u128) {
        self.buy_enabled = (bitmap & (1 << 0)) != 0;
        self.sell_enabled = (bitmap & (1 << 1)) != 0;
        self.whitelist_enabled = (bitmap & (1 << 2)) != 0;
        self.referral_enabled = (bitmap & (1 << 3)) != 0;
        self.staking_enabled = (bitmap & (1 << 4)) != 0;
        self.lp_fee_claims_enabled = (bitmap & (1 << 5)) != 0;
        self.migration_enabled = (bitmap & (1 << 6)) != 0;
        self.auto_migration_enabled = (bitmap & (1 << 7)) != 0;
        self.fee_updates_enabled = (bitmap & (1 << 8)) != 0;
        self.pause_enabled = (bitmap & (1 << 9)) != 0;
        self.flash_loan_detection_enabled = (bitmap & (1 << 10)) != 0;
        self.rate_limiting_enabled = (bitmap & (1 << 11)) != 0;
        self.dynamic_fees_enabled = (bitmap & (1 << 12)) != 0;
        self.whale_protection_enabled = (bitmap & (1 << 13)) != 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Feature {
    Buy,
    Sell,
    Whitelist,
    Referral,
    Staking,
    LpClaims,
    Migration,
    AutoMigration,
    FeeUpdates,
    Pause,
    FlashLoanDetection,
    RateLimiting,
    DynamicFees,
    WhaleProtection,
}

/// Add feature flag errors

