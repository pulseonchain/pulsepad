use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;
use crate::state::GlobalConfig;

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let config = &mut ctx.accounts.global_config;
    let bump = ctx.bumps.global_config;

    let platform_wallet = ctx.accounts.platform_wallet.key();
    let expected: Pubkey = PLATFORM_WALLET.parse().unwrap();
    require!(platform_wallet == expected, BondingError::InvalidPlatformWallet);

    config.init(
        ctx.accounts.authority.key(),
        platform_wallet,
        bump,
    );

    msg!("GlobalConfig initialized. Platform: {}", platform_wallet);
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = GlobalConfig::ACCOUNT_SIZE,
        seeds = [SEED_GLOBAL_CONFIG],
        bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// CHECK: verified in handler against hardcoded PLATFORM_WALLET constant
    pub platform_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
