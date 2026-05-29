use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// UpgradeableProgram - Supports program upgrades with data separation
// Uses a proxy pattern where the actual program data is stored in a separate account
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct UpgradeableProgram {
    pub mint: Pubkey,
    pub program_authority: Pubkey,
    pub current_version: u32,
    pub next_version: Option<u32>,
    pub upgrade_delay_seconds: u64,
    pub upgrade_start_time: Option<i64>,
    pub implementation_account: Pubkey,
    pub bump: u8,
}

impl UpgradeableProgram {
    pub const SEED: &'static [u8] = b"upgradeable_program";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 32  // program_authority
        + 4   // current_version
        + 4   // next_version (Option<u32>)
        + 8   // upgrade_delay_seconds
        + 8   // upgrade_start_time (Option<i64>)
        + 32  // implementation_account
        + 1;  // bump

    /// Initialize upgradeable program
    pub fn init(
        &mut self,
        mint: &Pubkey,
        program_authority: &Pubkey,
        bump: u8,
    ) {
        self.mint = *mint;
        self.program_authority = *program_authority;
        self.current_version = 1;
        self.next_version = None;
        self.upgrade_delay_seconds = 0;
        self.upgrade_start_time = None;
        self.implementation_account = Pubkey::default();
        self.bump = bump;
    }

    /// Start an upgrade
    pub fn start_upgrade(&mut self, next_version: u32, delay_seconds: u64) -> Result<()> {
        require!(
            self.program_authority == Pubkey::default() || // First init
            self.next_version.is_none(), // No pending upgrade
            BondingError::UpgradeInProgress
        );

        self.next_version = Some(next_version);
        self.upgrade_delay_seconds = delay_seconds;
        self.upgrade_start_time = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    /// Complete an upgrade if delay has passed
    pub fn complete_upgrade(&mut self) -> Result<()> {
        let start_time = self.upgrade_start_time.ok_or(BondingError::NoPendingUpgrade)?;
        let now = Clock::get()?.unix_timestamp;
        let elapsed = now.saturating_sub(start_time);

        require!(
            elapsed >= self.upgrade_delay_seconds as i64,
            BondingError::UpgradeDelayNotMet
        );

        self.current_version = self.next_version.ok_or(BondingError::NoPendingUpgrade)?;
        self.next_version = None;
        self.upgrade_start_time = None;

        Ok(())
    }

    /// Check if upgrade is ready to complete
    pub fn upgrade_ready(&self) -> bool {
        match self.upgrade_start_time {
            Some(start) => {
                let now = Clock::get().unwrap().unix_timestamp;
                now >= start + self.upgrade_delay_seconds as i64
            }
            None => false,
        }
    }

    /// Schedule upgrade
    pub fn schedule_upgrade(&mut self, delay_seconds: u64) -> Result<()> {
        require!(
            delay_seconds >= 0,
            BondingError::InvalidUpgradeDelay
        );
        self.upgrade_delay_seconds = delay_seconds;
        self.upgrade_start_time = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    /// Cancel pending upgrade
    pub fn cancel_upgrade(&mut self) -> Result<()> {
        require!(
            self.next_version.is_some(),
            BondingError::NoPendingUpgrade
        );
        self.next_version = None;
        self.upgrade_start_time = None;
        Ok(())
    }
}

/// Add upgrade errors

