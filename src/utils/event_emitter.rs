use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// EventEmitter - Enhanced event emission utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Emit an event with additional context
pub fn emit_with_context<T: anchor_lang::Event + anchor_lang::AccountsClose<'info>>(
    event: &T,
    context: &str,
) {
    event.emit();
    msg!("Event: {} | Context: {}", std::any::type_name::<T>(), context);
}

/// Emit trade event with detailed information
pub fn emit_trade_event(
    event_type: &str,
    mint: &Pubkey,
    trader: &Pubkey,
    sol_amount: u64,
    token_amount: u64,
    fees_platform: u64,
    fees_creator: u64,
    price_impact_bps: u64,
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    real_sol_reserves: u64,
) {
    msg!(
        "TRADE: {} | Mint: {} | Trader: {} | SOL: {} | Tokens: {} | PlatformFee: {} | CreatorFee: {} | Impact: {}bps | VirtualSol: {} | VirtualTokens: {} | RealSol: {}",
        event_type,
        mint,
        trader,
        sol_amount,
        token_amount,
        fees_platform,
        fees_creator,
        price_impact_bps,
        virtual_sol_reserves,
        virtual_token_reserves,
        real_sol_reserves
    );
}

/// Emit pool state change event
pub fn emit_pool_state_change(
    mint: &Pubkey,
    old_state: &str,
    new_state: &str,
    reason: &str,
) {
    msg!(
        "POOL_STATE: {} | {} → {} | Reason: {}",
        mint,
        old_state,
        new_state,
        reason
    );
}

/// Emit upgrade event
pub fn emit_upgrade_event(
    version: u32,
    upgrade_type: &str,
    scheduled_at: i64,
    deadline: i64,
) {
    msg!(
        "UPGRADE: Version {} | Type: {} | Scheduled: {} | Deadline: {}",
        version,
        upgrade_type,
        scheduled_at,
        deadline
    );
}

/// Emit compliance event
pub fn emit_compliance_event(
    mint: &Pubkey,
    action: &str,
    wallet: &Pubkey,
    status: &str,
    risk_level: u8,
) {
    msg!(
        "COMPLIANCE: {} | Wallet: {} | Action: {} | Status: {} | Risk: {}",
        mint,
        wallet,
        action,
        status,
        risk_level
    );
}

/// Emit metrics event
pub fn emit_metrics_event(
    mint: &Pubkey,
    total_volume: u64,
    unique_traders: u64,
    total_fees: u64,
    health_score: u16,
) {
    msg!(
        "METRICS: {} | Volume: {} | Traders: {} | Fees: {} | Health: {}%",
        mint,
        total_volume,
        unique_traders,
        total_fees,
        health_score
    );
}

/// Emit audit log event
pub fn emit_audit_log(
    mint: &Pubkey,
    log_type: &str,
    authority: &Pubkey,
    details: &str,
) {
    msg!(
        "AUDIT_LOG: {} | Type: {} | Authority: {} | Details: {}",
        mint,
        log_type,
        authority,
        details
    );
}

/// Emit feature flag change event
pub fn emit_feature_flag_event(
    mint: &Pubkey,
    feature: &str,
    old_value: bool,
    new_value: bool,
) {
    msg!(
        "FEATURE_FLAG: {} | {} | {} → {}",
        mint,
        feature,
        old_value,
        new_value
    );
}

/// Emit economic event
pub fn emit_economic_event(
    mint: &Pubkey,
    event_type: &str,
    value: u64,
    unit: &str,
    reason: &str,
) {
    msg!(
        "ECONOMIC: {} | {} | {} {} | Reason: {}",
        mint,
        event_type,
        value,
        unit,
        reason
    );
}
