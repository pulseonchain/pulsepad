// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title PulseCircuitBreaker
/// @notice Per-chain circuit breaker — stops operations when suspicious activity detected.
/// @dev Deployed ONE per chain. Controls:
///      - Trade volume caps per block
///      - Buy/sell frequency limits per address
///      - Graduation cooldown (prevents flash-loan graduation attacks)
///
///      CHAIN-SPECIFIC thresholds (set at deploy):
///      - Base:    high throughput (cheap gas = more transactions)
///      - BSC:     medium throughput
///      - ETH:     low throughput (expensive gas = fewer tx, but more value)
contract PulseCircuitBreaker {
    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    event CircuitTripped(bytes32 indexed reason, uint256 value, uint256 limit);
    event CircuitReset(bytes32 indexed reason);
    event VolumeCapUpdated(uint256 oldCap, uint256 newCap);
    event TxLimitUpdated(address indexed user, uint256 oldLimit, uint256 newLimit);
    event GraduationCooldownUpdated(uint256 oldSecs, uint256 newSecs);

    // ═══════════════════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════════════════
    error Breaker__NotAdmin();
    error Breaker__VolumeExceeded(uint256 volume, uint256 cap);
    error Breaker__TxFrequencyExceeded(address user, uint256 count, uint256 limit);
    error Breaker__GraduationTooSoon(uint256 lastGraduation, uint256 cooldown);
    error Breaker__Tripped(bytes32 reason);

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════════════════
    address public admin;

    /// @notice Max native volume (wei) per block across all pools
    uint256 public maxVolumePerBlock;

    /// @notice Max transactions per block per address
    uint256 public maxTxPerBlockPerAddress;

    /// @notice Cooldown between graduations (prevents flash-attacks)
    uint256 public graduationCooldownSecs;

    /// @notice Last graduation timestamp
    uint256 public lastGraduationAt;

    /// @notice Current block volume tracker
    uint256 public blockVolume;
    uint256 public volumeBlock;

    /// @notice Per-user per-block transaction count
    mapping(address => uint256) public txCount;
    mapping(address => uint256) public txBlock;

    /// @notice Named tripwires (set by admin, checked by PulsePool)
    mapping(bytes32 => bool) public tripped;

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════════════════

    /// @param _admin Governance/admin address
    /// @param _maxVolumePerBlock Max native (BNB/ETH) per block in wei
    /// @param _maxTxPerBlockPerAddress Max trades per address per block
    /// @param _graduationCooldownSecs Time between graduations
    constructor(
        address _admin,
        uint256 _maxVolumePerBlock,
        uint256 _maxTxPerBlockPerAddress,
        uint256 _graduationCooldownSecs
    ) {
        require(_admin != address(0), "Breaker: zero admin");
        admin = _admin;
        maxVolumePerBlock = _maxVolumePerBlock;
        maxTxPerBlockPerAddress = _maxTxPerBlockPerAddress;
        graduationCooldownSecs = _graduationCooldownSecs;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MODIFIER
    // ═══════════════════════════════════════════════════════════════════════════
    modifier onlyAdmin() {
        if (msg.sender != admin) revert Breaker__NotAdmin();
        _;
    }

    modifier whenNotTripped(bytes32 reason) {
        if (tripped[reason]) revert Breaker__Tripped(reason);
        _;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VOLUME CHECKS (called by PulsePool.buy/sell)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Check and update per-block volume. Reverts if cap exceeded.
    function checkVolume(uint256 amount) external {
        if (block.number != volumeBlock) {
            volumeBlock = block.number;
            blockVolume = 0;
        }
        uint256 newVolume = blockVolume + amount;
        if (newVolume > maxVolumePerBlock) {
            revert Breaker__VolumeExceeded(newVolume, maxVolumePerBlock);
        }
        blockVolume = newVolume;
    }

    /// @notice Check and update per-user per-block tx count.
    function checkTxFrequency(address user) external {
        if (block.number != txBlock[user]) {
            txBlock[user] = block.number;
            txCount[user] = 0;
        }
        uint256 count = txCount[user] + 1;
        if (count > maxTxPerBlockPerAddress) {
            revert Breaker__TxFrequencyExceeded(user, count, maxTxPerBlockPerAddress);
        }
        txCount[user] = count;
    }

    /// @notice Check graduation cooldown. Reverts if too soon.
    function checkGraduation() external {
        if (lastGraduationAt > 0) {
            uint256 elapsed = block.timestamp - lastGraduationAt;
            if (elapsed < graduationCooldownSecs) {
                revert Breaker__GraduationTooSoon(lastGraduationAt, graduationCooldownSecs);
            }
        }
        lastGraduationAt = block.timestamp;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRIPWIRES (Admin only)
    // ═══════════════════════════════════════════════════════════════════════════

    function trip(bytes32 reason) external onlyAdmin {
        tripped[reason] = true;
        emit CircuitTripped(reason, 0, 0);
    }

    function reset(bytes32 reason) external onlyAdmin {
        tripped[reason] = false;
        emit CircuitReset(reason);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CONFIG UPDATES (Admin only)
    // ═══════════════════════════════════════════════════════════════════════════

    function setVolumeCap(uint256 _cap) external onlyAdmin {
        uint256 old = maxVolumePerBlock;
        maxVolumePerBlock = _cap;
        emit VolumeCapUpdated(old, _cap);
    }

    function setTxLimit(uint256 _limit) external onlyAdmin {
        uint256 old = maxTxPerBlockPerAddress;
        maxTxPerBlockPerAddress = _limit;
        emit TxLimitUpdated(msg.sender, old, _limit);
    }

    function setGraduationCooldown(uint256 _secs) external onlyAdmin {
        uint256 old = graduationCooldownSecs;
        graduationCooldownSecs = _secs;
        emit GraduationCooldownUpdated(old, _secs);
    }

    function transferAdmin(address newAdmin) external onlyAdmin {
        require(newAdmin != address(0), "Breaker: zero admin");
        admin = newAdmin;
    }
}
