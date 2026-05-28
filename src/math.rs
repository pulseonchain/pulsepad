/// Pure bonding-curve math module.
///
/// All functions here are free of on-chain account dependencies — they operate
/// on raw u64/u128 values. This makes them:
///   1. Testable in isolation without Anchor test framework overhead
///   2. Usable off-chain in the SDK (Rust → WASM for TypeScript simulation)
///   3. Easy to audit — no side effects, pure functions only
///
/// The bonding curve formula is the same constant-product (x·y = k) used by
/// Uniswap v2 and Pump.fun, with virtual reserves to set the initial price:
///
///   price = virtual_sol / virtual_tokens
///
/// Virtual reserves are adjusted on every trade. Real reserves track actual
/// SOL and tokens in the vault.

/// Precision multiplier for reward-per-token accumulator (avoids integer loss).
pub const REWARD_PRECISION: u128 = 1_000_000_000_000;

/// Max fee in basis points (5%) — hard cap to protect users.
pub const MAX_FEE_BPS: u64 = 500;

// ─── Buy Pricing ──────────────────────────────────────────────────────────────

/// Calculate tokens out for a given net SOL in (after fees).
///
/// Formula: tokens_out = vt * net_sol / (vs + net_sol)
///
/// Returns None on overflow (impossible in practice with Solana's u64 ranges,
/// but we handle it for correctness).
///
/// # Arguments
/// * `virtual_token_reserves` — current virtual token reserves
/// * `virtual_sol_reserves`   — current virtual SOL reserves (lamports)
/// * `net_sol`                — SOL entering the curve (lamports, after fee deduction)
#[inline]
pub fn calc_tokens_out(
    virtual_token_reserves: u64,
    virtual_sol_reserves: u64,
    net_sol: u64,
) -> Option<u64> {
    let vt = virtual_token_reserves as u128;
    let vs = virtual_sol_reserves as u128;
    let s  = net_sol as u128;

    let numerator   = vt.checked_mul(s)?;
    let denominator = vs.checked_add(s)?;
    let tokens_out  = numerator.checked_div(denominator)?;

    // Safe to cast: tokens_out ≤ vt which is at most u64::MAX
    if tokens_out > u64::MAX as u128 { return None; }
    Some(tokens_out as u64)
}

// ─── Sell Pricing ─────────────────────────────────────────────────────────────

/// Calculate gross SOL out for a given token amount in.
///
/// Formula: sol_out = vs * tokens_in / (vt + tokens_in)
///
/// # Arguments
/// * `virtual_sol_reserves`   — current virtual SOL reserves (lamports)
/// * `virtual_token_reserves` — current virtual token reserves
/// * `tokens_in`              — token amount to sell
#[inline]
pub fn calc_sol_out(
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    tokens_in: u64,
) -> Option<u64> {
    let vs = virtual_sol_reserves as u128;
    let vt = virtual_token_reserves as u128;
    let t  = tokens_in as u128;

    let numerator   = vs.checked_mul(t)?;
    let denominator = vt.checked_add(t)?;
    let sol_out     = numerator.checked_div(denominator)?;

    if sol_out > u64::MAX as u128 { return None; }
    Some(sol_out as u64)
}

// ─── Fee Calculation ──────────────────────────────────────────────────────────

/// Calculate fee split from a gross SOL amount.
///
/// Returns `(total_fee, platform_fee, creator_fee)` in lamports.
///
/// # Arguments
/// * `gross_sol`          — gross SOL amount (lamports)
/// * `fee_bps`            — total fee in basis points (100 = 1%)
/// * `platform_share_bps` — platform's share of total fee (75 = 75%)
#[inline]
pub fn calc_fees(
    gross_sol: u64,
    fee_bps: u64,
    platform_share_bps: u64,
) -> (u64, u64, u64) {
    let total_fee = gross_sol
        .saturating_mul(fee_bps)
        .checked_div(10_000)
        .unwrap_or(0);
    let platform_fee = total_fee
        .saturating_mul(platform_share_bps)
        .checked_div(100)
        .unwrap_or(0);
    let creator_fee = total_fee.saturating_sub(platform_fee);
    (total_fee, platform_fee, creator_fee)
}

// ─── Price Impact ─────────────────────────────────────────────────────────────

