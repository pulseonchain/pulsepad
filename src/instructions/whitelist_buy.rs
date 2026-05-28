use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::consts::*;
use crate::errors::BondingError;
use crate::events::{BuyEvent, WhitelistBuyEvent};
use crate::state::{GlobalConfig, PoolState};

// ─────────────────────────────────────────────────────────────────────────────
// whitelist_buy() — Presale / early-access gate.
//
// An optional, creator-controlled presale phase for a token. Creators can
// activate a whitelist that:
//   1. Restricts buying to approved wallets for a configurable time window
//   2. Caps each whitelisted wallet's buy to a max_per_wallet amount
//   3. Optionally enforces a discounted fee for whitelisted buyers
//
// The WhitelistConfig PDA is created by the creator post-initialization
// and before any trading begins. It contains a Merkle root — the SDK
// provides helpers to generate and verify proofs off-chain.
//
// After the whitelist_end_time passes, anyone can buy normally (via buy()).
//
// Flow:
//   Creator creates WhitelistConfig with merkle_root + window + per-wallet cap
//   → Whitelisted buyers call whitelist_buy() with their Merkle proof
//   → After window: normal buy() works for everyone
// ─────────────────────────────────────────────────────────────────────────────

pub fn whitelist_buy(
    ctx: Context<WhitelistBuy>,
    sol_amount: u64,
    min_tokens_out: u64,
    merkle_proof: Vec<[u8; 32]>,
) -> Result<()> {
    let config = &ctx.accounts.global_config;
    require!(!config.paused, BondingError::Paused);
    require!(sol_amount > 0, BondingError::ZeroSolAmount);

    let pool = &mut ctx.accounts.pool_state;
    require!(!pool.graduated, BondingError::AlreadyGraduated);

    let wl = &ctx.accounts.whitelist_config;
    let now = Clock::get()?.unix_timestamp;

    // ── 1. Verify whitelist is still active ───────────────────────────────────
    require!(now < wl.whitelist_end_time, BondingError::WhitelistExpired);

    // ── 2. Verify Merkle proof ────────────────────────────────────────────────
    let leaf = anchor_lang::solana_program::hash::hash(
        ctx.accounts.user.key().as_ref()
    ).to_bytes();
    require!(
        verify_merkle_proof(&merkle_proof, wl.merkle_root, leaf),
        BondingError::NotWhitelisted
    );

    // ── 3. Check per-wallet cap ───────────────────────────────────────────────
    let wl_record = &mut ctx.accounts.whitelist_record;
    require!(
        wl_record.sol_spent.saturating_add(sol_amount) <= wl.max_sol_per_wallet,
        BondingError::WhitelistCapExceeded
    );

    // ── 4. Apply discounted fee if configured ─────────────────────────────────
    let effective_fee_bps = if wl.discounted_fee_bps > 0 {
        wl.discounted_fee_bps as u64
    } else {
        config.fee_basis_points
    };

    let total_fee = sol_amount
        .checked_mul(effective_fee_bps)
        .unwrap_or(0)
        .checked_div(10_000)
        .unwrap_or(0);
    let platform_fee = total_fee
        .checked_mul(config.platform_share_bps)
        .unwrap_or(0)
        .checked_div(100)
        .unwrap_or(0);
    let creator_fee = total_fee.saturating_sub(platform_fee);
    let net_sol = sol_amount.checked_sub(total_fee).ok_or(BondingError::MathOverflow)?;

    // ── 5. Price the buy ──────────────────────────────────────────────────────
    let tokens_out = pool.calc_buy(net_sol)?;
    require!(tokens_out >= min_tokens_out, BondingError::SlippageExceeded);

    let mint_key = ctx.accounts.mint.key();
    let fee_vault_seeds: &[&[&[u8]]] = &[&[
        SEED_FEE_VAULT,
        mint_key.as_ref(),
        &[pool.fee_vault_bump],
    ]];
    let pool_state_seeds: &[&[&[u8]]] = &[&[
        SEED_POOL_STATE,
        mint_key.as_ref(),
        &[pool.bump],
    ]];

    // ── 6. Transfer SOL → fee_vault ───────────────────────────────────────────
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
            },
        ),
        sol_amount,
    )?;

    // ── 7. Distribute fees ────────────────────────────────────────────────────
    if platform_fee > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.platform_wallet.to_account_info(),
                },
                fee_vault_seeds,
            ),
            platform_fee,
        )?;
    }
    if creator_fee > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.fee_recipient.to_account_info(),
                },
                fee_vault_seeds,
            ),
            creator_fee,
        )?;
    }

    // ── 8. Transfer tokens to buyer ───────────────────────────────────────────
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_state_seeds,
        ),
        tokens_out,
    )?;

    // ── 9. Update state ───────────────────────────────────────────────────────
    let (_from_bonding, _from_reserve) = pool.apply_buy(net_sol, tokens_out);
    wl_record.sol_spent = wl_record.sol_spent.saturating_add(sol_amount);
    wl_record.tokens_received = wl_record.tokens_received.saturating_add(tokens_out);

    emit!(WhitelistBuyEvent {
        mint: mint_key,
        buyer: ctx.accounts.user.key(),
        sol_amount,
        tokens_out,
        platform_fee,
        creator_fee,
        discount_applied: wl.discounted_fee_bps > 0,
        timestamp: now,
    });

    msg!(
        "Whitelist buy: {} lamports → {} tokens | fee_bps={} | buyer={}",
        sol_amount, tokens_out, effective_fee_bps, ctx.accounts.user.key()
    );
    Ok(())
}

