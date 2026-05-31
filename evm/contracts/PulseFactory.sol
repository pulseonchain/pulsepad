// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {PulsePool} from "./PulsePool.sol";
import {PulseGlobalConfig} from "./PulseGlobalConfig.sol";
import {PulseCircuitBreaker} from "./modules/PulseCircuitBreaker.sol";
import {PulseTier} from "./PulseConstants.sol";

/// @title PulseFactory
/// @notice CREATE2-based factory that deploys PulsePool contracts deterministically.
/// @dev Equivalent to Solana's PDA system — predictable addresses from (factory, salt).
///      Each chain has ONE PulseFactory that manages all PulsePools.
contract PulseFactory {
    // ═══════════════════════════════════════════════════════════════════════════
    // IMMUTABLES
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Global config — shared across all pools on this chain
    PulseGlobalConfig public immutable globalConfig;

    /// @notice Circuit breaker — shared across all pools on this chain
    PulseCircuitBreaker public immutable circuitBreaker;

    /// @notice DEX adapter mapping: MigrationTarget enum value → adapter address
    /// @dev Adapters are set by the factory owner (admin/DAO)
    mapping(PulsePool.MigrationTarget => address) public dexAdapters;

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice All deployed pool addresses (in order of creation)
    address[] public pools;

    /// @notice Token address → Pool address
    mapping(address => address) public poolByToken;

    /// @notice Number of pools deployed
    uint256 public poolCount;

    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    event PoolCreated(
        address indexed pool,
        address indexed token,
        address indexed creator,
        string name,
        string symbol,
        PulsePool.MigrationTarget migrationTarget,
        bytes32 salt
    );
    event DexAdapterSet(PulsePool.MigrationTarget indexed target, address adapter);

    // ═══════════════════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════════════════

    error PulseFactory__InvalidAdapter();
    error PulseFactory__NameTooLong();
    error PulseFactory__SymbolTooLong();
    error PulseFactory__PoolAlreadyExists();

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════════════════

    constructor(address _globalConfig, address _circuitBreaker) {
        require(_globalConfig != address(0), "PulseFactory: zero config");
        require(_circuitBreaker != address(0), "PulseFactory: zero breaker");
        globalConfig = PulseGlobalConfig(_globalConfig);
        circuitBreaker = PulseCircuitBreaker(_circuitBreaker);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ADMIN: SET DEX ADAPTERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Set the DEX adapter for a given migration target.
    /// @dev Only callable by the GlobalConfig owner (platform multisig).
    function setDexAdapter(
        PulsePool.MigrationTarget target,
        address adapter
    ) external {
        require(msg.sender == globalConfig.owner(), "PulseFactory: only config owner");
        require(adapter != address(0), "PulseFactory: zero adapter");
        dexAdapters[target] = adapter;
        emit DexAdapterSet(target, adapter);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CREATE POOL (CREATE2)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Deploy a new PulsePool + PulseToken via CREATE2.
    /// @param name Token name (1-32 ASCII chars)
    /// @param symbol Token symbol (1-10 uppercase ASCII chars)
    /// @param migrationTarget Target DEX for graduation
    /// @param salt Custom salt for CREATE2 (use keccak256 of name+symbol+creator)
    /// @return pool Address of the deployed PulsePool
    /// @return token Address of the deployed PulseToken
    function createPool(
        string calldata name,
        string calldata symbol,
        PulsePool.MigrationTarget migrationTarget,
        bytes32 salt,
        PulseTier tier,
        uint256 feeBps,
        bool feesToAgent,
        string calldata agentName,
        uint8 partialPct
    ) external returns (address pool, address token) {
        // ── Validate inputs (mirrors Solana validation) ────────────────────────
        bytes memory nameBytes = bytes(name);
        require(nameBytes.length >= 1 && nameBytes.length <= 32, "PulseFactory: name 1-32 chars");
        require(_isAscii(nameBytes), "PulseFactory: name must be ASCII");

        bytes memory symbolBytes = bytes(symbol);
        require(symbolBytes.length >= 1 && symbolBytes.length <= 10, "PulseFactory: symbol 1-10 chars");
        require(_isAscii(symbolBytes), "PulseFactory: symbol must be ASCII");

        // ── Get adapter (revert if not set) ────────────────────────────────────
        address adapter = dexAdapters[migrationTarget];
        require(adapter != address(0), "PulseFactory: adapter not set");

        // ── Deploy via CREATE2 ─────────────────────────────────────────────────
        bytes memory bytecode = abi.encodePacked(
            type(PulsePool).creationCode,
            abi.encode(name, symbol, msg.sender, migrationTarget, adapter, address(globalConfig), address(circuitBreaker), tier, feeBps, feesToAgent, agentName, partialPct)
        );

        assembly {
            pool := create2(0, add(bytecode, 0x20), mload(bytecode), salt)
        }
        require(pool != address(0), "PulseFactory: deployment failed");

        // ── Extract token address ──────────────────────────────────────────────
        token = PulsePool(pool).mint();

        // ── Store in registry ──────────────────────────────────────────────────
        pools.push(pool);
        poolByToken[token] = pool;
        poolCount = pools.length;

        emit PoolCreated(pool, token, msg.sender, name, symbol, migrationTarget, salt);

        return (pool, token);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PREDICT POOL ADDRESS (CREATE2)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Predict the address of a PulsePool before it's deployed.
    /// @dev Same formula as CREATE2: address(uint160(uint256(keccak256(
    ///      0xff ++ factory ++ salt ++ keccak256(init_code)))))
    function predictPoolAddress(
        string calldata name,
        string calldata symbol,
        PulsePool.MigrationTarget migrationTarget,
        bytes32 salt
    ) external view returns (address) {
        address adapter = dexAdapters[migrationTarget];
        require(adapter != address(0), "PulseFactory: adapter not set");

        bytes memory initCode = abi.encodePacked(
            type(PulsePool).creationCode,
            abi.encode(name, symbol, msg.sender, migrationTarget, adapter, address(globalConfig), address(circuitBreaker), tier, feeBps, feesToAgent, agentName, partialPct)
        );

        bytes32 hash = keccak256(
            abi.encodePacked(
                bytes1(0xff),
                address(this),
                salt,
                keccak256(initCode)
            )
        );

        return address(uint160(uint256(hash)));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VIEW FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Get all deployed pool addresses.
    function getAllPools() external view returns (address[] memory) {
        return pools;
    }

    /// @notice Get the pool for a token.
    function getPoolByToken(address token) external view returns (address) {
        return poolByToken[token];
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // INTERNAL HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Check if a byte array contains only printable ASCII characters.
    function _isAscii(bytes memory data) internal pure returns (bool) {
        for (uint256 i = 0; i < data.length; i++) {
            if (uint8(data[i]) < 0x20 || uint8(data[i]) > 0x7E) {
                return false;
            }
        }
        return true;
    }
}
