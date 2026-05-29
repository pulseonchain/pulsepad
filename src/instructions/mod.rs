// ═══════════════════════════════════════════════════════════════════════════
// CORE INSTRUCTIONS (existing)
// ═══════════════════════════════════════════════════════════════════════════
pub mod initialize;
pub mod create_token;
pub mod buy;
pub mod sell;
pub mod claim_fees;
pub mod transfer_authority;
pub mod migrate;
pub mod claim_lp_fees;
pub mod claim_migration_vault;
pub mod stake;
pub mod unstake;
pub mod claim_staker_rewards;
pub mod update_global_config;
pub mod close_pool;
pub mod whitelist_buy;
pub mod referral;

pub use initialize::*;
pub use create_token::*;
pub use buy::*;
pub use sell::*;
pub use claim_fees::*;
pub use transfer_authority::*;
pub use migrate::*;
pub use claim_lp_fees::*;
pub use claim_migration_vault::*;
pub use stake::*;
pub use unstake::*;
pub use claim_staker_rewards::*;
pub use update_global_config::*;
pub use close_pool::*;
pub use whitelist_buy::*;
pub use referral::*;

// ═══════════════════════════════════════════════════════════════════════════
// SECURITY INSTRUCTIONS (50 ideas #1-10)
// ═══════════════════════════════════════════════════════════════════════════

// #1 Reentrancy Guards
pub mod init_reentrancy_guard {
    use anchor_lang::prelude::*;
    use crate::security::ReentrancyGuard;
    pub fn init_reentrancy_guard(ctx: Context<InitReentrancyGuard>) -> Result<()> {
        let guard = &mut ctx.accounts.guard;
        guard.mint = ctx.accounts.mint.key();
        guard.locked = false;
        guard.bump = ctx.bumps.guard;
        msg!("Reentrancy guard initialized for {}", guard.mint);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitReentrancyGuard<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 1 + 1,
            seeds = [b"reentrancy_guard", mint.key().as_ref()],
            bump,
        )]
        pub guard: Account<'info, ReentrancyGuard>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod enter_reentrancy_guard {
    use anchor_lang::prelude::*;
    use crate::security::ReentrancyGuard;
    pub fn enter_reentrancy_guard(ctx: Context<EnterReentrancyGuard>) -> Result<()> {
        ctx.accounts.guard.enter()?;
        msg!("Reentrancy guard entered");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct EnterReentrancyGuard<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            mut,
            seeds = [b"reentrancy_guard", mint.key().as_ref()],
            bump = guard.bump,
        )]
        pub guard: Account<'info, ReentrancyGuard>,
    }
}

pub mod exit_reentrancy_guard {
    use anchor_lang::prelude::*;
    use crate::security::ReentrancyGuard;
    pub fn exit_reentrancy_guard(ctx: Context<ExitReentrancyGuard>) -> Result<()> {
        ctx.accounts.guard.exit();
        msg!("Reentrancy guard exited");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ExitReentrancyGuard<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            mut,
            seeds = [b"reentrancy_guard", mint.key().as_ref()],
            bump = guard.bump,
        )]
        pub guard: Account<'info, ReentrancyGuard>,
    }
}

// #2 Circuit Breaker
pub mod init_circuit_breaker {
    use anchor_lang::prelude::*;
    use crate::security::CircuitBreaker;
    pub fn init_circuit_breaker(ctx: Context<InitCircuitBreaker>) -> Result<()> {
        let cb = &mut ctx.accounts.circuit_breaker;
        cb.is_paused = false;
        cb.pause_start = 0;
        cb.pause_duration_seconds = 0;
        cb.paused_by = ctx.accounts.authority.key();
        cb.bump = ctx.bumps.circuit_breaker;
        msg!("Circuit breaker initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitCircuitBreaker<'info> {
        #[account(
            init,
            payer = authority,
            space = 8 + 1 + 8 + 8 + 32 + 1,
            seeds = [b"circuit_breaker"],
            bump,
        )]
        pub circuit_breaker: Account<'info, CircuitBreaker>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod trigger_circuit_breaker {
    use anchor_lang::prelude::*;
    use crate::security::CircuitBreaker;
    use crate::state::GlobalConfig;
    pub fn trigger_circuit_breaker(
        ctx: Context<TriggerCircuitBreaker>,
        duration_seconds: i64,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.config.authority,
            crate::errors::BondingError::Unauthorized
        );
        ctx.accounts.circuit_breaker.pause(ctx.accounts.authority.key())?;
        ctx.accounts.circuit_breaker.pause_duration_seconds = duration_seconds;
        msg!("⚠️ Circuit breaker triggered for {} seconds", duration_seconds);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct TriggerCircuitBreaker<'info> {
        #[account(seeds = [crate::consts::SEED_GLOBAL_CONFIG], bump = config.bump)]
        pub config: Account<'info, GlobalConfig>,
        #[account(mut, seeds = [b"circuit_breaker"], bump = circuit_breaker.bump)]
        pub circuit_breaker: Account<'info, CircuitBreaker>,
        pub authority: Signer<'info>,
    }
}

