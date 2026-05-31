// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {PulseConstants} from "./PulseConstants.sol";
import {PulseTier} from "./PulseConstants.sol";

/// @title PrebondConfig
/// @notice EVM equivalent of Solana's PrebondConfig PDA.
/// @dev Immutable after deployment. Set at pool creation via PulsePool constructor.
contract PrebondConfig {
    // ═══════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════

    /// @notice The PulsePool this config belongs to
    address public immutable pool;

    /// @notice ERC-20 token address
    address public immutable token;

    /// @notice Graduation tier (0=Fast, 1=Standard, 2=Stable)
    PulseTier public graduationTier;

    /// @notice Pool-level fee in basis points (100-500, i.e. 1%-5%)
    uint256 public totalFeeBps;

    /// @notice Whether fees route to the agent wallet instead of creator
    bool public feesToAgent;

    /// @notice Agent wallet address (if feesToAgent is true)
    address public agentWallet;

    /// @notice Agent public name: "Agent <TICKER>"
    string public agentName;

    /// @notice Whether anti-snipe protection is enabled
    bool public antiSnipeEnabled;

    /// @notice Partial migration percentage (0, 10, 20, or 30)
    uint8 public partialMigrationPct;

    /// @notice Creator address
    address public immutable creator;

    /// @notice Creation timestamp
    uint256 public immutable createdAt;

    // ═══════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════

    event PrebondConfigured(
        address indexed pool,
        address indexed token,
        PulseTier tier,
        uint256 feeBps,
        bool feesToAgent,
        string agentName,
        bool antiSnipe,
        uint8 partialMigrationPct
    );

    // ═══════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════

    error Prebond__InvalidFee(uint256 feeBps);
    error Prebond__InvalidAgentName(string name);
    error Prebond__InvalidPartialMigrationPct(uint8 pct);
    error Prebond__InvalidTier(PulseTier tier);

    // ═══════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════

    constructor(
        address _pool,
        address _token,
        PulseTier _tier,
        uint256 _feeBps,
        bool _feesToAgent,
        address _agentWallet,
        string memory _agentName,
        bool _antiSnipeEnabled,
        uint8 _partialMigrationPct,
        address _creator
    ) {
        require(_feeBps >= PulseConstants.MIN_FEE_BPS && _feeBps <= PulseConstants.MAX_FEE_BPS, "Prebond: invalid fee");
        require(_partialMigrationPct == 0 || _partialMigrationPct == 10 || _partialMigrationPct == 20 || _partialMigrationPct == 30, "Prebond: invalid pct");

        if (_feesToAgent) {
            require(bytes(_agentName).length >= 7 && bytes(_agentName).length <= 22, "Prebond: agent name 7-22 chars");
            bytes memory prefix = bytes("Agent ");
            for (uint i = 0; i < prefix.length; i++) {
                require(bytes(_agentName)[i] == prefix[i], "Prebond: must start with 'Agent '");
            }
        }

        pool = _pool;
        token = _token;
        graduationTier = _tier;
        totalFeeBps = _feeBps;
        feesToAgent = _feesToAgent;
        agentWallet = _feesToAgent ? _agentWallet : address(0);
        agentName = _agentName;
        antiSnipeEnabled = _antiSnipeEnabled;
        partialMigrationPct = _partialMigrationPct;
        creator = _creator;
        createdAt = block.timestamp;

        emit PrebondConfigured(_pool, _token, _tier, _feeBps, _feesToAgent, _agentName, _antiSnipeEnabled, _partialMigrationPct);
    }

    // ═══════════════════════════════════════════════════════════════
    // VIEW
    // ═══════════════════════════════════════════════════════════════

    function hasBuyback() external view returns (bool) {
        return partialMigrationPct > 0;
    }

    function tierThreshold(uint256 fast, uint256 standard, uint256 stable) external pure returns (uint256) {
        if (graduationTier == PulseTier.Fast) return fast;
        if (graduationTier == PulseTier.Standard) return standard;
        return stable;
    }
}

