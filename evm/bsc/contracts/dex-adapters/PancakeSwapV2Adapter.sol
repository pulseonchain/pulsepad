// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title PancakeSwapV2Adapter
/// @notice Pulse DEX adapter for PancakeSwap V2 on BNB Smart Chain.
/// @dev Deploys an ERC-20/BNB pair and adds liquidity.
contract PancakeSwapV2Adapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTANTS (BSC Mainnet)
    // ═══════════════════════════════════════════════════════════════════════════

    address public constant FACTORY = 0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73;
    address public constant ROUTER = 0x10ED43C718714eb63d5aA57B78B54704E256024E;
    address public constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;

    // ═══════════════════════════════════════════════════════════════════════════
    // IPulseDexAdapter Implementation
    // ═══════════════════════════════════════════════════════════════════════════

    /// @inheritdoc IPulseDexAdapter
    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "PCSV2: native amount mismatch");

        // Check if pair already exists
        pool = IPancakeSwapV2Factory(FACTORY).getPair(token, WBNB);
        if (pool == address(0)) {
            pool = IPancakeSwapV2Factory(FACTORY).createPair(token, WBNB);
        }

        // Transfer tokens from PulsePool (caller)
        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);

        // Approve router
        IERC20(token).forceApprove(ROUTER, tokenAmount);

        // Add liquidity
        (uint256 amountToken, uint256 amountBNB, uint256 liquidity) =
        IPancakeSwapV2Router(ROUTER).addLiquidityETH{value: nativeAmount}(
            token,
            tokenAmount,
            tokenAmount * 95 / 100,  // 5% slippage (BSC-standard)
            nativeAmount * 95 / 100,  // 5% slippage
            creator,                   // LP tokens → creator
            block.timestamp + 300      // 5 minute deadline
        );

        // Return dust if any (due to slippage)
        uint256 tokenDust = tokenAmount - amountToken;
        if (tokenDust > 0) {
            IERC20(token).safeTransfer(creator, tokenDust);
        }
        uint256 bnbDust = nativeAmount - amountBNB;
        if (bnbDust > 0) {
            (bool ok, ) = creator.call{value: bnbDust}("");
            require(ok, "PCSV2: dust return failed");
        }

        return (pool, liquidity);
    }

    /// @inheritdoc IPulseDexAdapter
    function claimLpFees(
        address /* pool */,
        address token
    ) external override returns (uint256 nativeFees, uint256 tokenFees) {
        // PancakeSwap V2 LP fees are auto-compounded into the LP token.
        // To claim, you must remove liquidity, take the fees, then re-add.
        // This is complex and rare — PulsePool only holds LP for the creator.
        // The creator can withdraw LP tokens and then remove liquidity themselves.
        // For now, return 0 — LP tokens are sent to creator directly.
        return (0, 0);
    }

    /// @inheritdoc IPulseDexAdapter
    function dexFactory() external pure override returns (address) {
        return FACTORY;
    }

    /// @inheritdoc IPulseDexAdapter
    function predictPoolAddress(address token) external view override returns (address) {
        return IPancakeSwapV2Factory(FACTORY).getPair(token, WBNB);
    }

    /// @inheritdoc IPulseDexAdapter
    function dexName() external pure override returns (string memory) {
        return "PancakeSwap V2";
    }
}

/// Minimal inline interfaces (avoid external Imports for simplicity)
interface IPancakeSwapV2Router {
    function addLiquidityETH(
        address token, uint256 amountTokenDesired, uint256 amountTokenMin,
        uint256 amountETHMin, address to, uint256 deadline
    ) external payable returns (uint256 amountToken, uint256 amountETH, uint256 liquidity);
}
interface IPancakeSwapV2Factory {
    function createPair(address tokenA, address tokenB) external returns (address pair);
    function getPair(address tokenA, address tokenB) external view returns (address pair);
}