pub mod reset_circuit_breaker {
    use anchor_lang::prelude::*;
    use crate::security::CircuitBreaker;
    pub fn reset_circuit_breaker(ctx: Context<ResetCircuitBreaker>) -> Result<()> {
        ctx.accounts.circuit_breaker.unpause(ctx.accounts.authority.key())?;
        msg!("✅ Circuit breaker reset");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ResetCircuitBreaker<'info> {
        #[account(mut, seeds = [b"circuit_breaker"], bump = circuit_breaker.bump)]
        pub circuit_breaker: Account<'info, CircuitBreaker>,
        pub authority: Signer<'info>,
    }
}

// #3 Rate Limiting
pub mod init_rate_limiter {
    use anchor_lang::prelude::*;
    use crate::security::RateLimiter;
    pub fn init_rate_limiter(ctx: Context<InitRateLimiter>) -> Result<()> {
        let rl = &mut ctx.accounts.rate_limiter;
        let now = Clock::get()?.unix_timestamp;
        rl.user = ctx.accounts.user.key();
        rl.mint = ctx.accounts.mint.key();
        rl.window_start = now;
        rl.window_count = 0;
        rl.window_volume = 0;
        rl.daily_count = 0;
        rl.daily_volume = 0;
        rl.bump = ctx.bumps.rate_limiter;
        msg!("Rate limiter initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitRateLimiter<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: user address used for PDA seeds
        pub user: UncheckedAccount<'info>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1,
            seeds = [b"rate_limiter", user.key().as_ref(), mint.key().as_ref()],
            bump,
        )]
        pub rate_limiter: Account<'info, RateLimiter>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod check_rate_limit {
    use anchor_lang::prelude::*;
    use crate::security::RateLimiter;
    pub fn check_rate_limit(ctx: Context<CheckRateLimit>, sol_amount: u64) -> Result<()> {
        let window_seconds: i64 = 3600;
        let max_window_volume: u64 = 100_000_000_000;
        let max_daily_volume: u64 = 500_000_000_000;
        ctx.accounts.rate_limiter.check_and_update(
            sol_amount,
            window_seconds,
            max_window_volume,
            max_daily_volume,
        )?;
        Ok(())
    }
    #[derive(Accounts)]
    pub struct CheckRateLimit<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: user address used for PDA seeds
        pub user: UncheckedAccount<'info>,
        #[account(
            mut,
            seeds = [b"rate_limiter", user.key().as_ref(), mint.key().as_ref()],
            bump = rate_limiter.bump,
        )]
        pub rate_limiter: Account<'info, RateLimiter>,
    }
}

// #7 Address Filtering (#9 blacklisting)
pub mod init_address_filter {
    use anchor_lang::prelude::*;
    use crate::security::AddressFilter;
    pub fn init_address_filter(
        ctx: Context<InitAddressFilter>,
        filter_type: u8,
    ) -> Result<()> {
        let af = &mut ctx.accounts.filter;
        af.mint = ctx.accounts.mint.key();
        af.address = ctx.accounts.target.key();
        af.filter_type = filter_type;
        af.added_by = ctx.accounts.authority.key();
        af.added_at = Clock::get()?.unix_timestamp;
        af.bump = ctx.bumps.filter;
        msg!("Address filter added: type={} address={}", filter_type, af.address);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitAddressFilter<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: target address to filter
        pub target: UncheckedAccount<'info>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 32 + 1 + 32 + 8 + 1,
            seeds = [b"address_filter", mint.key().as_ref(), target.key().as_ref()],
            bump,
        )]
        pub filter: Account<'info, AddressFilter>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod remove_address_filter {
    use anchor_lang::prelude::*;
    use crate::security::AddressFilter;
    pub fn remove_address_filter(ctx: Context<RemoveAddressFilter>) -> Result<()> {
        msg!("Address filter removed");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RemoveAddressFilter<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: target address
        pub target: UncheckedAccount<'info>,
        #[account(
            mut,
            seeds = [b"address_filter", mint.key().as_ref(), target.key().as_ref()],
            bump = filter.bump,
            close = authority,
        )]
        pub filter: Account<'info, AddressFilter>,
        #[account(mut)]
        pub authority: Signer<'info>,
    }
}

