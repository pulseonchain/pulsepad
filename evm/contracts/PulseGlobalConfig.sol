// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {BondingCurveMath} from "./lib/BondingCurveMath.sol";

/// @title PulseGlobalConfig
/// @notice Singleton configuration for the Pulse protocol on a given chain.
/// @dev Mirrors the Solana GlobalConfig PDA. Deployed once per chain.
///      Owned by the platform multisig/DAO.
contract PulseGlobalConfig is Ownable {
    using BondingCurveMath for uint256;

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTANTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Platform wallet — all chains use the same address.
    ///         0.75% of every trade goes here.
    address public constant PLATFORM_WALLET = 0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA;

    uint256 public constant BPS_DENOMINATOR = 10_000;
    uint256 public constant MAX_FEE_BPS = 500; // 5% max

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Total fee in basis points (100 = 1%)
    uint256 public feeBasisPoints = 100;

    /// @notice Platform's share of the total fee (75 = 75%)
    uint256 public platformShareBps = 75;

    /// @notice Creator's share of the total fee (25 = 25%)
    /// @dev Always = feeBasisPoints - platformShareBps
    uint256 public creatorShareBps = 25;

    /// @notice Graduation tier (global default — pools can override)
    uint8 public graduationTier; // 0=Fast, 1=Standard, 2=Stable

    /// @notice Graduation thresholds per tier (in wei)
    uint256 public fastThreshold;
    uint256 public standardThreshold;
    uint256 public stableThreshold;

    /// @notice Minimum creator fee reserve to leave for gas (0.005 native in wei)
    uint256 public minCreatorReserve = 0.005 ether;

    /// @notice Maximum single trade size in native wei (10 native by default)
    uint256 public maxTradeNative = 10 ether;

    /// @notice Maximum price impact in bps (1000 = 10%)
    uint256 public maxPriceImpactBps = 1000;

    /// @notice Protocol pause flag (only owner)
    bool public paused;

    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    event ConfigUpdated(
        uint256 feeBasisPoints,
        uint256 platformShareBps,
        uint256 creatorShareBps,
        uint256 graduationThreshold
    );
    event Paused(address indexed by);
    event Unpaused(address indexed by);

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════════════════

    /// @param _graduationThreshold OLD — kept for compat.
    /// @param _fast Fast tier threshold (e.g. 15 BNB, 9 ETH)
    /// @param _standard Standard tier threshold
    /// @param _stable Stable tier threshold
    constructor(uint256 _fast, uint256 _standard, uint256 _stable) Ownable(msg.sender) {
        require(_fast > 0 && _standard > _fast && _stable > _standard, "Pulse: invalid tiers");
        fastThreshold = _fast;
        standardThreshold = _standard;
        stableThreshold = _stable;
        graduationTier = 1;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MODIFIERS
    // ═══════════════════════════════════════════════════════════════════════════

    modifier whenNotPaused() {
        require(!paused, "Pulse: protocol paused");
        _;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CONFIGURATION (Owner Only)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Update fee parameters. Only owner.
    /// @param _totalFeeBps Total fee in bps (max 500 / 5%)
    /// @param _platformShareBps Platform share of total fee (must sum with creator to 100)
    function setFees(uint256 _totalFeeBps, uint256 _platformShareBps) external onlyOwner {
        require(_totalFeeBps > 0 && _totalFeeBps <= MAX_FEE_BPS, "Pulse: invalid fee bps");
        require(_platformShareBps <= 100, "Pulse: invalid platform share");
        feeBasisPoints = _totalFeeBps;
        platformShareBps = _platformShareBps;
        creatorShareBps = 100 - _platformShareBps;
        emit ConfigUpdated(feeBasisPoints, platformShareBps, creatorShareBps, graduationThreshold);
    }

    /// @notice Update graduation thresholds. Only owner.
    function setThresholds(uint8 _tier, uint256 _fast, uint256 _standard, uint256 _stable) external onlyOwner {
        require(_fast > 0 && _standard > _fast && _stable > _standard, "Pulse: invalid thresholds");
        require(_tier <= 2, "Pulse: invalid tier");
        graduationTier = _tier;
        fastThreshold = _fast;
        standardThreshold = _standard;
        stableThreshold = _stable;
        emit ConfigUpdated(feeBasisPoints, platformShareBps, creatorShareBps, _fast);
    }

    /// @notice Get threshold for a given tier.
    function getThresholdForTier(uint8 tier) external view returns (uint256) {
        if (tier == 0) return fastThreshold;
        if (tier == 1) return standardThreshold;
        return stableThreshold;
    }

    /// @notice Update max trade size. Only owner.
    function setMaxTradeNative(uint256 _maxTrade) external onlyOwner {
        require(_maxTrade > 0, "Pulse: zero max trade");
        maxTradeNative = _maxTrade;
    }

    /// @notice Update max price impact. Only owner.
    function setMaxPriceImpactBps(uint256 _maxImpact) external onlyOwner {
        require(_maxImpact <= BPS_DENOMINATOR, "Pulse: impact exceeds 100%");
        maxPriceImpactBps = _maxImpact;
    }

    /// @notice Pause all protocol operations. Only owner.
    function pause() external onlyOwner {
        require(!paused, "Pulse: already paused");
        paused = true;
        emit Paused(msg.sender);
    }

    /// @notice Unpause protocol. Only owner.
    function unpause() external onlyOwner {
        require(paused, "Pulse: not paused");
        paused = false;
        emit Unpaused(msg.sender);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FEE CALCULATION (View)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Calculate fee breakdown for a given native amount.
    /// @return totalFee Total fee in wei
    /// @return platformFee Goes to PLATFORM_WALLET
    /// @return creatorFee Accumulates for creator
    function calcFees(uint256 nativeAmount)
        external
        view
        returns (uint256 totalFee, uint256 platformFee, uint256 creatorFee)
    {
        return BondingCurveMath.calcFees(nativeAmount, feeBasisPoints, platformShareBps);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VALIDATION
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Validate that config parameters are within safe bounds.
    /// @dev Called by PulsePool on every trade (cheap — view function).
    function validate() external view {
        require(feeBasisPoints <= MAX_FEE_BPS, "Pulse: fee too high");
        require(platformShareBps + creatorShareBps == 100, "Pulse: share sum != 100");
        require(maxTradeNative > 0, "Pulse: zero max trade");
        require(maxPriceImpactBps <= BPS_DENOMINATOR, "Pulse: invalid max impact");
    }
}
