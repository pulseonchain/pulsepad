// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title PulseTimelockController
/// @notice Timelock for Pulse admin operations — different delays per chain.
/// @dev Inspired by OpenZeppelin TimelockController. Simplified for Pulse's needs.
///      CHAIN-SPECIFIC INSTANCE: each chain deploys with its own minDelay.
///      - ETH mainnet: 48 hours (2 days)
///      - BSC:          24 hours (1 day)
///      - Base:         6 hours
///
/// Controls:
///   - Admin calls (setFees, pause, unpause, graduation threshold changes)
///   - Adapter registration on PulseFactory
contract PulseTimelockController {
    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    event CallScheduled(bytes32 indexed id, address indexed target, uint256 value, bytes data, uint256 delay);
    event CallExecuted(bytes32 indexed id, address indexed target, uint256 value, bytes data);
    event Cancelled(bytes32 indexed id);
    event MinDelayChanged(uint256 oldDelay, uint256 newDelay);

    // ═══════════════════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════════════════
    error Timelock__NotReady(bytes32 id, uint256 readyAt, uint256 now);
    error Timelock__AlreadyScheduled(bytes32 id);
    error Timelock__NotScheduled(bytes32 id);
    error Timelock__NotAdmin();
    error Timelock__CannotExecute(bytes32 id);
    error Timelock__DelayTooLow(uint256 proposed, uint256 min);

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════════════════
    uint256 public constant MIN_DELAY = 1 hours;
    uint256 public constant MAX_DELAY = 7 days;

    address public immutable admin;
    uint256 public minDelay;

    struct ScheduledCall {
        address target;
        uint256 value;
        bytes data;
        uint256 readyAt;
        bool executed;
    }

    mapping(bytes32 => ScheduledCall) private _scheduled;

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════════════════
    constructor(address _admin, uint256 _minDelay) {
        require(_admin != address(0), "Timelock: zero admin");
        require(_minDelay >= MIN_DELAY && _minDelay <= MAX_DELAY, "Timelock: delay out of range");
        admin = _admin;
        minDelay = _minDelay;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MODIFIER
    // ═══════════════════════════════════════════════════════════════════════════
    modifier onlyAdmin() {
        if (msg.sender != admin) revert Timelock__NotAdmin();
        _;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SCHEDULE (Admin only)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Schedule a call to be executed after the timelock delay.
    /// @return id Unique hash for this scheduled call
    function schedule(
        address target,
        uint256 value,
        bytes calldata data,
        uint256 delay
    ) external onlyAdmin returns (bytes32 id) {
        if (delay < minDelay) revert Timelock__DelayTooLow(delay, minDelay);
        if (delay > MAX_DELAY) revert Timelock__DelayTooLow(delay, MAX_DELAY);

        id = hashOperation(target, value, data);
        if (_scheduled[id].readyAt != 0) revert Timelock__AlreadyScheduled(id);

        uint256 readyAt = block.timestamp + delay;
        _scheduled[id] = ScheduledCall({
            target: target,
            value: value,
            data: data,
            readyAt: readyAt,
            executed: false
        });

        emit CallScheduled(id, target, value, data, delay);
        return id;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXECUTE (Anyone — after delay)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Execute a scheduled call after the timelock has passed.
    /// @dev Anyone can execute — the call itself enforces access control.
    function execute(address target, uint256 value, bytes calldata data) external payable returns (bytes memory) {
        bytes32 id = hashOperation(target, value, data);
        ScheduledCall storage sc = _scheduled[id];

        if (sc.readyAt == 0) revert Timelock__NotScheduled(id);
        if (sc.executed) revert Timelock__CannotExecute(id);
        if (block.timestamp < sc.readyAt) revert Timelock__NotReady(id, sc.readyAt, block.timestamp);

        sc.executed = true;

        // slither-disable-next-line arbitrary-send-eth
        (bool success, bytes memory result) = target.call{value: value}(data);
        require(success, "Timelock: call failed");

        emit CallExecuted(id, target, value, data);
        return result;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CANCEL (Admin only)
    // ═══════════════════════════════════════════════════════════════════════════

    function cancel(bytes32 id) external onlyAdmin {
        ScheduledCall storage sc = _scheduled[id];
        if (sc.readyAt == 0) revert Timelock__NotScheduled(id);
        if (sc.executed) revert Timelock__CannotExecute(id);
        delete _scheduled[id];
        emit Cancelled(id);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // UPDATE DELAY (Timelocked itself — requires scheduling)
    // ═══════════════════════════════════════════════════════════════════════════

    function updateMinDelay(uint256 newDelay) external onlyAdmin {
        if (newDelay < MIN_DELAY || newDelay > MAX_DELAY) {
            revert Timelock__DelayTooLow(newDelay, minDelay);
        }
        uint256 old = minDelay;
        minDelay = newDelay;
        emit MinDelayChanged(old, newDelay);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VIEW
    // ═══════════════════════════════════════════════════════════════════════════

    function isOperation(bytes32 id) external view returns (bool) {
        return _scheduled[id].readyAt != 0 && !_scheduled[id].executed;
    }

    function isOperationReady(bytes32 id) external view returns (bool) {
        ScheduledCall storage sc = _scheduled[id];
        return sc.readyAt != 0 && !sc.executed && block.timestamp >= sc.readyAt;
    }

    function getOperation(bytes32 id) external view returns (
        address target, uint256 value, bytes memory data,
        uint256 readyAt, bool executed
    ) {
        ScheduledCall storage sc = _scheduled[id];
        return (sc.target, sc.value, sc.data, sc.readyAt, sc.executed);
    }

    function hashOperation(
        address target,
        uint256 value,
        bytes memory data
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(target, value, data));
    }
}