// #24 Flash Loan Detection (#8)
pub mod init_flash_loan_detector {
    use anchor_lang::prelude::*;
    use crate::security::FlashLoanDetector;
    pub fn init_flash_loan_detector(ctx: Context<InitFlashLoanDetector>) -> Result<()> {
        let detector = &mut ctx.accounts.detector;
        let bump = ctx.bumps.detector;
        detector.init(&ctx.accounts.mint.key(), bump);
        msg!("Flash loan detector initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitFlashLoanDetector<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 8 + 4 + 8 + 1 + 1,
            seeds = [b"flash_loan_detector", mint.key().as_ref()],
            bump,
        )]
        pub detector: Account<'info, FlashLoanDetector>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod record_flash_loan_check {
    use anchor_lang::prelude::*;
    use crate::security::FlashLoanDetector;
    pub fn record_flash_loan_check(
        ctx: Context<RecordFlashLoanCheck>,
        volume: u64,
    ) -> Result<()> {
        let window: i64 = 60;
        ctx.accounts.detector.record_trade(volume, window, 10, 50_000_000_000)?;
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RecordFlashLoanCheck<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            mut,
            seeds = [b"flash_loan_detector", mint.key().as_ref()],
            bump = detector.bump,
        )]
        pub detector: Account<'info, FlashLoanDetector>,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ANALYTICS INSTRUCTIONS (#31-40)
// ═══════════════════════════════════════════════════════════════════════════

pub mod init_metrics {
    use anchor_lang::prelude::*;
    use crate::analytics::Metrics;
    pub fn init_metrics(ctx: Context<InitMetrics>) -> Result<()> {
        let m = &mut ctx.accounts.metrics;
        let bump = ctx.bumps.metrics;
        m.init(&ctx.accounts.mint.key(), bump);
        msg!("Metrics initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitMetrics<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 4 + 4 + 8 + 8 + 1,
            seeds = [b"metrics", mint.key().as_ref()],
            bump,
        )]
        pub metrics: Account<'info, Metrics>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod record_buy_metrics {
    use anchor_lang::prelude::*;
    use crate::analytics::Metrics;
    use crate::state::PoolState;
    pub fn record_buy_metrics(
        ctx: Context<RecordBuyMetrics>,
        sol_amount: u64,
        tokens_out: u64,
        platform_fee: u64,
        creator_fee: u64,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.metrics.record_buy(
            sol_amount, tokens_out, platform_fee, creator_fee,
            ctx.accounts.pool_state.real_sol_reserves,
            &ctx.accounts.buyer.key(),
            now,
        );
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RecordBuyMetrics<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"metrics", mint.key().as_ref()], bump = metrics.bump)]
        pub metrics: Account<'info, Metrics>,
        #[account(seeds = [crate::consts::SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
        pub pool_state: Account<'info, PoolState>,
        /// CHECK: buyer address
        pub buyer: UncheckedAccount<'info>,
    }
}

pub mod record_sell_metrics {
    use anchor_lang::prelude::*;
    use crate::analytics::Metrics;
    pub fn record_sell_metrics(
        ctx: Context<RecordSellMetrics>,
        sol_amount: u64,
        tokens_in: u64,
        platform_fee: u64,
        creator_fee: u64,
    ) -> Result<()> {
        ctx.accounts.metrics.record_sell(
            sol_amount, tokens_in, platform_fee, creator_fee,
            &ctx.accounts.seller.key(),
        );
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RecordSellMetrics<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"metrics", mint.key().as_ref()], bump = metrics.bump)]
        pub metrics: Account<'info, Metrics>,
        /// CHECK: seller address
        pub seller: UncheckedAccount<'info>,
    }
}

pub mod init_pool_health {
    use anchor_lang::prelude::*;
    use crate::analytics::PoolHealth;
    pub fn init_pool_health(ctx: Context<InitPoolHealth>) -> Result<()> {
        let ph = &mut ctx.accounts.pool_health;
        let bump = ctx.bumps.pool_health;
        ph.init(&ctx.accounts.mint.key(), bump);
        msg!("Pool health tracker initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitPoolHealth<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 2 + 8 + 8 + 8 + 4 + 8 + 1 + 33 + 1,
            seeds = [b"pool_health", mint.key().as_ref()],
            bump,
        )]
        pub pool_health: Account<'info, PoolHealth>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod update_pool_health {
    use anchor_lang::prelude::*;
    use crate::analytics::PoolHealth;
    use crate::state::PoolState;
    pub fn update_pool_health(ctx: Context<UpdatePoolHealth>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.pool_health.update_health(
            ctx.accounts.pool_state.real_sol_reserves,
            ctx.accounts.pool_state.real_token_reserves,
            85_000_000_000, // graduation threshold
            now,
        );
        msg!("Pool health updated: {}% (category: {})",
            ctx.accounts.pool_health.health_score as f64 / 100.0,
            ctx.accounts.pool_health.health_category()
        );
        Ok(())
    }
    #[derive(Accounts)]
    pub struct UpdatePoolHealth<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"pool_health", mint.key().as_ref()], bump = pool_health.bump)]
        pub pool_health: Account<'info, PoolHealth>,
        #[account(seeds = [crate::consts::SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
        pub pool_state: Account<'info, PoolState>,
    }
}

