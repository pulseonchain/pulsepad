use anchor_lang::prelude::*;
use crate::state::MigrationTarget;

/// Pre-configured migration parameters for a token.
/// Created once during token creation (step 1c) so that migrate()
/// can be fully permissionless — no DEX accounts need to be passed in.
///
/// seeds: [b"migration_config", mint]
#[account]
pub struct MigrationConfig {
    pub mint: Pubkey,
    pub migration_target: MigrationTarget,
    /// DEX program ID (Raydium / Meteora / PumpSwap)
    pub dex_program_id: Pubkey,
    /// For Meteora: fee-sharing config account
    pub fee_share_config: Option<Pubkey>,
    /// DEX pool to send liquidity to (pre-computed PDA or known address)
    pub dex_pool: Pubkey,
    /// DEX token account to receive LP tokens
    pub dex_token_account: Pubkey,
    /// Creator fee_recipient for post-migration LP fees
    pub fee_recipient: Pubkey,
    pub bump: u8,
}

impl MigrationConfig {
    pub const ACCOUNT_SIZE: usize = 8
        + 32   // mint
        + 64   // migration_target (enum worst-case)
        + 32   // dex_program_id
        + 33   // fee_share_config (Option<Pubkey>)
        + 32   // dex_pool
        + 32   // dex_token_account
        + 32   // fee_recipient
        + 1;   // bump

    pub fn init(
        &mut self,
        mint: Pubkey,
        migration_target: MigrationTarget,
        dex_program_id: Pubkey,
        fee_share_config: Option<Pubkey>,
        dex_pool: Pubkey,
        dex_token_account: Pubkey,
        fee_recipient: Pubkey,
        bump: u8,
    ) {
        self.mint = mint;
        self.migration_target = migration_target;
        self.dex_program_id = dex_program_id;
        self.fee_share_config = fee_share_config;
        self.dex_pool = dex_pool;
        self.dex_token_account = dex_token_account;
        self.fee_recipient = fee_recipient;
        self.bump = bump;
    }
}
