use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// MathVerifier - Additional math verification utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Verify multiplication doesn't overflow
pub fn verify_multiply(a: u64, b: u64, result: u64) -> Result<u64> {
    let computed = a.checked_mul(b).ok_or(BondingError::MathOverflow)?;
    require!(
        computed == result as u128,
        BondingError::MathVerificationFailed
    );
    Ok(result)
}

/// Verify division is exact
pub fn verify_division(dividend: u64, divisor: u64, quotient: u64) -> Result<u64> {
    require!(
        divisor > 0,
        BondingError::DivisionByZero
    );
    
    let computed = dividend / divisor;
    require!(
        computed == quotient,
        BondingError::MathVerificationFailed
    );
    Ok(computed)
}

/// Verify percentage calculation
pub fn verify_percentage(
    base: u64,
    percentage_bps: u64,
    expected_result: u64,
) -> Result<u64> {
    let result = base
        .checked_mul(percentage_bps)
        .ok_or(BondingError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(BondingError::MathOverflow)?;
    
    require!(
        result == expected_result,
        BondingError::MathVerificationFailed
    );
    Ok(result)
}

/// Verify fee split calculation
pub fn verify_fee_split(
    gross: u64,
    total_fee_bps: u64,
    platform_share_bps: u64,
    expected_total: u64,
    expected_platform: u64,
    expected_creator: u64,
) -> Result<()> {
    let total_fee = gross
        .checked_mul(total_fee_bps)
        .ok_or(BondingError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(BondingError::MathOverflow)?;
    
    let platform_fee = total_fee
        .checked_mul(platform_share_bps)
        .ok_or(BondingError::MathOverflow)?
        .checked_div(100)
        .ok_or(BondingError::MathOverflow)?;
    
    let creator_fee = total_fee.saturating_sub(platform_fee);
    
    require!(
        total_fee == expected_total,
        BondingError::MathVerificationFailed
    );
    require!(
        platform_fee == expected_platform,
        BondingError::MathVerificationFailed
    );
    require!(
        creator_fee == expected_creator,
        BondingError::MathVerificationFailed
    );
    
    // Verify split sums correctly
    require!(
        platform_fee.saturating_add(creator_fee) == total_fee,
        BondingError::MathVerificationFailed
    );
    
    Ok(())
}

/// Verify constant-product invariant
pub fn verify_constant_product(
    virtual_sol: u64,
    virtual_token: u64,
    net_sol: u64,
    tokens_out: u64,
) -> Result<()> {
    let vs = virtual_sol as u128;
    let vt = virtual_token as u128;
    let s = net_sol as u128;
    let t = tokens_out as u128;
    
    let k_before = vs.saturating_mul(vt);
    let k_after = vs.saturating_add(s).saturating_mul(vt.saturating_sub(t));
    
    // After a trade, k should decrease due to fees and price impact
    require!(
        k_after <= k_before,
        BondingError::ConstantProductViolation
    );
    
    Ok(())
}

/// Verify price calculation
pub fn verify_price(
    virtual_sol: u64,
    virtual_token: u64,
    expected_price_bps: u64,
) -> Result<u64> {
    let price = (virtual_sol as u128)
        .saturating_mul(1_000_000_000_000u128)  // 12 decimal places
        .checked_div(virtual_token as u128)
        .ok_or(BondingError::MathOverflow)?;
    
    let price_bps = price
        .checked_mul(10_000u128)
        .ok_or(BondingError::MathOverflow)?
        .checked_div(1_000_000_000_000u128)
        .ok_or(BondingError::MathOverflow)?;
    
    require!(
        price_bps == expected_price_bps as u128,
        BondingError::MathVerificationFailed
    );
    
    Ok(price_bps as u64)
}

/// Verify graduation threshold
pub fn verify_graduation_threshold(
    current_sol: u64,
    threshold: u64,
    will_graduate: bool,
) -> Result<()> {
    let actual_graduated = current_sol >= threshold;
    require!(
        actual_graduated == will_graduate,
        BondingError::GraduationVerificationFailed
    );
    Ok(())
}

/// Verify pool state consistency
pub fn verify_pool_consistency(
    real_sol: u64,
    real_token: u64,
    virtual_sol: u64,
    virtual_token: u64,
) -> Result<()> {
    // Virtual reserves should be >= real reserves
    require!(
        virtual_sol >= real_sol,
        BondingError::PoolConsistencyViolation
    );
    require!(
        virtual_token >= real_token,
        BondingError::PoolConsistencyViolation
    );
    
    Ok(())
}

/// Add math verification errors

