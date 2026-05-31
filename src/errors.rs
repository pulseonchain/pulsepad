use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;

#[error_code]
pub enum BondingError {
    // ── Core Errors ──────────────────────────────────────────────────────────
    #[msg("Program is currently paused")]
    Paused,
    #[msg("Unauthorized: signer is not the current authority")]
    Unauthorized,
    #[msg("Token has already graduated to a DEX")]
    AlreadyGraduated,
    #[msg("Token has not yet reached the graduation threshold")]
    NotReadyToGraduate,
    #[msg("SOL amount must be greater than zero")]
    ZeroSolAmount,
    #[msg("Token amount must be greater than zero")]
    ZeroTokenAmount,
    #[msg("Insufficient tokens in bonding pool")]
    InsufficientPoolTokens,
    #[msg("Insufficient SOL in pool vault")]
    InsufficientPoolSol,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Creator fee balance is below the minimum reserve")]
    BelowMinReserve,
    #[msg("Invalid migration target configuration")]
    InvalidMigrationConfig,
    #[msg("Meteora lp/staker/holder shares must sum to 100")]
    InvalidShareSum,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Invalid platform wallet address")]
    InvalidPlatformWallet,
    #[msg("Stake amount must be greater than zero")]
    ZeroStakeAmount,
    #[msg("Insufficient staked balance to unstake that amount")]
    InsufficientStake,
    #[msg("No staker rewards available to claim")]
    NoRewardsToClaim,
    #[msg("Token name too long")]
    NameTooLong,
    #[msg("Token symbol too long")]
    SymbolTooLong,
    #[msg("Token URI too long")]
    UriTooLong,
    #[msg("Invalid token name: must be ASCII, 1-32 chars, no spaces")]
    InvalidName,
    #[msg("Invalid token symbol: must be ASCII, uppercase only")]
    InvalidSymbol,
    #[msg("Invalid token URI: must start with https:// or http://")]
    InvalidUri,
    #[msg("This migration target requires Meteora fee-share config")]
    MissingFeeShareConfig,
    #[msg("Invalid fee configuration")]
    InvalidFeeConfig,
    #[msg("Pool is not empty")]
    PoolNotEmpty,
    #[msg("Wallet is not on the whitelist")]
    NotWhitelisted,
    #[msg("Whitelist phase has expired")]
    WhitelistExpired,
    #[msg("Whitelist per-wallet SOL cap exceeded")]
    WhitelistCapExceeded,
    #[msg("Referral config does not exist or is inactive")]
    InvalidReferralConfig,
    #[msg("Pool stats account not initialized")]
    StatsNotInitialized,

    // ── Agent / Prebond Errors ──────────────────────────────────────────────
    #[msg("Agent wallet not configured for this pool")]
    AgentNotConfigured,
    #[msg("Agent claim interval not met (minimum 3 hours)")]
    AgentClaimTooSoon,
    #[msg("Vault claim exceeds 500K token daily cap")]
    VaultDailyCapExceeded,
    #[msg("Partial migration percentage must be 0, 10, 20, or 30")]
    InvalidPartialMigrationPct,
    #[msg("Anti-snipe window is still active — wait 3 minutes")]
    AntiSnipeActive,
    #[msg("Invalid graduation tier")]
    InvalidGraduationTier,
    #[msg("Buyback fund is empty — nothing to execute")]
    BuybackFundEmpty,
    #[msg("Buyback has already been executed for this window")]
    BuybackAlreadyExecuted,

    // ── Security Errors ──────────────────────────────────────────────────────
    #[msg("Reentrancy detected - operation blocked")]
    ReentrancyDetected,
    #[msg("Address is blacklisted")]
    BlacklistedAddress,
    #[msg("Suspicious flash loan activity detected")]
    SuspiciousFlashLoanActivity,
    #[msg("Rate limit exceeded for this window")]
    RateLimitExceeded,
    #[msg("Daily limit exceeded")]
    DailyLimitExceeded,
    #[msg("Invalid transaction ID")]
    InvalidTransactionId,
    #[msg("Invalid proof")]
    InvalidProof,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Protocol is already paused")]
    AlreadyPaused,
    #[msg("Protocol is not currently paused")]
    NotPaused,

    // ── Invariant Errors ─────────────────────────────────────────────────────
    #[msg("Protocol invariant violated")]
    InvalidInvariant,
    #[msg("Math verification failed")]
    MathVerificationFailed,
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    #[msg("Account not rent-exempt")]
    AccountNotRentExempt,
    #[msg("Token supply mismatch")]
    TokenSupplyMismatch,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Constant product invariant violated")]
    ConstantProductViolation,
    #[msg("Graduation verification failed")]
    GraduationVerificationFailed,
    #[msg("Pool state consistency violated")]
    PoolConsistencyViolation,
    #[msg("Invalid pool state: virtual reserves must be > 0")]
    InvalidPoolState,
    #[msg("Price impact too high")]
    PriceImpactTooHigh,

    // ── Economics Errors ─────────────────────────────────────────────────────
    #[msg("Dynamic fee calculation failed")]
    DynamicFeeCalculationFailed,
    #[msg("Fee redistribution calculation failed")]
    FeeRedistributionFailed,

    // ── Upgrade Errors ───────────────────────────────────────────────────────
    #[msg("Upgrade already in progress")]
    UpgradeInProgress,
    #[msg("No pending upgrade")]
    NoPendingUpgrade,
    #[msg("Upgrade delay not met")]
    UpgradeDelayNotMet,
    #[msg("Invalid upgrade delay")]
    InvalidUpgradeDelay,
    #[msg("Upgrade not authorized")]
    UpgradeNotAuthorized,
    #[msg("Invalid upgrade version")]
    InvalidUpgradeVersion,
    #[msg("Invalid upgrade schedule")]
    InvalidUpgradeSchedule,
    #[msg("Description too long")]
    DescriptionTooLong,
    #[msg("Upgrade not found")]
    UpgradeNotFound,
    #[msg("Upgrade deadline not met")]
    UpgradeDeadlineNotMet,
    #[msg("Feature is disabled")]
    FeatureDisabled,
    #[msg("Plugin configuration invalid")]
    InvalidPluginConfig,
    #[msg("Plugin not found")]
    PluginNotFound,
    #[msg("Upgrade path is full")]
    UpgradePathFull,

    // ── Compliance Errors ────────────────────────────────────────────────────
    #[msg("KYC has expired")]
    KycExpired,
    #[msg("KYC not verified for this user")]
    KycNotVerified,
    #[msg("Restricted jurisdiction")]
    RestrictedJurisdiction,
    #[msg("Audit log entry not found")]
    AuditLogEntryNotFound,

    // ── Config Errors ────────────────────────────────────────────────────────
    #[msg("Invalid configuration parameter")]
    InvalidConfig,
}

impl From<crate::state::BondingError> for BondingError {
    fn from(_: crate::state::BondingError) -> Self {
        BondingError::InvalidShareSum
    }
}
