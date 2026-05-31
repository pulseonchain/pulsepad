// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title AlienBaseAdapter
/// @notice Pulse DEX adapter for Alien Base on Base Chain.
/// @dev Alien Base uses Algebra's CL fork (same as Thena V3 on BSC).
contract AlienBaseAdapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    address public constant FACTORY = 0x3B9726B169D0FEF2FBF740D525a226fA283ef3F7;
    address public constant ROUTER = 0x8c1A3cF8f83074169FE5D7aD50B978e1cD6b37c7;
    address public constant WETH_BASE = 0x4200000000000000000000000000000000000006;

    int24 public constant TICK_LOWER = -887220;
    int24 public constant TICK_UPPER = 887220;

    function deployPoolAndAddLiquidity(
        address token, uint256 tokenAmount, uint256 nativeAmount, address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "AlienBase: native mismatch");

        pool = IAlgebraFactory(FACTORY).createPool(token, WETH_BASE);
        require(pool != address(0), "AlienBase: pool creation failed");

        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);
        IWETH(WETH_BASE).deposit{value: nativeAmount}();

        IERC20(token).forceApprove(pool, tokenAmount);
        IERC20(WETH_BASE).forceApprove(pool, nativeAmount);

        uint128 liquidity = _calcLiquidity(tokenAmount, nativeAmount);
        IAlgebraPool(pool).mint(creator, TICK_LOWER, TICK_UPPER, liquidity, "");

        _returnDust(token, creator);
        return (pool, uint256(liquidity));
    }

    function claimLpFees(address, address) external pure override returns (uint256, uint256) {
        return (0, 0);
    }

    function dexFactory() external pure override returns (address) { return FACTORY; }
    function dexName() external pure override returns (string memory) { return "Alien Base"; }

    function predictPoolAddress(address token) external view override returns (address) {
        return IAlgebraFactory(FACTORY).poolByPair(token, WETH_BASE);
    }

    function _calcLiquidity(uint256 a0, uint256 a1) internal pure returns (uint128) {
        uint256 product = a0 * a1;
        uint256 sqrt = _sqrt(product);
        sqrt = sqrt / 1e9;
        require(sqrt <= type(uint128).max, "AlienBase: overflow");
        return uint128(sqrt);
    }

    function _returnDust(address token, address creator) internal {
        uint256 tBal = IERC20(token).balanceOf(address(this));
        if (tBal > 0) IERC20(token).safeTransfer(creator, tBal);
        uint256 wethBal = IERC20(WETH_BASE).balanceOf(address(this));
        if (wethBal > 0) {
            IWETH(WETH_BASE).withdraw(wethBal);
            (bool ok, ) = creator.call{value: wethBal}("");
            require(ok, "AlienBase: dust return failed");
        }
    }

    function _sqrt(uint256 y) internal pure returns (uint256 z) {
        if (y > 3) { z = y; uint256 x = y / 2 + 1; while (x < z) { z = x; x = (y / x + x) / 2; } }
        else if (y != 0) { z = 1; }
    }
}

interface IAlgebraFactory {
    function createPool(address tokenA, address tokenB) external returns (address pool);
    function poolByPair(address tokenA, address tokenB) external view returns (address pool);
}
interface IAlgebraPool {
    function mint(address sender, address recipient, int24 bottomTick, int24 topTick,
        uint128 liquidityDesired, bytes calldata data
    ) external returns (uint256 amount0, uint256 amount1, uint128 liquidityActual);
}
interface IWETH { function deposit() external payable; function withdraw(uint256) external; }
