// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IPulseDexAdapter} from "../../../contracts/interfaces/IPulseDexAdapter.sol";

/// @title UniswapV2Adapter
/// @notice Pulse DEX adapter for Uniswap V2 on Ethereum Mainnet.
/// @dev Deploys an ERC-20/ETH pair and adds liquidity.
contract UniswapV2Adapter is IPulseDexAdapter {
    using SafeERC20 for IERC20;

    address public constant FACTORY = 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f;
    address public constant ROUTER = 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D;
    address public constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

    function deployPoolAndAddLiquidity(
        address token,
        uint256 tokenAmount,
        uint256 nativeAmount,
        address creator
    ) external payable override returns (address pool, uint256 liquidityTickets) {
        require(msg.value == nativeAmount, "UniV2: native mismatch");

        pool = IUniswapV2Factory(FACTORY).getPair(token, WETH);
        if (pool == address(0)) {
            pool = IUniswapV2Factory(FACTORY).createPair(token, WETH);
        }

        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);
        IERC20(token).forceApprove(ROUTER, tokenAmount);

        (uint256 amountToken, , uint256 liquidity) =
        IUniswapV2Router(ROUTER).addLiquidityETH{value: nativeAmount}(
            token, tokenAmount,
            tokenAmount * 95 / 100,   // 5% slippage (ETH is higher precision)
            nativeAmount * 95 / 100,  // 5% slippage
            creator,
            block.timestamp + 300
        );

        uint256 tokenDust = tokenAmount - amountToken;
        if (tokenDust > 0) IERC20(token).safeTransfer(creator, tokenDust);

        return (pool, liquidity);
    }

    function claimLpFees(address, address) external pure override returns (uint256, uint256) {
        return (0, 0);
    }

    function dexFactory() external pure override returns (address) { return FACTORY; }
    function dexName() external pure override returns (string memory) { return "Uniswap V2"; }

    function predictPoolAddress(address token) external view override returns (address) {
        return IUniswapV2Factory(FACTORY).getPair(token, WETH);
    }
}

interface IUniswapV2Router {
    function addLiquidityETH(
        address token, uint256 amountTokenDesired, uint256 amountTokenMin,
        uint256 amountETHMin, address to, uint256 deadline
    ) external payable returns (uint256 amountToken, uint256 amountETH, uint256 liquidity);
}
interface IUniswapV2Factory {
    function createPair(address tokenA, address tokenB) external returns (address pair);
    function getPair(address tokenA, address tokenB) external view returns (address pair);
}