/// @title AgentWallet
/// @notice EVM equivalent of Solana's AgentWallet PDA.
/// @dev Holds accumulated fees for the agent. Claims every 3 hours.
contract AgentWallet {
    // ═══════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════

    address public immutable pulsePool;
    address public immutable token;
    string public agentName;
    uint256 public totalEarned;
    uint256 public totalSpent;
    uint256 public claimableAmount;
    uint256 public lastClaimAt;
    uint256 public lastActionAt;
    uint256 public immutable createdAt;

    // ═══════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════

    event AgentClaimed(address indexed pool, uint256 amount, uint256 timestamp);
    event AgentFunded(address indexed pool, uint256 amount, uint256 timestamp);

    // ═══════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════

    error Agent__ClaimTooSoon(uint256 lastClaim, uint256 now_);
    error Agent__NothingToClaim();
    error Agent__NotPool();

    // ═══════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════

    constructor(address _pool, address _token, string memory _agentName) {
        pulsePool = _pool;
        token = _token;
        agentName = _agentName;
        createdAt = block.timestamp;
        lastClaimAt = block.timestamp;
        lastActionAt = block.timestamp;
    }

    // ═══════════════════════════════════════════════════════════════
    // MODIFIERS
    // ═══════════════════════════════════════════════════════════════

    modifier onlyPool() {
        if (msg.sender != pulsePool) revert Agent__NotPool();
        _;
    }

    // ═══════════════════════════════════════════════════════════════
    // POOL-FACING: Accumulate Fees
    // ═══════════════════════════════════════════════════════════════

    /// @notice Called by PulsePool on each buy/sell to accumulate agent's share.
    function accumulate(uint256 amount) external onlyPool {
        claimableAmount += amount;
        totalEarned += amount;
        lastActionAt = block.timestamp;
        emit AgentFunded(pulsePool, amount, block.timestamp);
    }

    // ═══════════════════════════════════════════════════════════════
    // AGENT: Claim
    // ═══════════════════════════════════════════════════════════════

    /// @notice Agent claims accumulated SOL/BNB. Minimum 3h between claims.
    function claim(address to) external onlyPool returns (uint256) {
        if (block.timestamp < lastClaimAt + PulseConstants.AGENT_CLAIM_COOLDOWN) {
            revert Agent__ClaimTooSoon(lastClaimAt, block.timestamp);
        }
        uint256 amount = claimableAmount;
        if (amount == 0) revert Agent__NothingToClaim();

        claimableAmount = 0;
        lastClaimAt = block.timestamp;
        lastActionAt = block.timestamp;
        totalSpent += amount;

        (bool ok, ) = to.call{value: amount}("");
        require(ok, "Agent: transfer failed");

        emit AgentClaimed(pulsePool, amount, block.timestamp);
        return amount;
    }

    receive() external payable {}
}

/// @title VaultClaimTracker
/// @notice EVM equivalent of Solana's VaultClaimTracker PDA.
/// @dev Enforces 500K token / 24h cap per claimer.
contract VaultClaimTracker {
    address public immutable token;
    address public immutable claimer;
    uint256 public tokensClaimed24h;
    uint256 public windowStart;
    uint256 public totalClaimed;

    event VaultClaimed(address indexed claimer, uint256 amount, uint256 total24h);

    error Vault__CapExceeded(uint256 requested, uint256 cap);

    constructor(address _token, address _claimer) {
        token = _token;
        claimer = _claimer;
        windowStart = block.timestamp;
    }

    function recordClaim(uint256 amount) external {
        if (block.timestamp >= windowStart + 24 hours) {
            tokensClaimed24h = 0;
            windowStart = block.timestamp;
        }
        uint256 newTotal = tokensClaimed24h + amount;
        if (newTotal > PulseConstants.MAX_VAULT_PER_24H) {
            revert Vault__CapExceeded(newTotal, PulseConstants.MAX_VAULT_PER_24H);
        }
        tokensClaimed24h = newTotal;
        totalClaimed += amount;
        emit VaultClaimed(claimer, amount, tokensClaimed24h);
    }
}