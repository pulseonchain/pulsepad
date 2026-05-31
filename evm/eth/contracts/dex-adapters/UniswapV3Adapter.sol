// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title UniswapV3Adapter
/// @notice Pulse DEX adapter for Uniswap V3 on Ethereum Mainnet.
/// @dev Mints a full-range position via NonfungiblePositionManager.
///      This is the PRIMARY migration target on ETH.
contract UniswapV3Adapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    address public constant FACTORY = 0x1F98431c8aD98523631AE4a59f267346ea31F984;
    address public constant NFT_MANAGER = 0xC36442b4a4522E871399CD717aBDD847Ab11FE88;
    address public constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    uint24 public constant FEE_TIER = 10000;

    int24 public constant TICK_LOWER = -887220;
    int24 public constant TICK_UPPER = 887220;

    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "UniV3: native mismatch");

        bool isToken0 = token < WETH;
        (address token0, address token1, uint256 amount0, uint256 amount1) =
            isToken0 ? (token, WETH, tokenAmount, nativeAmount) : (WETH, token, nativeAmount, tokenAmount);

        pool = IUniswapV3Factory(FACTORY).getPool(token0, token1, FEE_TIER);
        if (pool == address(0)) {
            pool = IUniswapV3Factory(FACTORY).createPool(token0, token1, FEE_TIER);
            uint160 sqrtPriceX96 = _calcSqrtPriceX96(amount0, amount1, isToken0);
            IUniswapV3Pool(pool).initialize(sqrtPriceX96);
        }

        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);
        IWETH(WETH).deposit{value: nativeAmount}();

        IERC20(token0).forceApprove(NFT_MANAGER, amount0);
        IERC20(token1).forceApprove(NFT_MANAGER, amount1);

        INonfungiblePositionManager.MintParams memory params = INonfungiblePositionManager.MintParams({
            token0: token0, token1: token1, fee: FEE_TIER,
            tickLower: TICK_LOWER, tickUpper: TICK_UPPER,
            amount0Desired: amount0, amount1Desired: amount1,
            amount0Min: 0, amount1Min: 0,
            recipient: creator, deadline: block.timestamp + 300
        });

        (uint256 tokenId, , uint256 r0, uint256 r1) = INonfungiblePositionManager(NFT_MANAGER).mint(params);

        if (r0 < amount0) IERC20(token0).safeTransfer(creator, amount0 - r0);
        if (r1 < amount1) IERC20(token1).safeTransfer(creator, amount1 - r1);

        uint256 wethBal = IERC20(WETH).balanceOf(address(this));
        if (wethBal > 0) { IWETH(WETH).withdraw(wethBal); (bool ok, ) = creator.call{value: wethBal}(""); require(ok); }

        return (pool, tokenId);
    }

    function claimLpFees(address, address) external pure override returns (uint256, uint256) {
        return (0, 0);
    }

    function dexFactory() external pure override returns (address) { return FACTORY; }
    function dexName() external pure override returns (string memory) { return "Uniswap V3"; }

    function predictPoolAddress(address token) external view override returns (address) {
        bool isToken0 = token < WETH;
        (address t0, address t1) = isToken0 ? (token, WETH) : (WETH, token);
        return IUniswapV3Factory(FACTORY).getPool(t0, t1, FEE_TIER);
    }

    function _calcSqrtPriceX96(uint256 a0, uint256 a1, bool isToken0) internal pure returns (uint160) {
        uint256 price = isToken0 ? (a1 * 1e18) / a0 : (a0 * 1e18) / a1;
        uint256 sqrtPrice = _sqrt(price * 2**192 / 1e18);
        require(sqrtPrice <= type(uint160).max, "UniV3: sqrt overflow");
        return uint160(sqrtPrice);
    }

    function _sqrt(uint256 y) internal pure returns (uint256 z) {
        if (y > 3) { z = y; uint256 x = y / 2 + 1; while (x < z) { z = x; x = (y / x + x) / 2; } }
        else if (y != 0) { z = 1; }
    }
}

interface IUniswapV3Factory {
    function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool);
    function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool);
}
interface IUniswapV3Pool { function initialize(uint160 sqrtPriceX96) external; }
interface INonfungiblePositionManager {
    struct MintParams {
        address token0; address token1; uint24 fee;
        int24 tickLower; int24 tickUpper;
        uint256 amount0Desired; uint256 amount1Desired;
        uint256 amount0Min; uint256 amount1Min;
        address recipient; uint256 deadline;
    }
    function mint(MintParams calldata params) external payable returns (
        uint256 tokenId, uint128 liquidity, uint256 amount0, uint256 amount1
    );
}
interface IWETH { function deposit() external payable; function withdraw(uint256) external; }