/// Calculate price impact of a buy as basis points (1 = 0.01%).
///
/// Price before buy: vs / vt
/// Price after buy:  (vs + net_sol) / (vt - tokens_out)
/// Impact: (price_after - price_before) / price_before * 10_000
///
/// Returns 0 on trivial inputs, 10_000 (100%) if the pool would be drained.
#[inline]
pub fn buy_price_impact_bps(
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    net_sol: u64,
    tokens_out: u64,
) -> u64 {
    if virtual_sol_reserves == 0 || virtual_token_reserves == 0 { return 0; }
    let vt_after = virtual_token_reserves.saturating_sub(tokens_out);
    if vt_after == 0 { return 10_000; }

    // Use u128 to avoid overflow in cross-multiplication
    let vs = virtual_sol_reserves as u128;
    let vt = virtual_token_reserves as u128;
    let vs_after = vs.saturating_add(net_sol as u128);
    let vt_after = vt_after as u128;

    // price_before = vs / vt (scaled by 1e12 for precision)
    // price_after  = vs_after / vt_after
    // impact_bps   = (price_after - price_before) / price_before * 10_000
    let price_before_scaled = vs.saturating_mul(1_000_000_000_000).checked_div(vt).unwrap_or(0);
    let price_after_scaled  = vs_after.saturating_mul(1_000_000_000_000).checked_div(vt_after).unwrap_or(0);

    if price_before_scaled == 0 { return 0; }
    let impact = price_after_scaled
        .saturating_sub(price_before_scaled)
        .saturating_mul(10_000)
        .checked_div(price_before_scaled)
        .unwrap_or(0);

    // Cap at 10_000 (100%)
    impact.min(10_000) as u64
}

/// Calculate price impact of a sell as basis points.
#[inline]
pub fn sell_price_impact_bps(
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    tokens_in: u64,
    sol_out: u64,
) -> u64 {
    if virtual_sol_reserves == 0 || virtual_token_reserves == 0 { return 0; }
    let vs = virtual_sol_reserves as u128;
    let vt = virtual_token_reserves as u128;
    let vs_after = vs.saturating_sub(sol_out as u128);
    let vt_after = vt.saturating_add(tokens_in as u128);

    let price_before_scaled = vs.saturating_mul(1_000_000_000_000).checked_div(vt).unwrap_or(0);
    let price_after_scaled  = vs_after.saturating_mul(1_000_000_000_000).checked_div(vt_after).unwrap_or(0);

    if price_before_scaled == 0 { return 0; }
    let impact = price_before_scaled
        .saturating_sub(price_after_scaled)
        .saturating_mul(10_000)
        .checked_div(price_before_scaled)
        .unwrap_or(0);

    impact.min(10_000) as u64
}

// ─── Staker Reward Math ───────────────────────────────────────────────────────

/// Increment the accumulated reward per token accumulator.
///
/// # Arguments
/// * `accumulated` — current accumulator value
/// * `new_sol`     — new SOL rewards to distribute (lamports)
/// * `total_staked`— total tokens staked right now
///
/// Returns the updated accumulator value.
#[inline]
pub fn add_reward_to_accumulator(
    accumulated: u128,
    new_sol: u64,
    total_staked: u64,
) -> u128 {
    if total_staked == 0 || new_sol == 0 { return accumulated; }
    let increase = (new_sol as u128)
        .saturating_mul(REWARD_PRECISION)
        .checked_div(total_staked as u128)
        .unwrap_or(0);
    accumulated.saturating_add(increase)
}

/// Calculate pending SOL rewards for a staker.
///
/// # Arguments
/// * `accumulated_reward_per_token` — current vault accumulator
/// * `reward_debt`                  — accumulator value at last checkpoint
/// * `amount_staked`                — tokens staked
#[inline]
pub fn pending_rewards(
    accumulated_reward_per_token: u128,
    reward_debt: u128,
    amount_staked: u64,
) -> u64 {
    accumulated_reward_per_token
        .saturating_sub(reward_debt)
        .checked_mul(amount_staked as u128)
        .unwrap_or(0)
        .checked_div(REWARD_PRECISION)
        .unwrap_or(0) as u64
}

// ─── Graduation ───────────────────────────────────────────────────────────────

/// Returns true if the projected SOL reserves (after a buy) meet the threshold.
#[inline]
pub fn will_graduate_after_buy(
    current_real_sol: u64,
    net_sol_in: u64,
    threshold: u64,
) -> bool {
    current_real_sol.saturating_add(net_sol_in) >= threshold
}

/// How much more SOL is needed to reach graduation.
/// Returns 0 if already at or past the threshold.
#[inline]
pub fn sol_to_graduation(real_sol_reserves: u64, threshold: u64) -> u64 {
    threshold.saturating_sub(real_sol_reserves)
}

