use anchor_lang::prelude::*;

#[error_code]
pub enum BondingError {
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

    #[msg("Creator fee balance is below the minimum reserve — wait for more trading volume")]
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

    #[msg("Token name too long (max 32 chars)")]
    NameTooLong,

    #[msg("Token symbol too long (max 10 chars)")]
    SymbolTooLong,

    #[msg("Token URI too long (max 200 chars)")]
    UriTooLong,

    #[msg("This migration target requires Meteora fee-share config but none was provided")]
    MissingFeeShareConfig,

    // ── Admin / Config ────────────────────────────────────────────────────────
    #[msg("Invalid fee configuration — fee bps too high or shares don't sum to 100")]
    InvalidFeeConfig,

    #[msg("Pool is not empty — drain all liquidity before closing")]
    PoolNotEmpty,

    // ── Whitelist ─────────────────────────────────────────────────────────────
    #[msg("Wallet is not on the whitelist for this token")]
    NotWhitelisted,

    #[msg("Whitelist phase has expired — use buy() instead")]
    WhitelistExpired,

    #[msg("Whitelist per-wallet SOL cap exceeded")]
    WhitelistCapExceeded,

    // ── Referral ──────────────────────────────────────────────────────────────
    #[msg("Referral config does not exist or is inactive")]
    InvalidReferralConfig,

    // ── Pool Stats ────────────────────────────────────────────────────────────
    #[msg("Pool stats account not initialized")]
    StatsNotInitialized,
}
