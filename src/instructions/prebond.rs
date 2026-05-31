use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;
use crate::state::*;
use crate::events::*;

/// Create PrebondConfig, AgentWallet, and VaultClaimTracker at pool creation.
pub fn init_prebond(
    ctx: Context<InitPrebond>,
    graduation_tier: GraduationTier,
    total_fee_bps: u64,
    fees_to_agent: bool,
    agent_name: String,
    anti_snipe_enabled: bool,
    partial_migration_pct: u8,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let mint_key = ctx.accounts.mint.key();

    let pbc = &mut ctx.accounts.prebond_config;
    pbc.init(
        mint_key,
        graduation_tier.clone(),
        total_fee_bps,
        fees_to_agent,
        Pubkey::default(),
        agent_name.clone(),
        anti_snipe_enabled,
        partial_migration_pct,
        ctx.accounts.creator.key(),
        ctx.bumps.prebond_config,
        now,
    )?;

    if fees_to_agent {
        let agent_pda = Pubkey::find_program_address(
            &[SEED_AGENT, mint_key.as_ref()],
            &crate::ID,
        ).0;
        let aw = &mut ctx.accounts.agent_wallet;
        aw.init(mint_key, agent_name.clone(), ctx.bumps.agent_wallet, now);
        ctx.accounts.prebond_config.agent_wallet = agent_pda;
    }

    let vct = &mut ctx.accounts.vault_claim_tracker;
    vct.init(mint_key, ctx.accounts.creator.key(), ctx.bumps.vault_claim_tracker);

    emit!(PrebondConfigCreatedEvent {
        mint: mint_key,
        creator: ctx.accounts.creator.key(),
        graduation_tier: format!("{:?}", graduation_tier),
        total_fee_bps,
        fees_to_agent,
        agent_name: agent_name.clone(),
        agent_wallet: ctx.accounts.agent_wallet.key(),
        anti_snipe_enabled,
        partial_migration_pct,
        timestamp: now,
    });

    msg!("PrebondConfig created for {}", mint_key);
    Ok(())
}

#[derive(Accounts)]
pub struct InitPrebond<'info> {
    /// CHECK: mint pubkey
    pub mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        space = PrebondConfig::ACCOUNT_SIZE,
        seeds = [b"prebond_config", mint.key().as_ref()],
        bump,
    )]
    pub prebond_config: Box<Account<'info, PrebondConfig>>,

    #[account(
        init_if_needed,
        payer = creator,
        space = AgentWallet::ACCOUNT_SIZE,
        seeds = [SEED_AGENT, mint.key().as_ref()],
        bump,
    )]
    pub agent_wallet: Box<Account<'info, AgentWallet>>,

    #[account(
        init,
        payer = creator,
        space = VaultClaimTracker::ACCOUNT_SIZE,
        seeds = [b"vault_claim_tracker", mint.key().as_ref(), creator.key().as_ref()],
        bump,
    )]
    pub vault_claim_tracker: Box<Account<'info, VaultClaimTracker>>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Vault claim with 500K token / 24h cap.
pub fn claim_vault_capped(ctx: Context<ClaimVaultCapped>, amount: u64) -> Result<()> {
    let tracker = &mut ctx.accounts.vault_claim_tracker;
    let now = Clock::get()?.unix_timestamp;
    tracker.record_claim(amount, now)?;

    let pool = &ctx.accounts.pool_state;
    let pool_bump = pool.bump;
    let mint_key_ref = ctx.accounts.mint.key();
    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE, mint_key_ref.as_ref(), &[pool_bump],
    ]];

    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.migration_vault_token_account.to_account_info(),
                to: ctx.accounts.claimer_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_state_seeds,
        ),
        amount,
    )?;

    emit!(VaultClaimEvent {
        mint: ctx.accounts.mint.key(),
        claimer: ctx.accounts.claimer.key(),
        amount,
        tokens_claimed_24h: tracker.tokens_claimed_24h,
        timestamp: now,
    });

    msg!("Vault claim: {} tokens (24h total: {})", amount, tracker.tokens_claimed_24h);
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimVaultCapped<'info> {
    #[account(mut)]
    pub mint: Box<Account<'info, anchor_spl::token::Mint>>,

    #[account(mut, seeds = [SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        mut,
        seeds = [b"vault_claim_tracker", mint.key().as_ref(), claimer.key().as_ref()],
        bump = vault_claim_tracker.bump,
    )]
    pub vault_claim_tracker: Box<Account<'info, VaultClaimTracker>>,

    /// CHECK: migration vault ATA
    #[account(mut)]
    pub migration_vault_token_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = claimer,
    )]
    pub claimer_token_account: Box<Account<'info, anchor_spl::token::TokenAccount>>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}