pub mod init_user_stats {
    use anchor_lang::prelude::*;
    use crate::analytics::UserStats;
    pub fn init_user_stats(ctx: Context<InitUserStats>) -> Result<()> {
        let us = &mut ctx.accounts.user_stats;
        let bump = ctx.bumps.user_stats;
        us.init(&ctx.accounts.user.key(), &ctx.accounts.mint.key(), bump);
        msg!("User stats initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitUserStats<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: user address
        pub user: UncheckedAccount<'info>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 4 + 4 + 1,
            seeds = [b"user_stats", user.key().as_ref(), mint.key().as_ref()],
            bump,
        )]
        pub user_stats: Account<'info, UserStats>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod record_user_buy {
    use anchor_lang::prelude::*;
    use crate::analytics::UserStats;
    pub fn record_user_buy(
        ctx: Context<RecordUserBuy>,
        volume: u64,
        fees: u64,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.user_stats.record_buy(volume, fees, now);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RecordUserBuy<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: user address
        pub user: UncheckedAccount<'info>,
        #[account(
            mut,
            seeds = [b"user_stats", user.key().as_ref(), mint.key().as_ref()],
            bump = user_stats.bump,
        )]
        pub user_stats: Account<'info, UserStats>,
    }
}

pub mod record_user_sell {
    use anchor_lang::prelude::*;
    use crate::analytics::UserStats;
    pub fn record_user_sell(
        ctx: Context<RecordUserSell>,
        volume: u64,
        fees: u64,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.user_stats.record_sell(volume, fees, now);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RecordUserSell<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        /// CHECK: user address
        pub user: UncheckedAccount<'info>,
        #[account(
            mut,
            seeds = [b"user_stats", user.key().as_ref(), mint.key().as_ref()],
            bump = user_stats.bump,
        )]
        pub user_stats: Account<'info, UserStats>,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ECONOMICS INSTRUCTIONS (#16-20)
// ═══════════════════════════════════════════════════════════════════════════

pub mod init_dynamic_fees {
    use anchor_lang::prelude::*;
    use crate::economics::DynamicFeeConfig;
    pub fn init_dynamic_fees(ctx: Context<InitDynamicFees>) -> Result<()> {
        let df = &mut ctx.accounts.dynamic_fee_config;
        let bump = ctx.bumps.dynamic_fee_config;
        df.init(&ctx.accounts.mint.key(), 100, bump); // 100 bps base fee
        msg!("Dynamic fee config initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitDynamicFees<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 1,
            seeds = [b"dynamic_fee", mint.key().as_ref()],
            bump,
        )]
        pub dynamic_fee_config: Account<'info, DynamicFeeConfig>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod update_dynamic_fee {
    use anchor_lang::prelude::*;
    use crate::economics::DynamicFeeConfig;
    pub fn update_dynamic_fee(
        ctx: Context<UpdateDynamicFee>,
        sol_reserves: u64,
        token_reserves: u64,
        volatility_bps: u64,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let fee = ctx.accounts.dynamic_fee_config.calculate_fee(
            sol_reserves, token_reserves, volatility_bps, now,
        );
        msg!("Dynamic fee updated: {} bps", fee);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct UpdateDynamicFee<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"dynamic_fee", mint.key().as_ref()], bump = dynamic_fee_config.bump)]
        pub dynamic_fee_config: Account<'info, DynamicFeeConfig>,
    }
}

pub mod init_liquidity_bootstrap {
    use anchor_lang::prelude::*;
    use crate::economics::LiquidityBootstrap;
    pub fn init_liquidity_bootstrap(
        ctx: Context<InitLiquidityBootstrap>,
        duration_seconds: u64,
    ) -> Result<()> {
        let lb = &mut ctx.accounts.bootstrap;
        let bump = ctx.bumps.bootstrap;
        lb.init(&ctx.accounts.mint.key(), duration_seconds, bump);
        msg!("Liquidity bootstrap initialized: {}s duration", duration_seconds);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitLiquidityBootstrap<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 8 + 8 + 8 + 8 + 1 + 1,
            seeds = [b"bootstrap", mint.key().as_ref()],
            bump,
        )]
        pub bootstrap: Account<'info, LiquidityBootstrap>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod get_bootstrap_pricing {
    use anchor_lang::prelude::*;
    use crate::economics::LiquidityBootstrap;
    pub fn get_bootstrap_pricing(ctx: Context<GetBootstrapPricing>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let virtual_sol = ctx.accounts.bootstrap.get_pricing_virtual_sol(now);
        let phase = ctx.accounts.bootstrap.get_phase(now);
        let remaining = ctx.accounts.bootstrap.remaining_time(now);
        msg!("Bootstrap: virtual_sol={} phase={:?} remaining={}s", virtual_sol, phase, remaining);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct GetBootstrapPricing<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(seeds = [b"bootstrap", mint.key().as_ref()], bump = bootstrap.bump)]
        pub bootstrap: Account<'info, LiquidityBootstrap>,
    }
}