/// Simple binary Merkle proof verification.
/// Leaf is hash(wallet_pubkey). Root is stored in WhitelistConfig.
fn verify_merkle_proof(proof: &[[u8; 32]], root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut current = leaf;
    for sibling in proof {
        // Sort before hashing to match standard implementations
        let (left, right) = if current <= *sibling {
            (current, *sibling)
        } else {
            (*sibling, current)
        };
        let mut input = [0u8; 64];
        input[..32].copy_from_slice(&left);
        input[32..].copy_from_slice(&right);
        current = anchor_lang::solana_program::hash::hash(&input).to_bytes();
    }
    current == root
}

// ─── Whitelist Config State ───────────────────────────────────────────────────
// Seeds: [b"whitelist_config", mint]

#[account]
pub struct WhitelistConfig {
    pub mint: Pubkey,
    pub creator: Pubkey,
    /// Merkle root of the whitelisted wallet addresses
    pub merkle_root: [u8; 32],
    /// Unix timestamp when the whitelist phase ends (normal trading opens)
    pub whitelist_end_time: i64,
    /// Maximum SOL any single whitelisted wallet can spend during the window
    pub max_sol_per_wallet: u64,
    /// Discounted fee bps for whitelisted buyers (0 = use normal fee)
    pub discounted_fee_bps: u16,
    pub bump: u8,
    pub _padding: [u8; 1],
}

impl WhitelistConfig {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 32  // creator
        + 32  // merkle_root
        + 8   // whitelist_end_time
        + 8   // max_sol_per_wallet
        + 2   // discounted_fee_bps
        + 1   // bump
        + 1;  // padding
}

// ─── Per-Wallet Whitelist Record ──────────────────────────────────────────────
// Seeds: [b"wl_record", mint, user]

#[account]
pub struct WhitelistRecord {
    pub mint: Pubkey,
    pub wallet: Pubkey,
    pub sol_spent: u64,
    pub tokens_received: u64,
    pub bump: u8,
}

impl WhitelistRecord {
    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 32  // wallet
        + 8   // sol_spent
        + 8   // tokens_received
        + 1;  // bump
}

#[derive(Accounts)]
pub struct WhitelistBuy<'info> {
    #[account(seeds = [SEED_GLOBAL_CONFIG], bump = global_config.bump)]
    pub global_config: Box<Account<'info, GlobalConfig>>,

    #[account(
        mut,
        seeds = [SEED_POOL_STATE, mint.key().as_ref()],
        bump = pool_state.bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_POOL_TOKENS, mint.key().as_ref()],
        bump = pool_state.pool_tokens_bump,
        token::mint = mint,
        token::authority = pool_state,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: whitelist config for this pool
    #[account(
        seeds = [b"whitelist_config", mint.key().as_ref()],
        bump = whitelist_config.bump,
    )]
    pub whitelist_config: Box<Account<'info, WhitelistConfig>>,

    #[account(
        init_if_needed,
        payer = user,
        space = WhitelistRecord::ACCOUNT_SIZE,
        seeds = [b"wl_record", mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub whitelist_record: Box<Account<'info, WhitelistRecord>>,

    /// CHECK: platform wallet
    #[account(mut, address = global_config.platform_wallet)]
    pub platform_wallet: UncheckedAccount<'info>,

    /// CHECK: fee vault
    #[account(mut, seeds = [SEED_FEE_VAULT, mint.key().as_ref()], bump = pool_state.fee_vault_bump)]
    pub fee_vault: UncheckedAccount<'info>,

    /// CHECK: fee recipient
    #[account(mut, seeds = [SEED_FEE_RECIPIENT, mint.key().as_ref()], bump = pool_state.fee_recipient_bump)]
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
