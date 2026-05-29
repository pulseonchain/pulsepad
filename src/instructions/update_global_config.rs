use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;
use crate::state::GlobalConfig;

// ─────────────────────────────────────────────────────────────────────────────
// update_global_config() — Admin only.
//
// Allows the platform authority to update any protocol-level parameter
// without redeploying. All fields are optional — pass None to keep the
// existing value.
//
// This is critical for protocol operation:
//   - Adjust graduation threshold (e.g. change from 85 SOL to 100 SOL)
//   - Update platform wallet (rotation)
//   - Pause / unpause trading globally (circuit breaker)
//   - Adjust fee splits (regulatory or competitive reasons)
//
// Security: only the `authority` stored in GlobalConfig can call this.
// ─────────────────────────────────────────────────────────────────────────────

/// Parameters for updating the global config.
/// Every field is optional — None means "don't change."
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateGlobalConfigParams {
    /// New platform wallet address (None = unchanged)
    pub platform_wallet: Option<Pubkey>,
    /// New total fee in basis points (None = unchanged, max 500 = 5%)
    pub fee_basis_points: Option<u64>,
    /// New platform share in bps of the total fee (None = unchanged)
    /// Must satisfy: platform_share_bps + creator_share_bps == 100
    pub platform_share_bps: Option<u64>,
    /// New creator share in bps of the total fee (None = unchanged)
    pub creator_share_bps: Option<u64>,
    /// New graduation SOL threshold in lamports (None = unchanged)
    pub graduation_sol_threshold: Option<u64>,
    /// New minimum creator reserve in lamports (None = unchanged)
    pub min_creator_reserve: Option<u64>,
    /// New max SOL trade size (None = unchanged)
    pub max_trade_sol: Option<u64>,
    /// New max token trade size (None = unchanged)
    pub max_trade_tokens: Option<u64>,
    /// New max price impact in bps (None = unchanged, 10000 = 100%)
    pub max_price_impact_bps: Option<u64>,
    /// Set paused state (None = unchanged)
    pub paused: Option<bool>,
    /// Transfer authority to a new admin (None = unchanged)
    pub new_authority: Option<Pubkey>,
}

pub fn update_global_config(
    ctx: Context<UpdateGlobalConfig>,
    params: UpdateGlobalConfigParams,
) -> Result<()> {
    let config = &mut ctx.accounts.global_config;

    // Verify caller is the current authority
    require!(
        ctx.accounts.authority.key() == config.authority,
        BondingError::Unauthorized
    );

    // Validate new config parameters (if provided)
    if let Some(max_trade_sol) = params.max_trade_sol {
        require!(max_trade_sol > 0 && max_trade_sol <= MAX_TRADE_SOL, BondingError::InvalidConfig);
    }
    if let Some(max_trade_tokens) = params.max_trade_tokens {
        require!(max_trade_tokens > 0 && max_trade_tokens <= MAX_TRADE_TOKENS, BondingError::InvalidConfig);
    }
    if let Some(max_price_impact_bps) = params.max_price_impact_bps {
        require!(max_price_impact_bps <= 10_000, BondingError::InvalidConfig);
    }
    if let Some(fee_bps) = params.fee_basis_points {
        require!(fee_bps <= 500, BondingError::InvalidFeeConfig);
        config.fee_basis_points = fee_bps;
    }

    // Apply share split — must still sum to 100
    match (params.platform_share_bps, params.creator_share_bps) {
        (Some(p), Some(c)) => {
            require!(p.checked_add(c) == Some(100), BondingError::InvalidFeeConfig);
            config.platform_share_bps = p;
            config.creator_share_bps = c;
        }
        (Some(p), None) => {
            let c = 100u64.checked_sub(p).ok_or(BondingError::InvalidFeeConfig)?;
            config.platform_share_bps = p;
            config.creator_share_bps = c;
        }
        (None, Some(c)) => {
            let p = 100u64.checked_sub(c).ok_or(BondingError::InvalidFeeConfig)?;
            config.platform_share_bps = p;
            config.creator_share_bps = c;
        }
        (None, None) => {}
    }

    if let Some(wallet) = params.platform_wallet {
        config.platform_wallet = wallet;
    }
    if let Some(threshold) = params.graduation_sol_threshold {
        require!(threshold > 0, BondingError::ZeroSolAmount);
        config.graduation_sol_threshold = threshold;
    }
    if let Some(reserve) = params.min_creator_reserve {
        config.min_creator_reserve = reserve;
    }
    if let Some(paused) = params.paused {
        config.paused = paused;
        if paused {
            msg!("⚠️  Protocol PAUSED by authority");
        } else {
            msg!("✅ Protocol UNPAUSED by authority");
        }
    }
    if let Some(new_auth) = params.new_authority {
        let old = config.authority;
        config.authority = new_auth;
        msg!("Authority transferred: {} → {}", old, new_auth);
    }

    msg!("GlobalConfig updated by {}", ctx.accounts.authority.key());
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateGlobalConfig<'info> {
    #[account(
        mut,
        seeds = [SEED_GLOBAL_CONFIG],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// Must be the current authority stored in GlobalConfig.
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
