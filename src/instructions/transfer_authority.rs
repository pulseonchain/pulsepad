use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::AuthorityTransferredEvent;
use crate::state::PoolState;

pub fn transfer_authority(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
    let pool = &mut ctx.accounts.pool_state;

    require!(
        ctx.accounts.current_authority.key() == pool.current_authority,
        BondingError::Unauthorized
    );

    let old_authority = pool.current_authority;
    pool.current_authority = new_authority;

    let now = Clock::get()?.unix_timestamp;
    emit!(AuthorityTransferredEvent {
        mint: ctx.accounts.mint.key(),
        old_authority,
        new_authority,
        timestamp: now,
    });

    msg!(
        "Authority transferred from {} to {}",
        old_authority,
        new_authority
    );
    Ok(())
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(
        mut,
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: pubkey used for PDA derivation only
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub current_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
