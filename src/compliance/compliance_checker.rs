use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// ComplianceChecker - On-chain compliance verification
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct ComplianceChecker {
    pub mint: Pubkey,
    pub kyc_required: bool,
    pub jurisdiction_blacklist: [u8; 32], // Bloom filter for jurisdictions
    pub risk_level: u8, // 0-100
    pub last_kyc_check: i64,
    pub kyc_expiry: i64,
    pub bump: u8,
}

impl ComplianceChecker {
    pub const SEED: &'static [u8] = b"compliance";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 1   // kyc_required
        + 32  // jurisdiction_blacklist
        + 1   // risk_level
        + 8   // last_kyc_check
        + 8   // kyc_expiry
        + 1;  // bump

    /// Initialize compliance checker
    pub fn init(&mut self, mint: &Pubkey, kyc_required: bool, bump: u8) {
        self.mint = *mint;
        self.kyc_required = kyc_required;
        self.jurisdiction_blacklist = [0; 32];
        self.risk_level = 0;
        self.last_kyc_check = 0;
        self.kyc_expiry = 0;
        self.bump = bump;
    }

    /// Check if KYC is required and valid
    pub fn check_kyc(&self, kyc_valid_until: i64) -> Result<()> {
        if !self.kyc_required {
            return Ok(());
        }

        let now = Clock::get()?.unix_timestamp;
        require!(
            kyc_valid_until > now,
            BondingError::KycExpired
        );

        require!(
            now < self.kyc_expiry,
            BondingError::KycNotVerified
        );

        Ok(())
    }

    /// Check jurisdiction compliance
    pub fn check_jurisdiction(&self, jurisdiction_hash: &[u8; 32]) -> Result<()> {
        // Simple bloom filter check
        // In production, use a proper bloom filter implementation
        let mut match_found = false;
        for i in 0..32 {
            if jurisdiction_hash[i] == self.jurisdiction_blacklist[i] {
                match_found = true;
                break;
            }
        }

        require!(
            !match_found,
            BondingError::RestrictedJurisdiction
        );

        Ok(())
    }

    /// Update risk level
    pub fn update_risk_level(&mut self, new_level: u8) {
        self.risk_level = new_level;
    }

    /// Get risk category
    pub fn risk_category(&self) -> &str {
        match self.risk_level {
            0..=20 => "Low",
            21..=40 => "Medium-Low",
            41..=60 => "Medium",
            61..=80 => "Medium-High",
            _ => "High",
        }
    }

    /// Validate compliance for a trade
    pub fn validate_trade(
        &self,
        user_kyc_expiry: i64,
        jurisdiction_hash: &[u8; 32],
    ) -> Result<()> {
        self.check_kyc(user_kyc_expiry)?;
        self.check_jurisdiction(jurisdiction_hash)?;
        Ok(())
    }
}

/// Add compliance errors

