use anchor_lang::prelude::*;
use crate::state::PoolState;

// ─────────────────────────────────────────────────────────────────────────────
// InvariantChecker - Verifies protocol invariants before/after operations
// ─────────────────────────────────────────────────────────────────────────────

/// Verify bonding curve invariants
pub fn verify_bonding_curve_invariants(pool: &PoolState) -> Result<()> {
    // Invariant 1: Virtual reserves must be positive
    require!(
        pool.virtual_sol_reserves > 0,
        BondingError::InvalidInvariant
    );
    require!(
        pool.virtual_token_reserves > 0,
        BondingError::InvalidInvariant
    );

    // Invariant 2: Real reserves cannot exceed supply
    require!(
        pool.real_token_reserves <= 700_000_000_000_000u64, // BONDING_SUPPLY
        BondingError::InvalidInvariant
    );
    require!(
        pool.reserve_tokens_remaining <= 97_052_391_304_347u64, // RESERVE_SUPPLY
        BondingError::InvalidInvariant
    );

    // Invariant 3: Graduated flag consistency
    if pool.graduated {
        require!(
            pool.dex_pool.is_some(),
            BondingError::InvalidInvariant
        );
    }

    // Invariant 4: Product consistency check (approximate)
    let k_before = pool.virtual_sol_reserves as u128 * pool.virtual_token_reserves as u128;
    let k_after = pool.virtual_sol_reserves.saturating_add(1) as u128 
        * pool.virtual_token_reserves.saturating_sub(1) as u128;
    
    // After a buy, k should decrease (price impact)
    require!(
        k_after <= k_before,
        BondingError::InvalidInvariant
    );

    Ok(())
}

/// Verify math operations
pub fn verify_math_operations(
    before: u64,
    after: u64,
    change: u64,
    operation: MathOperation,
) -> Result<()> {
    match operation {
        MathOperation::Addition => {
            let expected = before.saturating_add(change);
            require!(
                after == expected,
                BondingError::MathVerificationFailed
            );
        }
        MathOperation::Subtraction => {
            let expected = before.saturating_sub(change);
            require!(
                after == expected,
                BondingError::MathVerificationFailed
            );
        }
        MathOperation::Multiplication => {
            require!(
                change == 0 || after / change == before,
                BondingError::MathVerificationFailed
            );
        }
        MathOperation::Division => {
            require!(
                after.saturating_mul(change) == before,
                BondingError::MathVerificationFailed
            );
        }
    }
    Ok(())
}

/// Math operations for verification
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MathOperation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

/// Verify address ownership
pub fn verify_account_owner(account: &AccountInfo<'_>, expected_owner: &Pubkey) -> Result<()> {
    require!(
        account.owner == *expected_owner,
        BondingError::InvalidAccountOwner
    );
    Ok(())
}

/// Verify account rent-exemption
pub fn verify_rent_exempt(account: &AccountInfo<'_>) -> Result<()> {
    let rent = Rent::get()?;
    let data_len = account.data_len();
    let minimum_balance = rent.minimum_balance(data_len);
    
    require!(
        account.lamports() >= minimum_balance,
        BondingError::AccountNotRentExempt
    );
    Ok(())
}

/// Verify price impact is within acceptable bounds
pub fn verify_price_impact(impact_bps: u64, max_impact_bps: u64) -> Result<()> {
    require!(
        impact_bps <= max_impact_bps,
        BondingError::PriceImpactTooHigh
    );
    Ok(())
}

/// Verify token supply invariants
pub fn verify_token_supply(
    total_issued: u64,
    bonding_supply: u64,
    reserve_supply: u64,
    lp_supply: u64,
) -> Result<()> {
    let expected_total = bonding_supply + reserve_supply + lp_supply;
    require!(
        total_issued == expected_total,
        BondingError::TokenSupplyMismatch
    );
    Ok(())
}

/// Add invariant verification errors

