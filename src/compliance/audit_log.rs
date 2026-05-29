use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// AuditLog - Immutable log of important protocol events
// Seeds: [b"audit_log", mint, log_index]
// ─────────────────────────────────────────────────────────────────────────────

pub const LOG_TYPE_TRADE: u8 = 0;
pub const LOG_TYPE_MIGRATION: u8 = 1;
pub const LOG_TYPE_FEE_CLAIM: u8 = 2;
pub const LOG_TYPE_CONFIG_UPDATE: u8 = 3;
pub const LOG_TYPE_PAUSE: u8 = 4;
pub const LOG_TYPE_FUND_TRANSFER: u8 = 5;

#[account]
pub struct AuditLogEntry {
    pub mint: Pubkey,
    pub log_index: u64,
    pub log_type: u8,
    pub timestamp: i64,
    pub authority: Pubkey,
    pub details: [u8; 128], // Log-specific details
    pub bump: u8,
}

impl AuditLogEntry {
    pub const SEED: &'static [u8] = b"audit_log";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // log_index
        + 1   // log_type
        + 8   // timestamp
        + 32  // authority
        + 128 // details
        + 1;  // bump

    /// Create a new audit log entry
    pub fn create(
        &mut self,
        mint: &Pubkey,
        log_index: u64,
        log_type: u8,
        authority: &Pubkey,
        details: &[u8],
        bump: u8,
    ) {
        self.mint = *mint;
        self.log_index = log_index;
        self.log_type = log_type;
        self.timestamp = Clock::get().unwrap().unix_timestamp;
        self.authority = *authority;
        
        // Copy details (truncate if too long)
        let copy_len = details.len().min(128);
        self.details[..copy_len].copy_from_slice(&details[..copy_len]);
        self.bump = bump;
    }

    /// Get trade details
    pub fn get_trade_details(&self) -> (u64, u64, u64) {
        let data = &self.details;
        (
            u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8])),
        )
    }

    /// Get migration details
    pub fn get_migration_details(&self) -> (u64, u64, Pubkey) {
        let data = &self.details;
        (
            u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8])),
            Pubkey::try_from(&data[16..48]).unwrap_or(Pubkey::default()),
        )
    }

    /// Check if this is a trade log
    pub fn is_trade(&self) -> bool {
        self.log_type == LOG_TYPE_TRADE
    }

    /// Check if this is a migration log
    pub fn is_migration(&self) -> bool {
        self.log_type == LOG_TYPE_MIGRATION
    }

    /// Check if this is a fee claim log
    pub fn is_fee_claim(&self) -> bool {
        self.log_type == LOG_TYPE_FEE_CLAIM
    }
}

/// Add audit log errors

