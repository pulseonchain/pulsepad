// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title BondingCurveMath
/// @notice Pure math library for the Pulse constant-product bonding curve.
/// @dev Mirrors Solana math.rs exactly. All functions pure.
library BondingCurveMath {
    uint256 public constant REWARD_PRECISION = 1e12;
    uint256 public constant BPS_DENOMINATOR = 10_000;
    uint256 public constant MAX_FEE_BPS = 500;

    /// @notice 3-minute anti-snipe window
    uint256 public constant ANTI_SNIPE_WINDOW_SECS = 180;
    /// @notice Anti-snipe multiplier: 3.00x
    uint256 public constant ANTI_SNIPE_MULTIPLIER = 3;
    /// @notice Platform always gets 3/4 of fees
    uint256 public constant PLATFORM_FRACTION = 75;

    // ── Graduation Tiers ────────────────────────────────────────────────────
    enum GraduationTier { Fast, Standard, Stable }

    /// Thresholds per chain per tier
    function tierThreshold(GraduationTier tier, uint256 nativePerSol, uint256 baseFastSol) internal pure returns (uint256) {
        if (tier == GraduationTier.Fast) return baseFastSol;
        if (tier == GraduationTier.Standard) return (baseFastSol * 150) / 80;
        return (baseFastSol * 240) / 80; // Stable
    }

    // ── Buy pricing ─────────────────────────────────────────────────────────

    function calcTokensOut(uint256 virtualTokenReserves, uint256 virtualNativeReserves, uint256 netNative) internal pure returns (uint256) {
        require(netNative > 0 && virtualTokenReserves > 0 && virtualNativeReserves > 0, "Pulse: zero");
        return (virtualTokenReserves * netNative) / (virtualNativeReserves + netNative);
    }

    function calcTokensOutWithAntiSnipe(uint256 virtualTokenReserves, uint256 virtualNativeReserves, uint256 netNative, uint256 poolInitAt) internal view returns (uint256) {
        uint256 effectiveVirtual = virtualNativeReserves;
        if (poolInitAt > 0 && block.timestamp < poolInitAt + ANTI_SNIPE_WINDOW_SECS) {
            effectiveVirtual = virtualNativeReserves * ANTI_SNIPE_MULTIPLIER;
        }
        return calcTokensOut(virtualTokenReserves, effectiveVirtual, netNative);
    }

    // ── Sell pricing ────────────────────────────────────────────────────────

    function calcNativeOut(uint256 virtualNativeReserves, uint256 virtualTokenReserves, uint256 tokensIn) internal pure returns (uint256) {
        require(tokensIn > 0 && virtualNativeReserves > 0 && virtualTokenReserves > 0, "Pulse: zero");
        return (virtualNativeReserves * tokensIn) / (virtualTokenReserves + tokensIn);
    }

    // ── Fee calculation ─────────────────────────────────────────────────────

    function calcFees(uint256 grossNative, uint256 feeBps) internal pure returns (uint256 totalFee, uint256 platformFee, uint256 creatorFee) {
        totalFee = (grossNative * feeBps) / BPS_DENOMINATOR;
        platformFee = (totalFee * PLATFORM_FRACTION) / 100;
        creatorFee = totalFee - platformFee;
    }

    function buyPriceImpactBps(uint256 vs, uint256 vt, uint256 netNative, uint256 tokensOut) internal pure returns (uint256) {
        if (vs == 0 || vt == 0) return 0;
        uint256 vtAfter = vt - tokensOut;
        if (vtAfter == 0) return BPS_DENOMINATOR;
        uint256 priceBefore = (vs * 1e12) / vt;
        uint256 priceAfter = ((vs + netNative) * 1e12) / vtAfter;
        if (priceBefore == 0) return 0;
        uint256 impact = ((priceAfter - priceBefore) * BPS_DENOMINATOR) / priceBefore;
        return impact > BPS_DENOMINATOR ? BPS_DENOMINATOR : impact;
    }

    function sellPriceImpactBps(uint256 vs, uint256 vt, uint256 tokensIn, uint256 nativeOut) internal pure returns (uint256) {
        if (vs == 0 || vt == 0) return 0;
        uint256 priceBefore = (vs * 1e12) / vt;
        uint256 priceAfter = ((vs - nativeOut) * 1e12) / (vt + tokensIn);
        if (priceBefore == 0) return 0;
        uint256 impact = ((priceBefore - priceAfter) * BPS_DENOMINATOR) / priceBefore;
        return impact > BPS_DENOMINATOR ? BPS_DENOMINATOR : impact;
    }

    function graduationProgressBps(uint256 reserves, uint256 threshold) internal pure returns (uint16) {
        if (threshold == 0) return uint16(BPS_DENOMINATOR);
        uint256 p = (reserves * BPS_DENOMINATOR) / threshold;
        return uint16(p > BPS_DENOMINATOR ? BPS_DENOMINATOR : p);
    }

    function addRewardToAccumulator(uint256 acc, uint256 newNative, uint256 totalStaked) internal pure returns (uint256) {
        if (totalStaked == 0 || newNative == 0) return acc;
        return acc + (newNative * REWARD_PRECISION) / totalStaked;
    }

    function pendingRewards(uint256 accPerToken, uint256 rewardDebt, uint256 amountStaked) internal pure returns (uint256) {
        return ((accPerToken - rewardDebt) * amountStaked) / REWARD_PRECISION;
    }

    function _sqrt(uint256 y) internal pure returns (uint256 z) {
        if (y > 3) { z = y; uint256 x = y / 2 + 1; while (x < z) { z = x; x = (y / x + x) / 2; } }
        else if (y != 0) { z = 1; }
    }
}