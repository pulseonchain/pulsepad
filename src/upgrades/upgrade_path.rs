use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// UpgradePath - Documents and manages future protocol upgrades
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct UpgradePath {
    pub current_version: u32,
    pub latest_version: u32,
    pub upgrade_schedule: [UpgradeScheduleEntry; 10],
    pub bump: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpgradeScheduleEntry {
    pub version: u32,
    pub scheduled_at: Option<i64>,
    pub deadline: Option<i64>,
    pub upgrade_delay: u64,
    pub description: [u8; 64], // UTF-8 encoded description
}

impl UpgradePath {
    pub const SEED: &'static [u8] = b"upgrade_path";

    pub const ACCOUNT_SIZE: usize = 8
        + 4   // current_version
        + 4   // latest_version
        + 112 * 10 // UpgradeScheduleEntry * 10
        + 1;  // bump

    /// Initialize upgrade path
    pub fn init(&mut self, bump: u8) {
        self.current_version = 1;
        self.latest_version = 1;
        self.bump = bump;
        
        // Initialize all entries as empty
        for entry in &mut self.upgrade_schedule {
            entry.version = 0;
            entry.scheduled_at = None;
            entry.deadline = None;
            entry.upgrade_delay = 0;
            entry.description = [0; 64];
        }
    }

    /// Schedule a future upgrade
    pub fn schedule_upgrade(
        &mut self,
        version: u32,
        scheduled_at: i64,
        deadline: i64,
        delay_seconds: u64,
        description: &[u8],
    ) -> Result<()> {
        require!(
            version > self.current_version,
            BondingError::InvalidUpgradeVersion
        );
        require!(
            scheduled_at < deadline,
            BondingError::InvalidUpgradeSchedule
        );
        require!(
            description.len() <= 64,
            BondingError::DescriptionTooLong
        );

        // Find empty slot
        for entry in &mut self.upgrade_schedule {
            if entry.version == 0 {
                entry.version = version;
                entry.scheduled_at = Some(scheduled_at);
                entry.deadline = Some(deadline);
                entry.upgrade_delay = delay_seconds;
                entry.description[..description.len()].copy_from_slice(description);
                break;
            }
        }

        if version > self.latest_version {
            self.latest_version = version;
        }

        Ok(())
    }

    /// Execute a scheduled upgrade
    pub fn execute_upgrade(&mut self, version: u32) -> Result<()> {
        let entry = self.upgrade_schedule.iter()
            .find(|e| e.version == version)
            .ok_or(BondingError::UpgradeNotFound)?;

        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= entry.deadline.unwrap_or(0),
            BondingError::UpgradeDeadlineNotMet
        );

        self.current_version = version;

        // Mark this entry as completed
        for entry in &mut self.upgrade_schedule {
            if entry.version == version {
                entry.scheduled_at = None;
                entry.deadline = None;
                break;
            }
        }

        Ok(())
    }

    /// Get current version
    pub fn get_version(&self) -> u32 {
        self.current_version
    }

    /// Get latest available version
    pub fn get_latest_version(&self) -> u32 {
        self.latest_version
    }

    /// Check if upgrade is available
    pub fn has_upgrade_available(&self) -> bool {
        self.current_version < self.latest_version
    }

    /// Get pending upgrades
    pub fn get_pending_upgrades(&self) -> Vec<u32> {
        self.upgrade_schedule.iter()
            .filter(|e| e.version > 0 && e.scheduled_at.is_some())
            .map(|e| e.version)
            .collect()
    }
}

/// Add upgrade path errors

