use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;

#[account]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub platform_wallet: Pubkey,
    pub fee_basis_points: u64,
    pub platform_share_bps: u64,
    pub creator_share_bps: u64,
    pub graduation_sol_threshold: u64,
    pub min_creator_reserve: u64,
    pub max_trade_sol: u64,
    pub max_trade_tokens: u64,
    pub max_price_impact_bps: u64,
    pub paused: bool,
    pub bump: u8,
}

impl GlobalConfig {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // authority
        + 32  // platform_wallet
        + 8   // fee_basis_points
        + 8   // platform_share_bps
        + 8   // creator_share_bps
        + 8   // graduation_sol_threshold
        + 8   // min_creator_reserve
        + 1   // paused
        + 1;  // bump

    pub fn init(
        &mut self,
        authority: Pubkey,
        platform_wallet: Pubkey,
        bump: u8,
    ) {
        self.authority = authority;
        self.platform_wallet = platform_wallet;
        self.fee_basis_points = TOTAL_FEE_BPS;
        self.platform_share_bps = PLATFORM_SHARE_BPS;
        self.creator_share_bps = CREATOR_SHARE_BPS;
        self.graduation_sol_threshold = GRADUATION_SOL_THRESHOLD;
        self.min_creator_reserve = MIN_CREATOR_RESERVE;
        self.max_trade_sol = MAX_TRADE_SOL;
        self.max_trade_tokens = MAX_TRADE_TOKENS;
        self.max_price_impact_bps = 1000; // 10% max price impact
        self.paused = false;
        self.bump = bump;
    }

    pub fn calc_fees(&self, sol_amount: u64) -> (u64, u64, u64) {
        let total_fee = sol_amount
            .checked_mul(self.fee_basis_points)
            .unwrap_or(0)
            .checked_div(10_000)
            .unwrap_or(0);
        let platform_fee = total_fee
            .checked_mul(self.platform_share_bps)
            .unwrap_or(0)
            .checked_div(100)
            .unwrap_or(0);
        let creator_fee = total_fee.checked_sub(platform_fee).unwrap_or(0);
        (total_fee, platform_fee, creator_fee)
    }

    /// Validate that config parameters are within reasonable bounds
    pub fn validate(&self) -> Result<()> {
        require!(self.fee_basis_points <= MAX_FEE_BPS, BondingError::InvalidFeeConfig);
        require!(self.platform_share_bps + self.creator_share_bps == self.fee_basis_points,
            BondingError::InvalidFeeConfig);
        require!(self.max_trade_sol > 0 && self.max_trade_sol <= MAX_TRADE_SOL,
            BondingError::InvalidConfig);
        require!(self.max_trade_tokens > 0 && self.max_trade_tokens <= MAX_TRADE_TOKENS,
            BondingError::InvalidConfig);
        require!(self.max_price_impact_bps <= 10_000, BondingError::InvalidConfig);
        Ok(())
    }

}
