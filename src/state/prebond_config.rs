use anchor_lang::prelude::*;
use crate::consts::*;
use crate::errors::BondingError;

/// ─── Pre-Bonding Configuration ──────────────────────────────────────────────
/// Set at pool creation. Immutable after initialization.
/// Controls: tier, fees, agent assignment, anti-snipe, partial migration.
#[account]
#[derive(Default)]
pub struct PrebondConfig {
    pub mint: Pubkey,

    /// Graduation tier: Fast(80 SOL) | Standard(150 SOL) | Stable(240 SOL)
    pub graduation_tier: GraduationTier,

    /// Total fee in basis points (100-500, i.e. 1%-5%)
    /// Platform ALWAYS gets 3/4 of this. Creator gets the remaining 1/4.
    pub total_fee_bps: u64,

    /// Whether fees are routed to the agent wallet instead of creator
    pub fees_to_agent: bool,

    /// Agent PDA wallet (if fees_to_agent is true, this gets the 0.25%)
    pub agent_wallet: Pubkey,

    /// Agent name: "Agent <TICKER>" format
    pub agent_name: String,  // stored as 23 bytes max: "Agent " + 16 char ticker

    /// Anti-snipe: if true, first 3 minutes have 3x virtual SOL multiplier
    pub anti_snipe_enabled: bool,

    /// Whether partial migration is configured (keep some SOL in curve)
    pub partial_migration_pct: u8,  // 0, 10, 20, or 30

    /// Creator (original deployer, immutable)
    pub creator: Pubkey,

    /// Created timestamp
    pub created_at: i64,

    pub bump: u8,
}

impl PrebondConfig {
    // 8 bytes discriminator
    // + 32 mint
    // + 1  graduation_tier (enum)
    // + 8  total_fee_bps
    // + 1  fees_to_agent
    // + 32 agent_wallet
    // + 24 agent_name (4 bytes len + 20 bytes max)
    // + 1  anti_snipe_enabled
    // + 1  partial_migration_pct
    // + 32 creator
    // + 8  created_at
    // + 1  bump
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 1 + 8 + 1 + 32 + 24 + 1 + 1 + 32 + 8 + 1;

    pub fn init(
        &mut self,
        mint: Pubkey,
        graduation_tier: GraduationTier,
        total_fee_bps: u64,
        fees_to_agent: bool,
        mut agent_wallet: Pubkey,
        agent_name: String,
        anti_snipe_enabled: bool,
        partial_migration_pct: u8,
        creator: Pubkey,
        bump: u8,
        now: i64,
    ) -> Result<()> {
        // Validate fee range
        require!(
            total_fee_bps >= MIN_CREATOR_FEE_BPS && total_fee_bps <= MAX_CREATOR_FEE_BPS,
            BondingError::InvalidFeeConfig
        );

        // Validate agent name: must start with "Agent " followed by 1-16 ASCII chars
        if fees_to_agent {
            require!(
                agent_name.starts_with(AGENT_NAME_PREFIX),
                BondingError::InvalidName
            );
            let ticker_part = &agent_name[AGENT_NAME_PREFIX.len()..];
            require!(
                ticker_part.len() >= 1 && ticker_part.len() <= 16,
                BondingError::NameTooLong
            );
            require!(ticker_part.is_ascii(), BondingError::InvalidName);
            // Set agent_wallet to the PDA if not explicitly provided
            if agent_wallet == Pubkey::default() {
                agent_wallet = Pubkey::find_program_address(
                    &[SEED_AGENT, mint.as_ref()],
                    &crate::ID,
                ).0;
            }
        }

        // Validate partial migration percentage
        require!(
            partial_migration_pct == 0
                || partial_migration_pct == 10
                || partial_migration_pct == 20
                || partial_migration_pct == 30,
            BondingError::InvalidConfig
        );

        self.mint = mint;
        self.graduation_tier = graduation_tier;
        self.total_fee_bps = total_fee_bps;
        self.fees_to_agent = fees_to_agent;
        self.agent_wallet = agent_wallet;
        self.agent_name = agent_name;
        self.anti_snipe_enabled = anti_snipe_enabled;
        self.partial_migration_pct = partial_migration_pct;
        self.creator = creator;
        self.created_at = now;
        self.bump = bump;

        Ok(())
    }