/// Graduation progress as a u16 in basis points (0 = 0%, 10_000 = 100%).
#[inline]
pub fn graduation_progress_bps(real_sol_reserves: u64, threshold: u64) -> u16 {
    if threshold == 0 { return 10_000; }
    let progress = (real_sol_reserves as u128)
        .saturating_mul(10_000)
        .checked_div(threshold as u128)
        .unwrap_or(0)
        .min(10_000);
    progress as u16
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VIRTUAL_SOL: u64   = 30_000_000_000;          // 30 SOL
    const VIRTUAL_TOKEN: u64 = 1_073_000_000_000_000;   // 1.073B tokens

    #[test]
    fn test_buy_price_increases_with_size() {
        let small_buy = calc_tokens_out(VIRTUAL_TOKEN, VIRTUAL_SOL, 100_000_000).unwrap(); // 0.1 SOL
        let large_buy = calc_tokens_out(VIRTUAL_TOKEN, VIRTUAL_SOL, 1_000_000_000).unwrap(); // 1 SOL
        // 10x SOL should give less than 10x tokens (price impact)
        assert!(large_buy < small_buy * 10);
        // But more than small_buy
        assert!(large_buy > small_buy);
    }

    #[test]
    fn test_buy_sell_roundtrip_loses_to_fees() {
        // Buy 1 SOL, then sell those tokens back
        // Net SOL received should be less than 1 SOL (fees + price impact)
        let net_sol = 990_000_000u64; // 1 SOL after 1% fee
        let tokens_out = calc_tokens_out(VIRTUAL_TOKEN, VIRTUAL_SOL, net_sol).unwrap();

        let vt_after = VIRTUAL_TOKEN - tokens_out;
        let vs_after = VIRTUAL_SOL + net_sol;

        let sol_back = calc_sol_out(vs_after, vt_after, tokens_out).unwrap();
        // Should get back roughly what we put in (minus tiny price impact on this small trade)
        assert!(sol_back < net_sol);
        assert!(sol_back > net_sol * 99 / 100); // within 1% of net_sol
    }

    #[test]
    fn test_fee_calc_sums_correctly() {
        let (total, platform, creator) = calc_fees(1_000_000_000, 100, 75);
        assert_eq!(total, 10_000_000);           // 1%
        assert_eq!(platform, 7_500_000);         // 0.75%
        assert_eq!(creator, 2_500_000);          // 0.25%
        assert_eq!(platform + creator, total);
    }

    #[test]
    fn test_price_impact_small_buy() {
        let tokens = calc_tokens_out(VIRTUAL_TOKEN, VIRTUAL_SOL, 100_000_000).unwrap();
        let impact = buy_price_impact_bps(VIRTUAL_SOL, VIRTUAL_TOKEN, 100_000_000, tokens);
        // 0.1 SOL buy against 30 SOL virtual pool ≈ 0.33% impact (33 bps)
        // Should be under 1% (100 bps)
        assert!(impact < 100, "Price impact too high: {} bps", impact);
        // But should be non-zero (there IS impact)
        assert!(impact > 0, "Price impact should not be zero");
    }

    #[test]
    fn test_price_impact_large_buy() {
        let tokens = calc_tokens_out(VIRTUAL_TOKEN, VIRTUAL_SOL, 10_000_000_000).unwrap();
        let impact = buy_price_impact_bps(VIRTUAL_SOL, VIRTUAL_TOKEN, 10_000_000_000, tokens);
        // 10 SOL buy should have significant impact
        assert!(impact > 100, "Expected high impact, got {} bps", impact);
    }

    #[test]
    fn test_graduation_progress() {
        let threshold = 85_000_000_000u64;
        assert_eq!(graduation_progress_bps(0, threshold), 0);
        assert_eq!(graduation_progress_bps(threshold, threshold), 10_000);
        assert_eq!(graduation_progress_bps(threshold / 2, threshold), 5_000);
    }

    #[test]
    fn test_staker_rewards() {
        let mut acc: u128 = 0;
        acc = add_reward_to_accumulator(acc, 1_000_000_000, 100_000_000); // 1 SOL reward, 100M staked
        let pending = pending_rewards(acc, 0, 10_000_000); // 10M staked = 10% share
        // Should get ~0.1 SOL (10% of 1 SOL)
        assert!(pending > 90_000_000 && pending <= 100_000_000);
    }

    #[test]
    fn test_no_overflow_on_large_inputs() {
        // Should not panic on values near u64::MAX
        let result = calc_tokens_out(u64::MAX / 2, u64::MAX / 2, 1_000_000_000);
        assert!(result.is_some());

        let result = calc_sol_out(u64::MAX / 2, u64::MAX / 2, 1_000_000_000);
        assert!(result.is_some());
    }
}
