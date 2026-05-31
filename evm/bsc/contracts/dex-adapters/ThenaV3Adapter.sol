// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title ThenaV3Adapter
/// @notice Pulse DEX adapter for Thena V3 (Algebra CL fork) on BSC.
/// @dev Thena V3 uses Algebra's concentrated liquidity model with ve(3,3).
contract ThenaV3Adapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    address public constant FACTORY = 0x306F06C147f064A010530292A0aAE5d8D230bC3d;
    address public constant ROUTER = 0xd4ae6eCA985340Dd434D38F470aCCcE4DC48866D;
    address public constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;

    int24 public constant TICK_LOWER = -887220;
    int24 public constant TICK_UPPER = 887220;

    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "ThenaV3: native mismatch");

        // Create pool
        pool = IAlgebraFactory(FACTORY).createPool(token, WBNB);

        // Transfer tokens
        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);

        // Convert BNB to WBNB
        IWBNB(WBNB).deposit{value: nativeAmount}();

        // Approve pool
        IERC20(token).forceApprove(pool, tokenAmount);
        IERC20(WBNB).forceApprove(pool, nativeAmount);

        // Determine amounts based on token order in the pool
        bool isToken0 = token < WBNB;
        uint256 amount0 = isToken0 ? tokenAmount : nativeAmount;
        uint256 amount1 = isToken0 ? nativeAmount : tokenAmount;

        // Calculate liquidity amount for full range
        // Full range = tickLower=-887272, tickUpper=887272
        uint128 liquidity = _calculateLiquidity(amount0, amount1);

        // Mint full-range position
        try IAlgebraPool(pool).mint(
            creator,        // recipient
            TICK_LOWER,
            TICK_UPPER,
            liquidity,
            ""               // no callback data
        ) returns (uint256, uint256, uint128 liquidityActual) {
            liquidityTickets = uint256(liquidityActual);
        } catch {
            // If mint fails after pool creation, tokens + BNB remain in adapter
            // Return them to creator for manual handling
            revert("ThenaV3: mint failed");
        }

        // Return dust
        _returnDust(token, creator);

        return (pool, liquidityTickets);
    }

    function claimLpFees(address, address) external pure override returns (uint256, uint256) {
        return (0, 0); // LP position is held by creator
    }

    function dexFactory() external pure override returns (address) { return FACTORY; }
    function dexName() external pure override returns (string memory) { return "Thena V3"; }

    function predictPoolAddress(address token) external view override returns (address) {
        return IAlgebraFactory(FACTORY).poolByPair(token, WBNB);
    }

    function _calculateLiquidity(
        uint256 amount0, uint256 amount1
    ) internal pure returns (uint128) {
        // Full range = sqrt(amount0 * amount1)
        // Simplified: use the geometric mean as liquidity estimate
        uint256 liquidity = _approxSqrt(amount0 * amount1);
        require(liquidity <= type(uint128).max, "ThenaV3: liquidity overflow");
        return uint128(liquidity);
    }

    function _returnDust(address token, address creator) internal {
        IERC20 t = IERC20(token);
        uint256 tBal = t.balanceOf(address(this));
        if (tBal > 0) t.safeTransfer(creator, tBal);

        uint256 wbnbBal = IERC20(WBNB).balanceOf(address(this));
        if (wbnbBal > 0) {
            IWBNB(WBNB).withdraw(wbnbBal);
            (bool ok, ) = creator.call{value: wbnbBal}("");
            require(ok, "ThenaV3: bnb return failed");
        }
    }

    function _approxSqrt(uint256 y) internal pure returns (uint256 z) {
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