    /// Calculate fee split given the prebond-configured fee bps.
    /// Platform ALWAYS gets 3/4 of total fee.
    pub fn calc_fees(&self, sol_amount: u64) -> (u64, u64, u64) {
        let total_fee = sol_amount
            .checked_mul(self.total_fee_bps)
            .unwrap_or(0)
            .checked_div(10_000)
            .unwrap_or(0);
        let platform_fee = total_fee
            .checked_mul(PLATFORM_FRACTION)
            .unwrap_or(0)
            .checked_div(100)
            .unwrap_or(0);
        let creator_or_agent_fee = total_fee.saturating_sub(platform_fee);
        (total_fee, platform_fee, creator_or_agent_fee)
    }

    /// Whether this pool has a buyback fund (partial migration > 0).
    pub fn has_buyback(&self) -> bool {
        self.partial_migration_pct > 0
    }

    /// Get graduation threshold in lamports.
    pub fn graduation_threshold(&self) -> u64 {
        self.graduation_tier.threshold_sol()
    }
}

/// ─── Agent Wallet ────────────────────────────────────────────────────────────
/// On-chain wallet for the agent. Holds SOL for autonomous operations.
/// Controlled by the Pulse program (PDA). The agent runtime (off-chain)
/// calls instructions on behalf of this wallet.
#[account]
pub struct AgentWallet {
    pub mint: Pubkey,
    pub agent_name: String,      // "Agent <TICKER>"
    pub total_earned_lamports: u64,
    pub total_spent_lamports: u64,
    pub claimable_amount: u64,    // currently claimable by agent (reset after claim)
    pub last_claim_at: i64,
    pub last_action_at: i64,
    pub created_at: i64,
    pub bump: u8,
}

impl AgentWallet {
    pub const ACCOUNT_SIZE: usize = 8
        + 32   // mint
        + 24   // agent_name (4 len + 20 chars)
        + 8    // total_earned_lamports
        + 8    // total_spent_lamports
        + 8    // claimable_amount
        + 8    // last_claim_at
        + 8    // last_action_at
        + 8    // created_at
        + 1;   // bump

    pub fn init(
        &mut self,
        mint: Pubkey,
        agent_name: String,
        bump: u8,
        now: i64,
    ) {
        self.mint = mint;
        self.agent_name = agent_name;
        self.total_earned_lamports = 0;
        self.total_spent_lamports = 0;
        self.claimable_amount = 0;
        self.last_claim_at = now;
        self.last_action_at = now;
        self.created_at = now;
        self.bump = bump;
    }

    /// Agent claims its accumulated SOL. Capped at 3-hour intervals.
    pub fn claim(&mut self, now: i64) -> Result<u64> {
        let elapsed = now.saturating_sub(self.last_claim_at);
        require!(elapsed >= 3 * 3600, BondingError::RateLimitExceeded);

        let amount = self.claimable_amount;
        require!(amount > 0, BondingError::NoRewardsToClaim);

        self.claimable_amount = 0;
        self.last_claim_at = now;
        self.last_action_at = now;
        self.total_earned_lamports = self.total_earned_lamports.saturating_add(amount);

        Ok(amount)
    }
}

/// ─── Vault Claim Tracker ─────────────────────────────────────────────────────
/// Tracks per-creator/per-agent vault claims to enforce 500K token / 24h cap.
#[account]
pub struct VaultClaimTracker {
    pub mint: Pubkey,
    pub claimer: Pubkey,         // creator or agent wallet
    pub tokens_claimed_24h: u64,
    pub last_claim_window_start: i64,
    pub total_claimed: u64,
    pub bump: u8,
}

impl VaultClaimTracker {
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;

    pub fn init(&mut self, mint: Pubkey, claimer: Pubkey, bump: u8) {
        self.mint = mint;
        self.claimer = claimer;
        self.tokens_claimed_24h = 0;
        self.last_claim_window_start = 0;
        self.total_claimed = 0;
        self.bump = bump;
    }

    /// Record a vault claim. Resets window if 24h have passed.
    pub fn record_claim(&mut self, amount: u64, now: i64) -> Result<()> {
        let elapsed = now.saturating_sub(self.last_claim_window_start);
        if elapsed >= VAULT_CLAIM_COOLDOWN_SECS {
            self.tokens_claimed_24h = 0;
            self.last_claim_window_start = now;
        }

        let new_total = self.tokens_claimed_24h.saturating_add(amount);
        require!(
            new_total <= MAX_VAULT_CLAIM_PER_24H,
            BondingError::DailyLimitExceeded
        );

        self.tokens_claimed_24h = new_total;
        self.total_claimed = self.total_claimed.saturating_add(amount);
        Ok(())
    }
}