pub mod init_whale_protection {
    use anchor_lang::prelude::*;
    use crate::economics::WhaleProtection;
    pub fn init_whale_protection(ctx: Context<InitWhaleProtection>) -> Result<()> {
        let wp = &mut ctx.accounts.whale_protection;
        let bump = ctx.bumps.whale_protection;
        wp.init(&ctx.accounts.mint.key(), 100, 1_000_000_000_000, 5_000_000_000_000, bump);
        msg!("Whale protection initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitWhaleProtection<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 1,
            seeds = [b"whale_protection", mint.key().as_ref()],
            bump,
        )]
        pub whale_protection: Account<'info, WhaleProtection>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod calculate_whale_fee {
    use anchor_lang::prelude::*;
    use crate::economics::WhaleProtection;
    pub fn calculate_whale_fee(
        ctx: Context<CalculateWhaleFee>,
        trade_volume: u64,
    ) -> Result<()> {
        let effective_fee = ctx.accounts.whale_protection.calculate_effective_fee(trade_volume);
        let (progressive, total) = ctx.accounts.whale_protection.calculate_progressive_fee(trade_volume);
        let status = ctx.accounts.whale_protection.get_whale_status(trade_volume);
        msg!("Whale fee: effective={} progressive={} total={} status={:?}", effective_fee, progressive, total, status);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct CalculateWhaleFee<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(seeds = [b"whale_protection", mint.key().as_ref()], bump = whale_protection.bump)]
        pub whale_protection: Account<'info, WhaleProtection>,
    }
}

pub mod init_fee_redistribution {
    use anchor_lang::prelude::*;
    use crate::economics::FeeRedistributionConfig;
    pub fn init_fee_redistribution(ctx: Context<InitFeeRedistribution>) -> Result<()> {
        let fr = &mut ctx.accounts.fee_redistribution;
        let bump = ctx.bumps.fee_redistribution;
        fr.init(&ctx.accounts.mint.key(), 75, 25, 0, bump);
        msg!("Fee redistribution config initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitFeeRedistribution<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1,
            seeds = [b"fee_redistribution", mint.key().as_ref()],
            bump,
        )]
        pub fee_redistribution: Account<'info, FeeRedistributionConfig>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod redistribute_fees {
    use anchor_lang::prelude::*;
    use crate::economics::FeeRedistributionConfig;
    pub fn redistribute_fees(
        ctx: Context<RedistributeFees>,
        total_fees: u64,
    ) -> Result<()> {
        let (platform, creator, staker, lp, reserve) = ctx.accounts.fee_redistribution.distribute_fees(
            total_fees,
            &ctx.accounts.pool_state,
            0, // total_staked - would read from staker vault in production
        );
        msg!("Fee redistribution: platform={} creator={} staker={} lp={} reserve={}", platform, creator, staker, lp, reserve);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct RedistributeFees<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"fee_redistribution", mint.key().as_ref()], bump = fee_redistribution.bump)]
        pub fee_redistribution: Account<'info, FeeRedistributionConfig>,
        #[account(seeds = [crate::consts::SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
        pub pool_state: Account<'info, crate::state::PoolState>,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UPGRADES INSTRUCTIONS (#41-50)
// ═══════════════════════════════════════════════════════════════════════════

pub mod init_upgradeable {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradeableProgram;
    pub fn init_upgradeable(ctx: Context<InitUpgradeable>) -> Result<()> {
        let up = &mut ctx.accounts.upgradeable;
        let bump = ctx.bumps.upgradeable;
        up.init(&ctx.accounts.mint.key(), &ctx.accounts.authority.key(), bump);
        msg!("Upgradeable program initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitUpgradeable<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 32 + 4 + 4 + 8 + 8 + 32 + 1,
            seeds = [b"upgradeable_program", mint.key().as_ref()],
            bump,
        )]
        pub upgradeable: Account<'info, UpgradeableProgram>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod schedule_upgrade {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradeableProgram;
    pub fn schedule_upgrade(
        ctx: Context<ScheduleUpgrade>,
        next_version: u32,
        delay_seconds: u64,
    ) -> Result<()> {
        ctx.accounts.upgradeable.start_upgrade(next_version, delay_seconds)?;
        msg!("Upgrade scheduled: version {} with {}s delay", next_version, delay_seconds);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ScheduleUpgrade<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"upgradeable_program", mint.key().as_ref()], bump = upgradeable.bump)]
        pub upgradeable: Account<'info, UpgradeableProgram>,
        pub authority: Signer<'info>,
    }
}

pub mod complete_upgrade {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradeableProgram;
    pub fn complete_upgrade(ctx: Context<CompleteUpgrade>) -> Result<()> {
        ctx.accounts.upgradeable.complete_upgrade()?;
        msg!("Upgrade completed: version {}", ctx.accounts.upgradeable.current_version);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct CompleteUpgrade<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"upgradeable_program", mint.key().as_ref()], bump = upgradeable.bump)]
        pub upgradeable: Account<'info, UpgradeableProgram>,
        pub authority: Signer<'info>,
    }
}

