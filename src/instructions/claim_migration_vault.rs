use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::state::PoolState;

/// Claim tokens from the migration vault.
/// Only the current_authority (creator) can claim.
/// The migration vault holds half of the remaining tokens at migration time
/// (the other half was burned).
pub fn claim_migration_vault(ctx: Context<ClaimMigrationVault>) -> Result<()> {
    let pool = &ctx.accounts.pool_state;

    require!(pool.graduated, BondingError::NotReadyToGraduate);
    require!(
        ctx.accounts.authority.key() == pool.current_authority,
        BondingError::Unauthorized
    );

    // Validate migration_vault_token_account is the correct ATA
    let expected_vata = anchor_spl::associated_token::get_associated_token_address(
        &ctx.accounts.migration_vault.key(),
        &ctx.accounts.mint.key(),
    );
    require!(
        ctx.accounts.migration_vault_token_account.key() == expected_vata,
        BondingError::InvalidMigrationConfig,
    );

    // Read token account balance from UncheckedAccount data
    let vault_balance = {
        let data = ctx.accounts.migration_vault_token_account.try_borrow_data()?;
        // Token account layout: 32 (mint) + 32 (owner) + 8 (amount) = 72 bytes minimum
        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(&data[64..72]);
        u64::from_le_bytes(amount_bytes)
    };
    require!(vault_balance > 0, BondingError::NoRewardsToClaim);

    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[&[u8]]] = &[&[
        SEED_MIGRATION_VAULT,
        mint_key.as_ref(),
        &[pool.migration_vault_bump],
    ]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.migration_vault_token_account.to_account_info(),
                to: ctx.accounts.creator_token_account.to_account_info(),
                authority: ctx.accounts.migration_vault.to_account_info(),
            },
            seeds,
        ),
        vault_balance,
    )?;

    msg!(
        "Claimed {} tokens from migration vault for {}",
        vault_balance,
        ctx.accounts.authority.key()
    );
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimMigrationVault<'info> {
    #[account(
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    pub mint: Account<'info, Mint>,

    /// CHECK: migration_vault PDA — program-controlled
    #[account(
        seeds = [SEED_MIGRATION_VAULT, mint.key().as_ref()],
        bump = pool_state.migration_vault_bump,
    )]
    pub migration_vault: UncheckedAccount<'info>,

    /// CHECK: validated in handler — must be ATA of migration_vault PDA
    #[account(mut)]
    pub migration_vault_token_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
