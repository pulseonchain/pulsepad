// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library PulseConstants {
    address constant PLATFORM_WALLET = 0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA;
    uint256 constant TOTAL_SUPPLY        = 1_097_052_391 * 1e18;
    uint256 constant BONDING_SUPPLY      = 700_000_000 * 1e18;
    uint256 constant RESERVE_SUPPLY      =  97_052_391 * 1e18;
    uint256 constant LP_RESERVE_SUPPLY   = 300_000_000 * 1e18;
    uint256 constant INITIAL_VIRTUAL_NATIVE = 30 ether;
    uint256 constant INITIAL_VIRTUAL_TOKEN  = 1_073_000_000 * 1e18;
    uint256 constant TOTAL_FEE_BPS    = 100;
    uint256 constant PLATFORM_FRACTION = 75;
    uint256 constant BPS_DENOMINATOR  = 10_000;

    // ── Tier Thresholds (per chain) ─────────────────────────────────────────
    // Fast | Standard | Stable
    // BSC: 15 BNB ($10.7K) | 35 BNB ($24.9K) | 50 BNB ($35.6K)
    uint256 constant BSC_FAST     = 15 ether;
    uint256 constant BSC_STANDARD = 35 ether;
    uint256 constant BSC_STABLE   = 50 ether;
    // ETH: 9 ETH ($18K) | 16 ETH ($32K) | 30 ETH ($60K)
    uint256 constant ETH_FAST     = 9 ether;
    uint256 constant ETH_STANDARD = 16 ether;
    uint256 constant ETH_STABLE   = 30 ether;
    // Base: same as ETH (OP Stack L2, native token = ETH)
    uint256 constant BASE_FAST     = 9 ether;
    uint256 constant BASE_STANDARD = 16 ether;
    uint256 constant BASE_STABLE   = 30 ether;

    // ── Anti-Snipe ──────────────────────────────────────────────────────────
    uint256 constant ANTI_SNIPE_SECS = 180;
    uint256 constant ANTI_SNIPE_MULTIPLIER = 3;

    // ── Agent / Vault ───────────────────────────────────────────────────────
    uint256 constant MAX_VAULT_PER_24H = 500_000 * 1e18;
    uint256 constant VAULT_COOLDOWN = 24 hours;
    uint256 constant AGENT_CLAIM_COOLDOWN = 3 hours;

    // ── Fee Range ───────────────────────────────────────────────────────────
    uint256 constant MIN_FEE_BPS = 100;
    uint256 constant MAX_FEE_BPS = 500;
}

// ─── Tier Enum ───────────────────────────────────────────────────────────────

enum PulseTier { Fast, Standard, Stable }

// ─── Address Libraries ────────────────────────────────────────────────────────

library BSCAddresses {
    address constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
    address constant PCS_V2_FACTORY = 0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73;
    address constant PCS_V2_ROUTER  = 0x10ED43C718714eb63d5aA57B78B54704E256024E;
    address constant PCS_V3_FACTORY    = 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865;
    address constant PCS_V3_SWAP_ROUTER= 0x13f4EA83D0bd40E75C8222255bc855a974568Dd4;
    address constant PCS_V3_NFT_MGR    = 0x46A15B0b27311cedF172AB29E4f4766fbE7F4364;
    address constant THENA_V3_FACTORY = 0x306F06C147f064A010530292A0aAE5d8D230bC3d;
    address constant THENA_V3_ROUTER  = 0xd4ae6eCA985340Dd434D38F470aCCcE4DC48866D;
    address constant BISWAP_V3_FACTORY = 0x7C3d53606f9c03E262f1B7Ea2C2149B6d65D8b11;
    address constant BISWAP_V3_ROUTER  = 0x237c0c520c8A6E2c8EE0dFdA41eFD7693C4c7f20;
}

library ETHAddresses {
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant UNI_V2_FACTORY = 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f;
    address constant UNI_V2_ROUTER  = 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D;
    address constant UNI_V3_FACTORY    = 0x1F98431c8aD98523631AE4a59f267346ea31F984;
    address constant UNI_V3_ROUTER     = 0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45;
    address constant UNI_V3_NFT_MGR    = 0xC36442b4a4522E871399CD717aBDD847Ab11FE88;
    address constant SUSHI_V2_FACTORY = 0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac;
    address constant SUSHI_V2_ROUTER  = 0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F;
    address constant BALANCER_VAULT = 0xBA12222222228d8Ba445958a75a0704d566BF2C8;
}

library BaseAddresses {
    address constant WETH = 0x4200000000000000000000000000000000000006;
    address constant AERODROME_FACTORY = 0x420DD381b31aEf6683db6B902084cB0FFECe40Da;
    address constant AERODROME_ROUTER  = 0xcF77a3Ba9A5CA399B7c97c74d54e5b1Beb874E43;
    address constant VE_AERO           = 0xeBf418Fe2512e7E6bd9b87a8F0f294aCDC67e6B4;
    address constant ALIEN_BASE_FACTORY = 0x3B9726B169D0FEF2FBF740D525a226fA283ef3F7;
    address constant ALIEN_BASE_ROUTER  = 0x8c1A3cF8f83074169FE5D7aD50B978e1cD6b37c7;
    address constant UNI_V3_BASE_FACTORY = 0x33128a8fC17869897dcE68Ed026d694621f6FDfD;
    address constant UNI_V3_BASE_NFT_MGR = 0x03a520b32C04BF3bEEf7BEb72E919cf822Ed34f1;
}