use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers - Common utility functions
// ─────────────────────────────────────────────────────────────────────────────

/// Calculate percentage of a value
pub fn calculate_percentage(value: u64, percentage_bps: u64) -> u64 {
    value
        .saturating_mul(percentage_bps)
        .saturating_div(10_000)
}

/// Calculate fee split
pub fn calculate_fee_split(
    gross: u64,
    total_fee_bps: u64,
    platform_share_bps: u64,
) -> (u64, u64, u64) {
    let total_fee = calculate_percentage(gross, total_fee_bps);
    let platform_fee = calculate_percentage(total_fee, platform_share_bps);
    let creator_fee = total_fee.saturating_sub(platform_fee);
    (total_fee, platform_fee, creator_fee)
}

/// Calculate price impact in basis points
pub fn calculate_price_impact_bps(
    before_price: u64,
    after_price: u64,
) -> u64 {
    if before_price == 0 {
        return 0;
    }
    
    let diff = if after_price > before_price {
        after_price.saturating_sub(before_price)
    } else {
        before_price.saturating_sub(after_price)
    };
    
    diff
        .saturating_mul(10_000)
        .saturating_div(before_price)
}

/// Clamp value between min and max
pub fn clamp(value: u64, min: u64, max: u64) -> u64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Calculate time remaining until a timestamp
pub fn time_remaining(until: i64, now: i64) -> i64 {
    until.saturating_sub(now)
}

/// Check if a timestamp has passed
pub fn has_passed(timestamp: i64, now: i64) -> bool {
    now >= timestamp
}

/// Get current day/week/month from timestamp
pub fn get_time_period(timestamp: i64) -> (u64, u64, u64) {
    let day_seconds = 86_400;
    let week_seconds = 604_800;
    let month_seconds = 2_592_000;
    
    let seconds_since_epoch = timestamp as u64;
    let day = seconds_since_epoch / day_seconds;
    let week = seconds_since_epoch / week_seconds;
    let month = seconds_since_epoch / month_seconds;
    
    (day, week, month)
}

/// Calculate sliding window average
pub fn sliding_window_average(values: &[u64], window_size: usize) -> Option<u64> {
    if values.is_empty() || window_size == 0 {
        return None;
    }
    
    let start = values.len().saturating_sub(window_size);
    let window = &values[start..];
    
    if window.is_empty() {
        return None;
    }
    
    let sum: u64 = window.iter().sum();
    Some(sum / window.len() as u64)
}

/// Check if value is within tolerance
pub fn within_tolerance(actual: u64, expected: u64, tolerance_bps: u64) -> bool {
    let diff = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };
    
    let max_diff = expected
        .saturating_mul(tolerance_bps)
        .saturating_div(10_000);
    
    diff <= max_diff
}

/// Safe division with rounding
pub fn safe_divide(numerator: u64, denominator: u64, round_up: bool) -> u64 {
    if denominator == 0 {
        return 0;
    }
    
    let result = numerator / denominator;
    let remainder = numerator % denominator;
    
    if remainder == 0 {
        return result;
    }
    
    if round_up {
        result.saturating_add(1)
    } else {
        result
    }
}

/// Convert basis points to percentage string
pub fn bps_to_percent_string(bps: u64) -> String {
    format!("{:.2}%", bps as f64 / 100.0)
}

/// Convert percentage to basis points
pub fn percent_to_bps(percentage: f64) -> u64 {
    (percentage * 100.0).round() as u64
}

/// Generate PDA bump seed
pub fn find_bump(mint: &Pubkey, seed: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[seed, mint.as_ref()], &crate::ID)
}
