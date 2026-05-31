// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title BiswapV3Adapter
/// @notice Pulse DEX adapter for Biswap V3 on BSC.
/// @dev Biswap V3 uses the same Algebra CL fork as Thena V3.
contract BiswapV3Adapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    address public constant FACTORY = 0x7C3d53606f9c03E262f1B7Ea2C2149B6d65D8b11;
    address public constant ROUTER = 0x237c0c520c8A6E2c8EE0dFdA41eFD7693C4c7f20;
    address public constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;

    int24 public constant TICK_LOWER = -887220;
    int24 public constant TICK_UPPER = 887220;

    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable override returns (address pool, uint256) {
        require(msg.value == nativeAmount, "BiswapV3: native mismatch");

        pool = IAlgebraFactory(FACTORY).createPool(token, WBNB);
        require(pool != address(0), "BiswapV3: pool creation failed");

        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);
        IWBNB(WBNB).deposit{value: nativeAmount}();

        IERC20(token).forceApprove(pool, tokenAmount);
        IERC20(WBNB).forceApprove(pool, nativeAmount);

        uint128 liquidity = _calcLiquidity(tokenAmount, nativeAmount);
        IAlgebraPool(pool).mint(creator, TICK_LOWER, TICK_UPPER, liquidity, "");

        _returnDust(token, creator);
        return (pool, uint256(liquidity));
    }

    function claimLpFees(address, address) external pure override returns (uint256, uint256) {
        return (0, 0);
    }

    function dexFactory() external pure override returns (address) { return FACTORY; }
    function dexName() external pure override returns (string memory) { return "Biswap V3"; }

    function predictPoolAddress(address token) external view override returns (address) {
        return IAlgebraFactory(FACTORY).poolByPair(token, WBNB);
    }

    /// @notice Approximate full-range liquidity from token/native amounts.
    function _calcLiquidity(uint256 a0, uint256 a1) internal pure returns (uint128) {
        // Full-range liquidity ≈ sqrt(amount0 * amount1) / tickSpacingFactor
        // Simplified: use geometric mean
        uint256 product = a0 * a1;
        uint256 sqrt = _sqrt(product);
        sqrt = sqrt / 1e9; // Scale down for practical liquidity values
        require(sqrt <= type(uint128).max, "BiswapV3: overflow");
        return uint128(sqrt);
    }

    function _returnDust(address token, address creator) internal {
        uint256 tBal = IERC20(token).balanceOf(address(this));
        if (tBal > 0) IERC20(token).safeTransfer(creator, tBal);
        uint256 wbnbBal = IERC20(WBNB).balanceOf(address(this));
        if (wbnbBal > 0) {
            IWBNB(WBNB).withdraw(wbnbBal);
            (bool ok, ) = creator.call{value: wbnbBal}("");
            require(ok, "BiswapV3: dust return failed");
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
interface IWBNB {
    function deposit() external payable;
    function withdraw(uint256) external;
}
