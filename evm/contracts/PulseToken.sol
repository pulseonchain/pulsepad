// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";

/// @title PulseToken
/// @notice ERC-20 token deployed by each PulsePool.
/// @dev Mint authority is held by the deploying PulsePool.
///      After graduation, mint authority is revoked (token supply forever fixed).
///      Equivalent to the SPL token in the Solana version.
contract PulseToken is ERC20, ERC20Burnable {
    /// @notice The PulsePool that created this token (holds mint authority)
    address public immutable pool;

    /// @notice Whether minting has been permanently revoked
    bool public mintRevoked;

    /// @notice Timestamp of creation
    uint256 public immutable createdAt;

    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    event MintRevoked();

    // ═══════════════════════════════════════════════════════════════════════════
    // MODIFIERS
    // ═══════════════════════════════════════════════════════════════════════════

    modifier onlyPool() {
        require(msg.sender == pool, "PulseToken: only pool");
        _;
    }

    modifier canMint() {
        require(!mintRevoked, "PulseToken: mint revoked");
        _;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════════════════

    constructor(
        string memory name,
        string memory symbol,
        address _pool
    ) ERC20(name, symbol) {
        require(_pool != address(0), "PulseToken: zero pool");
        pool = _pool;
        createdAt = block.timestamp;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MINTING (Pool Only)
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Mint tokens. Only callable by the PulsePool.
    /// @param to Recipient address
    /// @param amount Amount to mint (in 18-decimal wei)
    function mint(address to, uint256 amount) external onlyPool canMint {
        _mint(to, amount);
    }

    /// @notice Permanently revoke mint authority.
    /// @dev Called by PulsePool during initializePool().
    ///      Once called, token supply is FOREVER FIXED.
    function revokeMint() external onlyPool {
        require(!mintRevoked, "PulseToken: already revoked");
        mintRevoked = true;
        emit MintRevoked();
    }

    /// @notice Token decimals — always 18 for EVM.
    /// @dev Solana uses 6 decimals. EVM standard is 18.
    ///      Token amounts are scaled up: Solana's 700M * 10^6 = EVM's 700M * 10^18.
    function decimals() public pure override returns (uint8) {
        return 18;
    }
}
