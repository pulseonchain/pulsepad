use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;
use crate::state::GlobalConfig;

// ─────────────────────────────────────────────────────────────────────────────
// Referral System
//
// A trustless on-chain referral program that splits the creator's 0.25% fee
// between the creator and a referrer wallet. This incentivizes third-party
// frontends, bots, and aggregators to integrate Pulse.
//
// How it works:
//   - The referrer's wallet address is passed as a remaining account in buy/sell
//   - If a valid ReferralConfig PDA exists for the referrer, the referral split
//     is applied automatically in the instruction handler
//   - The referrer earns `referral_share_bps` of the creator_fee
//   - The creator earns the remainder
//
// ReferralConfig is created by the platform (to prevent abuse) and specifies
// what % of the creator's share goes to the referrer.
//
// Example: 50% referral share of creator's 0.25% = 0.125% to referrer
// ─────────────────────────────────────────────────────────────────────────────

pub const SEED_REFERRAL_CONFIG: &[u8] = b"referral_config";
pub const SEED_REFERRAL_RECORD: &[u8] = b"referral_record";

/// Global referral configuration for a specific referrer wallet.
/// Created by platform authority — referrers can't create their own.
/// Seeds: [b"referral_config", referrer]
#[account]
pub struct ReferralConfig {
    /// The referrer's wallet address
    pub referrer: Pubkey,
    /// Referral share in bps of the creator_fee (e.g., 5000 = 50% of creator's 0.25%)
    /// Max 10_000 (100% — referrer gets all of creator's fee)
    pub referral_share_bps: u16,
    /// Whether this referral config is active
    pub active: bool,
    pub bump: u8,
}

impl ReferralConfig {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // referrer
        + 2   // referral_share_bps
        + 1   // active
        + 1;  // bump
}

/// Per-(referrer, mint) referral tracking.
/// Seeds: [b"referral_record", referrer, mint]
#[account]
pub struct ReferralRecord {
    pub referrer: Pubkey,
    pub mint: Pubkey,
    /// Total SOL earned from trades referencing this mint (lamports)
    pub total_earned: u64,
    /// Total number of trades referred
    pub total_referrals: u64,
    pub bump: u8,
}

impl ReferralRecord {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // referrer
        + 32  // mint
        + 8   // total_earned
        + 8   // total_referrals
        + 1;  // bump
}

/// Create a referral config for a referrer wallet.
/// Only callable by the platform authority.
pub fn create_referral_config(
    ctx: Context<CreateReferralConfig>,
    referral_share_bps: u16,
) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid
    require!(
        ctx.accounts.platform_authority.key() == config.authority,
        BondingError::Unauthorized
    );
    require!(referral_share_bps <= 10_000, BondingError::InvalidFeeConfig);

    let rc = &mut ctx.accounts.referral_config;
    rc.referrer = ctx.accounts.referrer.key();
    rc.referral_share_bps = referral_share_bps;
    rc.active = true;
    rc.bump = ctx.bumps.referral_config;

    msg!(
        "Referral config created: referrer={} share={}bps",
        rc.referrer, referral_share_bps
    );
    Ok(())
}

/// Disable/enable a referral config.
/// Only callable by the platform authority.
pub fn set_referral_active(
    ctx: Context<SetReferralActive>,
    active: bool,
) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid
    require!(
        ctx.accounts.platform_authority.key() == ctx.accounts.global_config.authority,
        BondingError::Unauthorized
    );
    ctx.accounts.referral_config.active = active;
    msg!(
        "Referral config {} for {}",
        if active { "ACTIVATED" } else { "DEACTIVATED" },
        ctx.accounts.referral_config.referrer
    );
    Ok(())
}

/// Helper: apply referral split to creator_fee.
/// Returns (referral_amount, creator_remainder).
/// Call this inside buy/sell if a referral account is present.
pub fn split_creator_fee_with_referral(
    creator_fee: u64,
    referral_share_bps: u16,
) -> (u64, u64) {
    let referral_amount = (creator_fee as u128)
        .saturating_mul(referral_share_bps as u128)
        .checked_div(10_000)
        .unwrap_or(0) as u64;
    let creator_remainder = creator_fee.saturating_sub(referral_amount);
    (referral_amount, creator_remainder)
}

#[derive(Accounts)]
pub struct CreateReferralConfig<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        init,
        payer = platform_authority,
        space = ReferralConfig::ACCOUNT_SIZE,
        seeds = [SEED_REFERRAL_CONFIG, referrer.key().as_ref()],
        bump,
    )]
    pub referral_config: Account<'info, ReferralConfig>,

    /// CHECK: the referrer wallet — just a pubkey, no signer required
    pub referrer: UncheckedAccount<'info>,

    #[account(mut)]
    pub platform_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetReferralActive<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [SEED_REFERRAL_CONFIG, referral_config.referrer.as_ref()],
        bump = referral_config.bump,
    )]
    pub referral_config: Account<'info, ReferralConfig>,

    #[account(mut)]
    pub platform_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
