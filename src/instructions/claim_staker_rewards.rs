use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::StakeRewardClaimedEvent;
use crate::state::{PoolState, StakeAccount, StakerVault};

pub fn claim_staker_rewards(ctx: Context<ClaimStakerRewards>) -> Result<()> {
    require!(
        ctx.accounts.user.key() == ctx.accounts.stake_account.owner,
        BondingError::Unauthorized
    );

    // Calculate pending SOL rewards
    let pending = {
        let vault = &ctx.accounts.staker_vault;
        let acc   = &ctx.accounts.stake_account;
        vault.pending_rewards(acc)
    };

    require!(pending > 0, BondingError::NoRewardsToClaim);

    let mint_key = ctx.accounts.mint.key();

    // ── Transfer SOL from staker_vault lamports → user ────────────────────────
    // staker_vault is a PDA that holds both Anchor data AND SOL lamports.
    // We use direct lamport manipulation (the safe pattern for PDAs holding both).
    {
        let vault_info = ctx.accounts.staker_vault.to_account_info();
        let user_info  = ctx.accounts.user.to_account_info();

        // Verify the vault has enough lamports beyond its rent-exempt minimum
        let vault_lamports = vault_info.lamports();
        let rent = Rent::get()?;
        let vault_data_len = vault_info.data_len();
        let min_lamports = rent.minimum_balance(vault_data_len);

        require!(
            vault_lamports.saturating_sub(min_lamports) >= pending,
            BondingError::InsufficientPoolSol
        );

        **vault_info.try_borrow_mut_lamports()? -= pending;
        **user_info.try_borrow_mut_lamports()? += pending;
    }

    // ── Update reward debt checkpoint ─────────────────────────────────────────
    let new_debt = ctx.accounts.staker_vault.accumulated_reward_per_token;
    let acc = &mut ctx.accounts.stake_account;
    acc.reward_debt   = new_debt;
    acc.last_claimed  = Clock::get()?.unix_timestamp;

    let now = acc.last_claimed;

    emit!(StakeRewardClaimedEvent {
        mint: mint_key,
        staker: ctx.accounts.user.key(),
        amount: pending,
        timestamp: now,
    });

    msg!(
        "Staker {} claimed {} lamports in rewards",
        ctx.accounts.user.key(),
        pending
    );
    Ok(())
}

// ─── Distribute incoming SOL to StakerVault (internal helper) ────────────────
// Called when Meteora fee sharing sends SOL to fee_recipient.
// Moves the staker_share portion into staker_vault and updates the global
// reward-per-token accumulator. This function is called from claim_lp_fees
// or any future fee distribution instruction.
pub fn credit_staker_vault<'a>(
    staker_vault: &mut Account<'a, StakerVault>,
    staker_vault_info: &'a AccountInfo<'a>,
    source_info: &'a AccountInfo<'a>,
    source_seeds: &[&[&[u8]]],
    amount: u64,
    system_program: &'a Program<'a, System>,
) -> Result<()> {
    use anchor_lang::system_program;

    if amount == 0 || staker_vault.total_staked == 0 {
        return Ok(());
    }

    system_program::transfer(
        CpiContext::new_with_signer(
            system_program.to_account_info(),
            system_program::Transfer {
                from: source_info.clone(),
                to: staker_vault_info.clone(),
            },
            source_seeds,
        ),
        amount,
    )?;

    staker_vault.add_rewards(amount);
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimStakerRewards<'info> {
    #[account(
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_STAKE, mint.key().as_ref(), user.key().as_ref()],
        bump = stake_account.bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    // staker_vault holds both the Anchor data and the SOL reward lamports
    #[account(
        mut,
        seeds = [StakerVault::SEED, mint.key().as_ref()],
        bump = staker_vault.bump,
    )]
    pub staker_vault: Account<'info, StakerVault>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
