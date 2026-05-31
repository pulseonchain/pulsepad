// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title PancakeSwapV3Adapter
/// @notice Pulse DEX adapter for PancakeSwap V3 (Concentrated Liquidity) on BSC.
/// @dev Mints a full-range position via NonfungiblePositionManager.
contract PancakeSwapV3Adapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    // ═══════════════════════════════════════════════════════════════════════════
    // CONSTANTS (BSC Mainnet)
    // ═══════════════════════════════════════════════════════════════════════════

    address public constant FACTORY = 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865;
    address public constant NFT_MANAGER = 0x46A15B0b27311cedF172AB29E4f4766fbE7F4364;
    address public constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
    uint24 public constant FEE_TIER = 10000; // 1% — high volatility for new tokens

    /// @notice Number of positions minted (tracks NFT IDs)
    uint256 public positionCount;

    // ═══════════════════════════════════════════════════════════════════════════
    // IPulseDexAdapter Implementation
    // ═══════════════════════════════════════════════════════════════════════════

    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "PCSV3: native amount mismatch");

        // Determine token order (token0 = smaller address)
        bool isToken0 = token < WBNB;
        (address token0, address token1, uint256 amount0, uint256 amount1) =
            isToken0
                ? (token, WBNB, tokenAmount, nativeAmount)
                : (WBNB, token, nativeAmount, tokenAmount);

        // Create pool if not exists
        pool = IUniswapV3Factory(FACTORY).getPool(token0, token1, FEE_TIER);
        if (pool == address(0)) {
            pool = IUniswapV3Factory(FACTORY).createPool(token0, token1, FEE_TIER);
            // Initialize pool with sqrt price
            uint160 sqrtPriceX96 = _calculateSqrtPriceX96(amount0, amount1, isToken0);
            IUniswapV3Pool(pool).initialize(sqrtPriceX96);
        }

        // Transfer tokens from PulsePool
        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);

        // Convert BNB to WBNB
        IWBNB(WBNB).deposit{value: nativeAmount}();

        // Approve NFT Manager
        IERC20(token0).forceApprove(NFT_MANAGER, amount0);
        IERC20(token1).forceApprove(NFT_MANAGER, amount1);

        // Mint full-range position
        INonfungiblePositionManager.MintParams memory params = INonfungiblePositionManager.MintParams({\n            token0: token0,
            token1: token1,
            fee: FEE_TIER,
            tickLower: -887220,
            tickUpper: 887220,
            amount0Desired: amount0,
            amount1Desired: amount1,
            amount0Min: 0,
            amount1Min: 0,
            recipient: creator,  // NFT goes to creator
            deadline: block.timestamp + 300
        });

        (uint256 tokenId, , uint256 returned0, uint256 returned1) =
            INonfungiblePositionManager(NFT_MANAGER).mint(params);

        positionCount++;

        // Return dust
        if (returned0 < amount0) {
            IERC20(token0).safeTransfer(creator, amount0 - returned0);
        }
        if (returned1 < amount1) {
            IERC20(token1).safeTransfer(creator, amount1 - returned1);
        }

        // Unwrap remaining WBNB and return as native
        uint256 wbnbBalance = IERC20(WBNB).balanceOf(address(this));
        if (wbnbBalance > 0) {
            IWBNB(WBNB).withdraw(wbnbBalance);
            (bool ok, ) = creator.call{value: wbnbBalance}("");
            require(ok, "PCSV3: dust return failed");
        }

        return (pool, tokenId);
    }

    function claimLpFees(
        address /* pool */,
        address /* token */
    ) external pure override returns (uint256 nativeFees, uint256 tokenFees) {
        // V3 fees require the NFT to claim, and NFT is held by creator.
        // Creator can claim their own LP fees via the Uniswap V3 interface.
        return (0, 0);
    }

    function dexFactory() external pure override returns (address) {
        return FACTORY;
    }

    function predictPoolAddress(address token) external view override returns (address) {
        bool isToken0 = token < WBNB;
        (address token0, address token1) = isToken0 ? (token, WBNB) : (WBNB, token);
        return IUniswapV3Factory(FACTORY).getPool(token0, token1, FEE_TIER);
    }

    function dexName() external pure override returns (string memory) {
        return "PancakeSwap V3";
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // INTERNAL
    // ═══════════════════════════════════════════════════════════════════════════

    /// @notice Calculate sqrtPriceX96 for pool initialization.
    /// @dev sqrt(price) * 2^96 where price = token1/token0
    function _calculateSqrtPriceX96(
        uint256 amount0,
        uint256 amount1,
        bool isToken0
    ) internal pure returns (uint160) {
        uint256 price = isToken0
            ? (amount1 * 1e18) / amount0
            : (amount0 * 1e18) / amount1;
        // sqrt(price) * 2^96
        uint256 sqrtPrice = _sqrt(price * 2**192 / 1e18);
        require(sqrtPrice <= type(uint160).max, "PCSV3: sqrt overflow");
        return uint160(sqrtPrice);
    }

    /// @notice Babylonian method for sqrt (precision: floor).
    function _sqrt(uint256 y) internal pure returns (uint256 z) {
        if (y > 3) {
            z = y;
            uint256 x = y / 2 + 1;
            while (x < z) {
                z = x;
                x = (y / x + x) / 2;
            }
        } else if (y != 0) {
            z = 1;
        }
    }
}

/// Internal interfaces
interface IUniswapV3Factory {
    function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool);
    function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool);
}
interface IUniswapV3Pool {
    function initialize(uint160 sqrtPriceX96) external;
}
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
interface IWBNB {
    function deposit() external payable;
    function withdraw(uint256) external;
}
