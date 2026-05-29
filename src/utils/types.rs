use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Types - Common type definitions
// ─────────────────────────────────────────────────────────────────────────────

/// Migration target for graduation
#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum MigrationTarget {
    RaydiumCpmm,
    MeteoraDammV1 {
        enable_dynamic_vault: bool,
        lp_share: u8,
        staker_share: u8,
        holder_share: u8,
    },
    MeteoraDlmm {
        fee_bps: u16,
        bin_step: u16,
        lp_share: u8,
        staker_share: u8,
        holder_share: u8,
    },
    PumpSwapBurn,
    PumpSwapHoldLp,
}

impl MigrationTarget {
    pub fn validate(&self) -> Result<()> {
        match self {
            MigrationTarget::MeteoraDammV1 { lp_share, staker_share, holder_share, .. } => {
                let sum = (*lp_share as u16)
                    .checked_add(*staker_share as u16)
                    .unwrap_or(0)
                    .checked_add(*holder_share as u16)
                    .unwrap_or(0);
                require!(sum == 100, BondingError::InvalidShareSum);
            }
            MigrationTarget::MeteoraDlmm { lp_share, staker_share, holder_share, .. } => {
                let sum = (*lp_share as u16)
                    .checked_add(*staker_share as u16)
                    .unwrap_or(0)
                    .checked_add(*holder_share as u16)
                    .unwrap_or(0);
                require!(sum == 100, BondingError::InvalidShareSum);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn has_ongoing_fees(&self) -> bool {
        !matches!(self, MigrationTarget::PumpSwapBurn)
    }
}

/// Trade direction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TradeDirection {
    Buy,
    Sell,
}

/// Fee type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FeeType {
    Platform,
    Creator,
    Staker,
    Lp,
    Reserve,
}

/// Pool state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PoolState {
    Initializing,
    Active,
    Paused,
    Graduated,
    Closed,
}

/// Whale status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WhaleStatus {
    Normal,
    Whale,
    SuperWhale,
}

/// Health score category
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HealthCategory {
    Critical,
    Warning,
    Healthy,
    Excellent,
}

/// Upgrade status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpgradeStatus {
    NoUpgrade,
    Scheduled,
    Ready,
    InProgress,
}

/// Feature status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FeatureStatus {
    Enabled,
    Disabled,
    Pending,
}

/// Risk level
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RiskLevel {
    Low,
    MediumLow,
    Medium,
    MediumHigh,
    High,
}

/// Compliance status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComplianceStatus {
    Verified,
    Expired,
    NotVerified,
    Restricted,
}

/// Event type for audit logging
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventType {
    TradeBuy,
    TradeSell,
    Migration,
    FeeClaim,
    ConfigUpdate,
    Pause,
    Unpause,
    FundTransfer,
    Staking,
    WhitelistBuy,
    Referral,
}
