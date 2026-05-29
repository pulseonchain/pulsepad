use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// PluginSystem - Allows custom logic per token pool
// Seeds: [b"plugin", mint, plugin_type]
// ─────────────────────────────────────────────────────────────────────────────

pub const PLUGIN_TYPE_FEE_SCHEDULE: u8 = 0;
pub const PLUGIN_TYPE_LIQUIDITY_MONITOR: u8 = 1;
pub const PLUGIN_TYPE_SECURITY_CHECK: u8 = 2;

#[account]
pub struct PluginConfig {
    pub mint: Pubkey,
    pub plugin_type: u8,
    pub config_data: [u8; 256], // Custom configuration
    pub enabled: bool,
    pub bump: u8,
}

impl PluginConfig {
    pub const SEED: &'static [u8] = b"plugin";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 1   // plugin_type
        + 256 // config_data
        + 1   // enabled
        + 1;  // bump

    /// Initialize plugin config
    pub fn init(
        &mut self,
        mint: &Pubkey,
        plugin_type: u8,
        config_data: &[u8; 256],
        bump: u8,
    ) {
        self.mint = *mint;
        self.plugin_type = plugin_type;
        self.config_data = *config_data;
        self.enabled = true;
        self.bump = bump;
    }

    /// Check if plugin is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get fee schedule config (for PLUGIN_TYPE_FEE_SCHEDULE)
    pub fn get_fee_schedule(&self) -> [u64; 8] {
        let data = &self.config_data;
        [
            u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[24..32].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[32..40].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[40..48].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[48..56].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[56..64].try_into().unwrap_or([0; 8])),
        ]
    }

    /// Get liquidity monitor config (for PLUGIN_TYPE_LIQUIDITY_MONITOR)
    pub fn get_liquidity_thresholds(&self) -> (u64, u64, u64) {
        let data = &self.config_data;
        (
            u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8])),
        )
    }

    /// Get security check config (for PLUGIN_TYPE_SECURITY_CHECK)
    pub fn get_security_config(&self) -> (u64, u64, u64) {
        let data = &self.config_data;
        (
            u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8])),
        )
    }
}

/// Add plugin errors