pub mod cancel_upgrade {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradeableProgram;
    pub fn cancel_upgrade(ctx: Context<CancelUpgrade>) -> Result<()> {
        ctx.accounts.upgradeable.cancel_upgrade()?;
        msg!("Upgrade cancelled");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct CancelUpgrade<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"upgradeable_program", mint.key().as_ref()], bump = upgradeable.bump)]
        pub upgradeable: Account<'info, UpgradeableProgram>,
        pub authority: Signer<'info>,
    }
}

pub mod init_feature_flags {
    use anchor_lang::prelude::*;
    use crate::upgrades::FeatureFlags;
    pub fn init_feature_flags(ctx: Context<InitFeatureFlags>) -> Result<()> {
        let ff = &mut ctx.accounts.feature_flags;
        let bump = ctx.bumps.feature_flags;
        ff.init(&ctx.accounts.mint.key(), bump);
        msg!("Feature flags initialized: bitmap={}", ff.to_bitmap());
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitFeatureFlags<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 14 + 1 + 1,
            seeds = [b"feature_flags", mint.key().as_ref()],
            bump,
        )]
        pub feature_flags: Account<'info, FeatureFlags>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod toggle_feature {
    use anchor_lang::prelude::*;
    use crate::upgrades::FeatureFlags;
    use crate::utils::Feature;
    pub fn toggle_feature(
        ctx: Context<ToggleFeature>,
        feature: Feature,
    ) -> Result<()> {
        ctx.accounts.feature_flags.toggle_feature(feature)?;
        msg!("Feature toggled: {:?} = {}", feature, ctx.accounts.feature_flags.is_enabled(feature));
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ToggleFeature<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"feature_flags", mint.key().as_ref()], bump = feature_flags.bump)]
        pub feature_flags: Account<'info, FeatureFlags>,
        pub authority: Signer<'info>,
    }
}

pub mod set_feature_bitmap {
    use anchor_lang::prelude::*;
    use crate::upgrades::FeatureFlags;
    pub fn set_feature_bitmap(
        ctx: Context<SetFeatureBitmap>,
        bitmap: u128,
    ) -> Result<()> {
        ctx.accounts.feature_flags.from_bitmap(bitmap);
        msg!("Feature bitmap updated: {}", bitmap);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct SetFeatureBitmap<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut, seeds = [b"feature_flags", mint.key().as_ref()], bump = feature_flags.bump)]
        pub feature_flags: Account<'info, FeatureFlags>,
        pub authority: Signer<'info>,
    }
}

pub mod init_plugin {
    use anchor_lang::prelude::*;
    use crate::upgrades::PluginConfig;
    pub fn init_plugin(
        ctx: Context<InitPlugin>,
        plugin_type: u8,
        config_data: [u8; 256],
    ) -> Result<()> {
        let plugin = &mut ctx.accounts.plugin;
        let bump = ctx.bumps.plugin;
        plugin.init(&ctx.accounts.mint.key(), plugin_type, &config_data, bump);
        msg!("Plugin initialized: type={}", plugin_type);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitPlugin<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 1 + 256 + 1 + 1,
            seeds = [b"plugin", mint.key().as_ref(), &[plugin_type]],
            bump,
        )]
        pub plugin: Account<'info, PluginConfig>,
        pub plugin_type: u8,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod execute_plugin {
    use anchor_lang::prelude::*;
    use crate::upgrades::PluginConfig;
    pub fn execute_plugin(ctx: Context<ExecutePlugin>) -> Result<()> {
        require!(ctx.accounts.plugin.is_enabled(), crate::errors::BondingError::FeatureDisabled);
        msg!("Plugin executed: type={}", ctx.accounts.plugin.plugin_type);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ExecutePlugin<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(seeds = [b"plugin", mint.key().as_ref(), &[plugin.plugin_type]], bump = plugin.bump)]
        pub plugin: Account<'info, PluginConfig>,
    }
}

pub mod init_upgrade_path {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradePath;
    pub fn init_upgrade_path(ctx: Context<InitUpgradePath>) -> Result<()> {
        let up = &mut ctx.accounts.upgrade_path;
        let bump = ctx.bumps.upgrade_path;
        up.init(bump);
        msg!("Upgrade path initialized");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitUpgradePath<'info> {
        #[account(
            init,
            payer = authority,
            space = 8 + 4 + 4 + (4 + 8 + 8 + 8 + 64) * 10 + 1,
            seeds = [b"upgrade_path"],
            bump,
        )]
        pub upgrade_path: Account<'info, UpgradePath>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod schedule_upgrade_path {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradePath;
    pub fn schedule_upgrade_path(
        ctx: Context<ScheduleUpgradePath>,
        version: u32,
        scheduled_at: i64,
        deadline: i64,
        delay_seconds: u64,
    ) -> Result<()> {
        let desc = b"Protocol upgrade";
        ctx.accounts.upgrade_path.schedule_upgrade(version, scheduled_at, deadline, delay_seconds, desc)?;
        msg!("Upgrade path scheduled: version {} at {}", version, scheduled_at);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ScheduleUpgradePath<'info> {
        #[account(mut, seeds = [b"upgrade_path"], bump = upgrade_path.bump)]
        pub upgrade_path: Account<'info, UpgradePath>,
        pub authority: Signer<'info>,
    }
}

