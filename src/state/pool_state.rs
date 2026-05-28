use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;

// ─── Migration Target ─────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
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

    pub fn size() -> usize {
        1 + 5
    }
}

// ─── Pool State ───────────────────────────────────────────────────────────────

#[account]
pub struct PoolState {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub current_authority: Pubkey,
    pub migration_target: MigrationTarget,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub reserve_tokens_remaining: u64,
    pub graduated: bool,
    pub dex_pool: Option<Pubkey>,
    pub created_at: i64,
    pub bump: u8,
    pub fee_vault_bump: u8,
    pub fee_recipient_bump: u8,
    pub lp_reserve_bump: u8,
    pub pool_tokens_bump: u8,
    pub migration_vault_bump: u8,
}

impl PoolState {
    pub const ACCOUNT_SIZE: usize = 8
        + 32   // mint
        + 32   // creator
        + 32   // current_authority
        + 64   // migration_target (enum + worst-case data)
        + 8    // virtual_sol_reserves
        + 8    // virtual_token_reserves
        + 8    // real_sol_reserves
        + 8    // real_token_reserves
        + 8    // reserve_tokens_remaining
        + 1    // graduated
        + 33   // dex_pool (Option<Pubkey>)
        + 8    // created_at
        + 1    // bump
        + 1    // fee_vault_bump
        + 1    // fee_recipient_bump
        + 1    // lp_reserve_bump
        + 1    // pool_tokens_bump
        + 1;   // migration_vault_bump

    pub fn init(
        &mut self,
        mint: Pubkey,
        creator: Pubkey,
        migration_target: MigrationTarget,
        bump: u8,
        fee_vault_bump: u8,
        fee_recipient_bump: u8,
        lp_reserve_bump: u8,
        pool_tokens_bump: u8,
        migration_vault_bump: u8,
        now: i64,
    ) {
        self.mint = mint;
        self.creator = creator;
        self.current_authority = creator;
        self.migration_target = migration_target;
        self.virtual_sol_reserves = INITIAL_VIRTUAL_SOL;
        self.virtual_token_reserves = INITIAL_VIRTUAL_TOKEN;
        self.real_sol_reserves = 0;
        self.real_token_reserves = BONDING_SUPPLY;
        self.reserve_tokens_remaining = RESERVE_SUPPLY;
        self.graduated = false;
        self.dex_pool = None;
        self.created_at = now;
        self.bump = bump;
        self.fee_vault_bump = fee_vault_bump;
        self.fee_recipient_bump = fee_recipient_bump;
        self.lp_reserve_bump = lp_reserve_bump;
        self.pool_tokens_bump = pool_tokens_bump;
        self.migration_vault_bump = migration_vault_bump;
    }

    // ── Pure constant-product pricing (no price cap) ──────────────────────────
    // Reserve tokens (97M) are ONLY used when a single buy request exceeds
    // the remaining bonding supply. This prevents gradual reserve drain
    // and ensures the reserve is preserved for large buys near graduation.

    pub fn calc_buy(&self, net_sol: u64) -> Result<u64> {
        let vt = self.virtual_token_reserves as u128;
        let vs = self.virtual_sol_reserves as u128;
        let s  = net_sol as u128;

        let tokens_out = vt
            .checked_mul(s)
            .ok_or(BondingError::MathOverflow)?
            .checked_div(vs.checked_add(s).ok_or(BondingError::MathOverflow)?)
            .ok_or(BondingError::MathOverflow)? as u64;

        // Reserve is only tapped when bonding is insufficient for this single buy
        let available = if tokens_out <= self.real_token_reserves {
            self.real_token_reserves
        } else {
            self.real_token_reserves
                .checked_add(self.reserve_tokens_remaining)
                .ok_or(BondingError::MathOverflow)?
        };
        require!(tokens_out <= available, BondingError::InsufficientPoolTokens);
        Ok(tokens_out)
    }

    pub fn calc_sell(&self, tokens_in: u64) -> Result<u64> {
        let vs = self.virtual_sol_reserves as u128;
        let vt = self.virtual_token_reserves as u128;
        let t  = tokens_in as u128;

        let sol_out = vs
            .checked_mul(t)
            .ok_or(BondingError::MathOverflow)?
            .checked_div(vt.checked_add(t).ok_or(BondingError::MathOverflow)?)
            .ok_or(BondingError::MathOverflow)? as u64;

        require!(sol_out <= self.real_sol_reserves, BondingError::InsufficientPoolSol);
        Ok(sol_out)
    }

    /// Apply a buy: deduct from bonding first. Only tap reserve when this
    /// single buy exceeds remaining bonding supply.
    /// Returns (bonding_deducted, reserve_deducted) for event logging.
    pub fn apply_buy(&mut self, net_sol: u64, tokens_out: u64) -> (u64, u64) {
        self.virtual_sol_reserves = self.virtual_sol_reserves.saturating_add(net_sol);
        self.virtual_token_reserves = self.virtual_token_reserves.saturating_sub(tokens_out);
        self.real_sol_reserves = self.real_sol_reserves.saturating_add(net_sol);

        if tokens_out <= self.real_token_reserves {
            // Normal case: bonding covers the full buy
            self.real_token_reserves = self.real_token_reserves.saturating_sub(tokens_out);
            (tokens_out, 0)
        } else {
            // Large buy: exhaust bonding, take remainder from reserve
            let from_bonding = self.real_token_reserves;
            let from_reserve = tokens_out - from_bonding;
            self.real_token_reserves = 0;
            self.reserve_tokens_remaining = self.reserve_tokens_remaining.saturating_sub(from_reserve);
            (from_bonding, from_reserve)
        }
    }

    pub fn apply_sell(&mut self, tokens_in: u64, sol_out: u64) {
        self.virtual_sol_reserves = self.virtual_sol_reserves.saturating_sub(sol_out);
        self.virtual_token_reserves = self.virtual_token_reserves.saturating_add(tokens_in);
        self.real_sol_reserves = self.real_sol_reserves.saturating_sub(sol_out);
        self.real_token_reserves = self.real_token_reserves.saturating_add(tokens_in);
    }

    pub fn is_ready_to_graduate(&self, threshold: u64) -> bool {
        !self.graduated && self.real_sol_reserves >= threshold
    }
}
