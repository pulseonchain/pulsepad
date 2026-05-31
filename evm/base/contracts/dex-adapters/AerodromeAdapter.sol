// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title AerodromeAdapter
/// @notice Pulse DEX adapter for Aerodrome on Base Chain.
/// @dev Aerodrome is the PRIMARY DEX on Base. Uses a modified Velodrome V2 codebase
///      with ve(3,3) flywheel. Supports both Volatile (CPAMM) and CL pools.
contract AerodromeAdapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    address public constant FACTORY = 0x420DD381b31aEf6683db6B902084cB0FFECe40Da;
    address public constant ROUTER = 0xcF77a3Ba9A5CA399B7c97c74d54e5b1Beb874E43;
    address public constant WETH_BASE = 0x4200000000000000000000000000000000000006;

    /// @notice Use Volatile pool type (constant product, like our curve)
    bool public constant STABLE = false;

    function deployPoolAndAddLiquidity(
        address token, uint256 tokenAmount, uint256 nativeAmount, address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "Aerodrome: native mismatch");

        // Aerodrome Router handles pool creation + liquidity in one call
        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);
        IERC20(token).forceApprove(ROUTER, tokenAmount);

        (uint256 amountToken, , uint256 liquidity) =
        IAerodromeRouter(ROUTER).addLiquidityETH{value: nativeAmount}(
            token,
            STABLE,   // false = volatile pair
            tokenAmount,
            tokenAmount * 95 / 100,
            nativeAmount * 95 / 100,
            creator,
            block.timestamp + 300
        );

        // Get pool address from factory
        pool = IAerodromeFactory(FACTORY).getPool(token, WETH_BASE, STABLE);
        require(pool != address(0), "Aerodrome: pool not found");

        uint256 dust = tokenAmount - amountToken;
        if (dust > 0) IERC20(token).safeTransfer(creator, dust);

        return (pool, liquidity);
    }

    function claimLpFees(address, address) external pure override returns (uint256, uint256) {
        return (0, 0); // LP tokens sent to creator directly
    }

    function dexFactory() external pure override returns (address) { return FACTORY; }
    function dexName() external pure override returns (string memory) { return "Aerodrome"; }

    /// @dev Uses Aerodrome's getPool which takes (tokenA, tokenB, stable)
    function predictPoolAddress(address token) external view override returns (address) {
        return IAerodromeFactory(FACTORY).getPool(token, WETH_BASE, STABLE);
    }
}

interface IAerodromeRouter {
    function addLiquidityETH(
        address token, bool stable,
        uint256 amountTokenDesired, uint256 amountTokenMin,
        uint256 amountETHMin, address to, uint256 deadline
    ) external payable returns (uint256 amountToken, uint256 amountETH, uint256 liquidity);
}
interface IAerodromeFactory {
    function getPool(address tokenA, address tokenB, bool stable) external view returns (address);
}
