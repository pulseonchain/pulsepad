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

    // NEW: Tier & Anti-Snipe
    pub graduation_tier: GraduationTier,
    pub anti_snipe_enabled: bool,
    pub pool_init_at: i64,

    // NEW: Partial Migration
    pub partial_migration_pct: u8,
    pub buyback_active: bool,
    pub buyback_sol_reserves: u64,
    pub buyback_token_reserves: u64,
    pub buyback_virtual_sol_reserves: u64,
    pub buyback_virtual_token_reserves: u64,
    pub last_buyback_at: i64,

    // NEW: Agent
    pub agent_wallet: Pubkey,
    pub fees_to_agent: bool,

    // Config
    pub pool_fee_bps: u64,

    pub bump: u8,
    pub fee_vault_bump: u8,
    pub fee_recipient_bump: u8,
    pub lp_reserve_bump: u8,
    pub pool_tokens_bump: u8,
    pub migration_vault_bump: u8,
}

impl PoolState {
    pub const ACCOUNT_SIZE: usize = 8
        + 32 + 32 + 32 + 64 + 8 + 8 + 8 + 8 + 8 + 1 + 33 + 8
        + 1 + 1 + 8 + 1 + 1 + 8 + 8 + 8 + 8 + 8 + 32 + 1 + 8
        + 1 + 1 + 1 + 1 + 1 + 1;

    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        mint: Pubkey,
        creator: Pubkey,
        migration_target: MigrationTarget,
        graduation_tier: GraduationTier,
        anti_snipe_enabled: bool,
        partial_migration_pct: u8,
        agent_wallet: Pubkey,
        fees_to_agent: bool,
        pool_fee_bps: u64,
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
        self.graduation_tier = graduation_tier;
        self.anti_snipe_enabled = anti_snipe_enabled;
        self.pool_init_at = 0;
        self.partial_migration_pct = partial_migration_pct;
        self.buyback_active = false;
        self.buyback_sol_reserves = 0;
        self.buyback_token_reserves = 0;
        self.buyback_virtual_sol_reserves = 0;
        self.buyback_virtual_token_reserves = 0;
        self.last_buyback_at = 0;
        self.agent_wallet = agent_wallet;
        self.fees_to_agent = fees_to_agent;
        self.pool_fee_bps = pool_fee_bps;
        self.bump = bump;
        self.fee_vault_bump = fee_vault_bump;
        self.fee_recipient_bump = fee_recipient_bump;
        self.lp_reserve_bump = lp_reserve_bump;
        self.pool_tokens_bump = pool_tokens_bump;
        self.migration_vault_bump = migration_vault_bump;
    }

    pub fn graduation_threshold(&self) -> u64 {
        self.graduation_tier.threshold_sol()
    }

    pub fn is_anti_snipe_active(&self, now: i64) -> bool {
        if !self.anti_snipe_enabled || self.pool_init_at == 0 { return false; }
        now.saturating_sub(self.pool_init_at) < ANTI_SNIPE_WINDOW_SECS
    }

    pub fn effective_virtual_sol(&self, now: i64) -> u64 {
        if self.is_anti_snipe_active(now) {
            let base = self.virtual_sol_reserves as u128;
            let multiplied = base.saturating_mul(ANTI_SNIPE_MULTIPLIER_BASIS as u128).saturating_div(100);
            multiplied.min(u64::MAX as u128) as u64
        } else {
            self.virtual_sol_reserves
        }
    }

    pub fn has_partial_migration(&self) -> bool {
        self.partial_migration_pct > 0 && self.partial_migration_pct <= 30
    }

    pub fn calc_buy(&self, net_sol: u64) -> Result<u64> {
        let now = Clock::get()?.unix_timestamp;
        self.calc_buy_at(net_sol, now)
    }

    pub fn calc_buy_at(&self, net_sol: u64, now: i64) -> Result<u64> {
        let vt = self.virtual_token_reserves as u128;
        let vs = self.effective_virtual_sol(now) as u128;
        let s  = net_sol as u128;
        let tokens_out = vt.checked_mul(s).ok_or(BondingError::MathOverflow)?
            .checked_div(vs.checked_add(s).ok_or(BondingError::MathOverflow)?)
            .ok_or(BondingError::MathOverflow)? as u64;
        let available = if tokens_out <= self.real_token_reserves {
            self.real_token_reserves
        } else {
            self.real_token_reserves.checked_add(self.reserve_tokens_remaining)
                .ok_or(BondingError::MathOverflow)?
        };
        require!(tokens_out <= available, BondingError::InsufficientPoolTokens);
        Ok(tokens_out)
    }

    pub fn calc_sell(&self, tokens_in: u64) -> Result<u64> {
        let vs = self.virtual_sol_reserves as u128;
        let vt = self.virtual_token_reserves as u128;
        let t  = tokens_in as u128;
        let sol_out = vs.checked_mul(t).ok_or(BondingError::MathOverflow)?
            .checked_div(vt.checked_add(t).ok_or(BondingError::MathOverflow)?)
            .ok_or(BondingError::MathOverflow)? as u64;
        require!(sol_out <= self.real_sol_reserves, BondingError::InsufficientPoolSol);
        Ok(sol_out)
    }

    pub fn apply_buy(&mut self, net_sol: u64, tokens_out: u64) -> (u64, u64) {
        self.virtual_sol_reserves = self.virtual_sol_reserves.saturating_add(net_sol);
        self.virtual_token_reserves = self.virtual_token_reserves.saturating_sub(tokens_out);
        self.real_sol_reserves = self.real_sol_reserves.saturating_add(net_sol);
        if tokens_out <= self.real_token_reserves {
            self.real_token_reserves = self.real_token_reserves.saturating_sub(tokens_out);
            (tokens_out, 0)
        } else {
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

    pub fn activate_buyback(&mut self, kept_sol: u64, kept_tokens: u64) {
        self.buyback_active = true;
        self.buyback_sol_reserves = kept_sol;
        self.buyback_token_reserves = kept_tokens;
        self.buyback_virtual_sol_reserves = self.virtual_sol_reserves;
        self.buyback_virtual_token_reserves = self.virtual_token_reserves;
        self.virtual_sol_reserves = self.virtual_sol_reserves.saturating_sub(kept_sol);
        self.virtual_token_reserves = self.virtual_token_reserves.saturating_add(kept_tokens);
    }
}
