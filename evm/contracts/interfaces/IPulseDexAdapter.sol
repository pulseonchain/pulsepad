// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title IPulseDexAdapter
/// @notice Common interface for all DEX adapters.
/// @dev Each chain implements its own adapters. The PulsePool calls this
///      interface during migrate() to deploy liquidity to the target DEX.
interface IPulseDexAdapter {
    /// @notice Deploy a new liquidity pool on the target DEX and add liquidity.
    /// @param token The ERC-20 token address
    /// @param tokenAmount Amount of tokens to deposit
    /// @param nativeAmount Amount of native tokens (BNB/ETH) to deposit (sent as msg.value)
    /// @param creator Creator address (receives LP tokens/NFT)
    /// @return pool The DEX pool/liquidity position address
    /// @return liquidityTickets Amount of LP tokens/NFT ID received
    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable returns (address pool, uint256 liquidityTickets);

    /// @notice Claim LP fees from a previously deployed pool.
    /// @param pool The DEX pool address
    /// @param token The ERC-20 token address
    /// @return nativeFees Native tokens claimed (BNB/ETH in wei)
    /// @return tokenFees Token fees claimed
    function claimLpFees(
        address pool,
        address token
    ) external returns (uint256 nativeFees, uint256 tokenFees);

    /// @notice Get the DEX factory address (for CREATE2 salt verification).
    function dexFactory() external view returns (address);

    /// @notice Predict the pool address before deployment.
    /// @param token The ERC-20 token address
    /// @return pool The predicted DEX pool address
    function predictPoolAddress(address token) external view returns (address pool);

    /// @notice Returns the DEX name for event logging.
    function dexName() external pure returns (string memory);
}

/// @title IPancakeSwapV2Router
/// @notice Minimal interface for PancakeSwap V2 Router (used by adapter).
interface IPancakeSwapV2Router {
    function addLiquidityETH(
        address token,
        uint256 amountTokenDesired,
        uint256 amountTokenMin,
        uint256 amountETHMin,
        address to,
        uint256 deadline
    ) external payable returns (uint256 amountToken, uint256 amountETH, uint256 liquidity);

    function factory() external pure returns (address);
    function WETH() external pure returns (address);
}

/// @title IPancakeSwapV2Factory
/// @notice Minimal interface for PancakeSwap V2 Factory.
interface IPancakeSwapV2Factory {
    function createPair(address tokenA, address tokenB) external returns (address pair);
    function getPair(address tokenA, address tokenB) external view returns (address pair);
}

/// @title IUniswapV3Factory
/// @notice Minimal interface for Uniswap V3 Factory.
interface IUniswapV3Factory {
    function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool);
    function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool);
}

/// @title INonfungiblePositionManager
/// @notice Minimal interface for Uniswap V3 Position Manager.
interface INonfungiblePositionManager {
    struct MintParams {
        address token0;
        address token1;
        uint24 fee;
        int24 tickLower;
        int24 tickUpper;
        uint256 amount0Desired;
        uint256 amount1Desired;
        uint256 amount0Min;
        uint256 amount1Min;
        address recipient;
        uint256 deadline;
    }

    function mint(MintParams calldata params) external payable returns (
        uint256 tokenId,
        uint128 liquidity,
        uint256 amount0,
        uint256 amount1
    );

    function collect(uint256 tokenId, address recipient, uint128 amount0Max, uint128 amount1Max)
        external returns (uint256 amount0, uint256 amount1);
}

/// @title IAerodromeFactory
/// @notice Minimal interface for Aerodrome Factory (Base chain).
interface IAerodromeFactory {
    function createPool(
        address tokenA,
        address tokenB,
        int24 tickSpacing
    ) external returns (address pool);

    function getPool(address tokenA, address tokenB, int24 tickSpacing)
        external view returns (address pool);
}

/// @title IAerodromePool
/// @notice Minimal interface for Aerodrome Pool (Concentrated Liquidity).
interface IAerodromePool {
    function mint(
        address recipient,
        int24 tickLower,
        int24 tickUpper,
        uint128 amount,
        bytes calldata data
    ) external returns (uint256 amount0, uint256 amount1);

    function collect(
        address recipient,
        int24 tickLower,
        int24 tickUpper,
        uint128 amount0Requested,
        uint128 amount1Requested
    ) external returns (uint128 amount0, uint128 amount1);

    function token0() external view returns (address);
    function token1() external view returns (address);
    function tickSpacing() external view returns (int24);
}

/// @title IAlgebraFactory
/// @notice Minimal interface for Thena V3 / Alien Base (Algebra fork) Factory.
interface IAlgebraFactory {
    function createPool(address tokenA, address tokenB) external returns (address pool);
    function poolByPair(address tokenA, address tokenB) external view returns (address pool);
}

/// @title IAlgebraPool
/// @notice Minimal interface for Algebra CL Pool.
interface IAlgebraPool {
    function mint(
        address sender,
        address recipient,
        int24 bottomTick,
        int24 topTick,
        uint128 liquidityDesired,
        bytes calldata data
    ) external returns (uint256 amount0, uint256 amount1, uint128 liquidityActual);
}
