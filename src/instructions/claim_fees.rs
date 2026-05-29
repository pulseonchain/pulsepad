use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::FeeClaimedEvent;
use crate::state::{GlobalConfig, PoolState};

pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
    let config = &ctx.accounts.global_config;
    config.validate()?; // Ensure config parameters are valid
    let pool = &ctx.accounts.pool_state;

    // Only current_authority may claim
    require!(
        ctx.accounts.authority.key() == pool.current_authority,
        BondingError::Unauthorized
    );

    let vault_balance = ctx.accounts.fee_recipient.lamports();
    let min_reserve = config.min_creator_reserve;

    require!(vault_balance > min_reserve, BondingError::BelowMinReserve);

    let claimable = vault_balance.checked_sub(min_reserve).ok_or(BondingError::MathOverflow)?;

    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_RECIPIENT,
        mint_key.as_ref(),
        &[pool.fee_recipient_bump],
    ]];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.fee_recipient.to_account_info(),
                to: ctx.accounts.authority.to_account_info(),
            },
            seeds,
        ),
        claimable,
    )?;

    let now = Clock::get()?.unix_timestamp;
    emit!(FeeClaimedEvent {
        mint: mint_key,
        authority: ctx.accounts.authority.key(),
        amount: claimable,
        timestamp: now,
    });

    msg!("Claimed {} lamports from fee_recipient", claimable);
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimFees<'info> {
    #[account(
        seeds = [SEED_GLOBAL_CONFIG],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: mint pubkey used for PDA derivation only
    pub mint: UncheckedAccount<'info>,

    /// CHECK: PDA — holds creator's accumulated 0.25% fees
    #[account(
        mut,
        seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()],
        bump = pool_state.fee_recipient_bump,
    )]
    pub fee_recipient: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
