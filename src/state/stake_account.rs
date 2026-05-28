use anchor_lang::prelude::*;

pub const REWARD_PRECISION: u128 = 1_000_000_000_000;

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount_staked: u64,
    pub staked_at: i64,
    pub last_claimed: i64,
    pub reward_debt: u128,  // checkpoint: accumulated_reward_per_token at last snapshot
    pub bump: u8,
}

impl StakeAccount {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // owner
        + 32  // mint
        + 8   // amount_staked
        + 8   // staked_at
        + 8   // last_claimed
        + 16  // reward_debt (u128)
        + 1;  // bump
}

// ─── Staker Reward Vault ──────────────────────────────────────────────────────
// A PDA that holds both on-chain data (reward tracking) and SOL lamports
// (the actual reward SOL). Stakers call claim_staker_rewards() to pull their
// proportional share from this account's lamports.
//
// seeds: [b"staker_vault", mint]
//
// SOL flows here from fee_recipient (Wallet 2) when distribute_creator_fees
// is triggered for Meteora targets with staker_share > 0.
#[account]
pub struct StakerVault {
    pub mint: Pubkey,
    pub total_staked: u64,
    pub accumulated_reward_per_token: u128,
    pub total_distributed: u64,
    pub bump: u8,
}

impl StakerVault {
    pub const SEED: &'static [u8] = b"staker_vault";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 8   // total_staked
        + 16  // accumulated_reward_per_token (u128)
        + 8   // total_distributed
        + 1;  // bump

    /// Increase accumulated_reward_per_token when new SOL rewards arrive
    pub fn add_rewards(&mut self, new_sol: u64) {
        if self.total_staked == 0 || new_sol == 0 {
            return;
        }
        let increase = (new_sol as u128)
            .saturating_mul(REWARD_PRECISION)
            .checked_div(self.total_staked as u128)
            .unwrap_or(0);
        self.accumulated_reward_per_token = self.accumulated_reward_per_token.saturating_add(increase);
        self.total_distributed = self.total_distributed.saturating_add(new_sol);
    }

    /// Calculate pending SOL rewards for a stake account
    pub fn pending_rewards(&self, stake: &StakeAccount) -> u64 {
        self.accumulated_reward_per_token
            .saturating_sub(stake.reward_debt)
            .checked_mul(stake.amount_staked as u128)
            .unwrap_or(0)
            .checked_div(REWARD_PRECISION)
            .unwrap_or(0) as u64
    }
}