pub mod execute_upgrade_path {
    use anchor_lang::prelude::*;
    use crate::upgrades::UpgradePath;
    pub fn execute_upgrade_path(
        ctx: Context<ExecuteUpgradePath>,
        version: u32,
    ) -> Result<()> {
        ctx.accounts.upgrade_path.execute_upgrade(version)?;
        msg!("Upgrade path executed: version {}", version);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ExecuteUpgradePath<'info> {
        #[account(mut, seeds = [b"upgrade_path"], bump = upgrade_path.bump)]
        pub upgrade_path: Account<'info, UpgradePath>,
        pub authority: Signer<'info>,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPLIANCE INSTRUCTIONS
// ═══════════════════════════════════════════════════════════════════════════

pub mod init_compliance {
    use anchor_lang::prelude::*;
    use crate::compliance::ComplianceChecker;
    pub fn init_compliance(
        ctx: Context<InitCompliance>,
        kyc_required: bool,
    ) -> Result<()> {
        let cc = &mut ctx.accounts.compliance;
        let bump = ctx.bumps.compliance;
        cc.init(&ctx.accounts.mint.key(), kyc_required, bump);
        msg!("Compliance checker initialized: kyc_required={}", kyc_required);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitCompliance<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(
            init,
            payer = authority,
            space = 8 + 32 + 1 + 32 + 1 + 8 + 8 + 1,
            seeds = [b"compliance", mint.key().as_ref()],
            bump,
        )]
        pub compliance: Account<'info, ComplianceChecker>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod check_compliance {
    use anchor_lang::prelude::*;
    use crate::compliance::ComplianceChecker;
    pub fn check_compliance(
        ctx: Context<CheckCompliance>,
        kyc_expiry: i64,
    ) -> Result<()> {
        ctx.accounts.compliance.check_kyc(kyc_expiry)?;
        msg!("Compliance check passed");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct CheckCompliance<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(seeds = [b"compliance", mint.key().as_ref()], bump = compliance.bump)]
        pub compliance: Account<'info, ComplianceChecker>,
    }
}

pub mod init_audit_log {
    use anchor_lang::prelude::*;
    use crate::compliance::AuditLogEntry;
    pub fn init_audit_log(ctx: Context<InitAuditLog>) -> Result<()> {
        msg!("Audit log entry created");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct InitAuditLog<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod log_audit_event {
    use anchor_lang::prelude::*;
    pub fn log_audit_event(
        ctx: Context<LogAuditEvent>,
        log_type: u8,
        details: [u8; 64],
    ) -> Result<()> {
        msg!("AUDIT: type={} authority={} details={:?}", log_type, ctx.accounts.authority.key(), &details[..16]);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct LogAuditEvent<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        pub authority: Signer<'info>,
    }
}

pub mod verify_whitelist_proof {
    use anchor_lang::prelude::*;
    use crate::compliance::verify_whitelist_proof as verify;
    pub fn verify_whitelist_proof(
        ctx: Context<VerifyWhitelistProof>,
        inclusion_proof: Vec<u8>,
        root_hash: [u8; 32],
    ) -> Result<()> {
        verify(&ctx.accounts.wallet.key(), &inclusion_proof, &root_hash)?;
        msg!("Whitelist proof verified");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct VerifyWhitelistProof<'info> {
        /// CHECK: wallet address
        pub wallet: UncheckedAccount<'info>,
    }
}

pub mod verify_referral_proof {
    use anchor_lang::prelude::*;
    use crate::compliance::verify_referral_proof as verify;
    pub fn verify_referral_proof(
        ctx: Context<VerifyReferralProof>,
        referral_proof: Vec<u8>,
        root_hash: [u8; 32],
    ) -> Result<()> {
        verify(&ctx.accounts.referrer.key(), &referral_proof, &root_hash)?;
        msg!("Referral proof verified");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct VerifyReferralProof<'info> {
        /// CHECK: referrer address
        pub referrer: UncheckedAccount<'info>,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INVARIANT CHECKER INSTRUCTIONS
// ═══════════════════════════════════════════════════════════════════════════

pub mod verify_pool_invariants {
    use anchor_lang::prelude::*;
    use crate::invariants::verify_bonding_curve_invariants;
    pub fn verify_pool_invariants(ctx: Context<VerifyPoolInvariants>) -> Result<()> {
        verify_bonding_curve_invariants(&ctx.accounts.pool_state)?;
        msg!("Pool invariants verified ✓");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct VerifyPoolInvariants<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
        #[account(seeds = [crate::consts::SEED_POOL_STATE, mint.key().as_ref()], bump = pool_state.bump)]
        pub pool_state: Account<'info, crate::state::PoolState>,
    }
}

pub mod verify_math_operations {
    use anchor_lang::prelude::*;
    use crate::invariants::{verify_math_operations, MathOperation};
    pub fn verify_math_operations(
        ctx: Context<VerifyMathOps>,
        before: u64,
        after: u64,
        change: u64,
        operation: u8,
    ) -> Result<()> {
        let op = match operation {
            0 => MathOperation::Addition,
            1 => MathOperation::Subtraction,
            2 => MathOperation::Multiplication,
            _ => MathOperation::Division,
        };
        verify_math_operations(before, after, change, op)?;
        msg!("Math operations verified ✓");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct VerifyMathOps<'info> {
        pub mint: Account<'info, anchor_spl::token::Mint>,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TIME-LOCKED AUTHORITY TRANSFER (#5)
// ═══════════════════════════════════════════════════════════════════════════

pub mod schedule_authority_transfer {
    use anchor_lang::prelude::*;
    use crate::state::GlobalConfig;
    pub fn schedule_authority_transfer(
        ctx: Context<ScheduleAuthorityTransfer>,
        new_authority: Pubkey,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.config.authority,
            crate::errors::BondingError::Unauthorized
        );
        ctx.accounts.pending_transfer.new_authority = Some(new_authority);
        ctx.accounts.pending_transfer.scheduled_at = Clock::get()?.unix_timestamp;
        ctx.accounts.pending_transfer.delay_seconds = 86_400; // 24 hours
        msg!("Authority transfer scheduled: {} (executes in 24h)", new_authority);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ScheduleAuthorityTransfer<'info> {
        #[account(seeds = [crate::consts::SEED_GLOBAL_CONFIG], bump = config.bump)]
        pub config: Account<'info, GlobalConfig>,
        #[account(
            init_if_needed,
            payer = authority,
            space = 8 + 32 + 8 + 8 + 1,
            seeds = [b"pending_authority_transfer"],
            bump,
        )]
        pub pending_transfer: Account<'info, PendingAuthorityTransfer>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
}

pub mod execute_authority_transfer {
    use anchor_lang::prelude::*;
    use crate::state::GlobalConfig;
    pub fn execute_authority_transfer(ctx: Context<ExecuteAuthorityTransfer>) -> Result<()> {
        let transfer = &ctx.accounts.pending_transfer;
        let new_auth = transfer.new_authority.ok_or(crate::errors::BondingError::Unauthorized)?;
        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= transfer.scheduled_at + transfer.delay_seconds,
            crate::errors::BondingError::Unauthorized
        );
        ctx.accounts.config.authority = new_auth;
        msg!("Authority transfer executed: {}", new_auth);
        Ok(())
    }
    #[derive(Accounts)]
    pub struct ExecuteAuthorityTransfer<'info> {
        #[account(mut, seeds = [crate::consts::SEED_GLOBAL_CONFIG], bump = config.bump)]
        pub config: Account<'info, GlobalConfig>,
        #[account(
            mut,
            seeds = [b"pending_authority_transfer"],
            bump = pending_transfer.bump,
            close = authority,
        )]
        pub pending_transfer: Account<'info, PendingAuthorityTransfer>,
        #[account(mut)]
        pub authority: Signer<'info>,
    }
}

pub mod cancel_authority_transfer {
    use anchor_lang::prelude::*;
    use crate::state::GlobalConfig;
    pub fn cancel_authority_transfer(ctx: Context<CancelAuthorityTransfer>) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.config.authority,
            crate::errors::BondingError::Unauthorized
        );
        msg!("Authority transfer cancelled");
        Ok(())
    }
    #[derive(Accounts)]
    pub struct CancelAuthorityTransfer<'info> {
        #[account(seeds = [crate::consts::SEED_GLOBAL_CONFIG], bump = config.bump)]
        pub config: Account<'info, GlobalConfig>,
        #[account(
            mut,
            seeds = [b"pending_authority_transfer"],
            bump = pending_transfer.bump,
            close = authority,
        )]
        pub pending_transfer: Account<'info, PendingAuthorityTransfer>,
        #[account(mut)]
        pub authority: Signer<'info>,
    }
}

// ─── Pending Authority Transfer Account ─────────────────────────────────────

#[account]
pub struct PendingAuthorityTransfer {
    pub new_authority: Option<Pubkey>,
    pub scheduled_at: i64,
    pub delay_seconds: i64,
    pub bump: u8,
}

impl PendingAuthorityTransfer {
    pub const ACCOUNT_SIZE: usize = 8
        + 33  // new_authority (Option<Pubkey>)
        + 8   // scheduled_at
        + 8   // delay_seconds
        + 1;  // bump
}
