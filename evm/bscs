

Pulse on 
BNB Smart Chain  — a constant-product bonding curve protocol built in Solidity that lets any creator launch a token, run it through a bonding curve, and graduate it to PancakeSwap, Thena, or Biswap — not just one DEX.

Each chain gets one `PulseFactory`, bonded tokens can migrate to four DEX launchpads on BSC, and the entire protocol feeds into a cross-chain DAO governed by top token holders.

---

Website: https://pulse.jelly-os.xyz/

AGENT INFRA -> SOON 
---

##  BSC Edition

```
            ┌─────────────────────────────────────────┐
            │           PULSE ON BSC                  │
            │    (one PulseFactory on BNB Chain)      │
            │                                         │
            │  Creator launches ──►  CP curve trades  │
            │              20, 50 or 85 BNB raised    │
            │                           │             │
            │                ┌──────────┼──────────┐  │
            │                ▼          ▼          ▼  │
            │          PancakeSwap  PancakeSwap  Thena│
            │             V2          V3          V3  │
            │                       Biswap V3         │
            └─────────────────────────────────────────┘
                                  │
                                  ▼
                     ┌────────────────────────┐
                     │  COMMUNITY BOARD (DAO) │
                     │  Cross-chain top       │
                     │  holders govern Pulse  │
                     └────────────────────────┘
```

1. **One bonding curve per token** — Each token gets its own `PulsePool` deployed via CREATE2 by `PulseFactory` on BSC.
2. **Graduate to any BSC DEX** — When the bonding curve hits 85 BNB, it migrates liquidity to PancakeSwap V2, PancakeSwap V3, Thena V3, or Biswap V3.
3. **BSC-native experience** — 3-second blocks, ~$0.15 per trade in gas, massive retail user base in Asia.
4. **DAO governance** — Top token holders across ALL chains form a Community Board.
5. **Fees fund development** — 1% per trade: 0.75% to `0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA`, 0.25% to creator.

---

## Current Status: Contracts Complete

Full plan and audit: [PLAN.md](./PLAN.md)

- `PulseGlobalConfig` — singleton config with 85 BNB graduation threshold
- `PulseFactory` — CREATE2 factory on BSC (Chain 56)
- `PulsePool` — full bonding curve + ERC-20 + staking + migration
- `PulseToken` — ERC-20 with mint authority held by pool
- 4 DEX adapters for BSC

---

## Architecture

### Token Supply

| Bucket             | Amount         | Purpose                                        |
| ------------------ | -------------- | ------------------------------------------------ |
| Bonding Supply     | 700,000,000    | Sold via the bonding curve                       |
| Reserve Supply     | 97,052,391     | Guarantees last-buyer fills near graduation      |
| LP Reserve         | 300,000,000    | Seeded to the DEX at graduation                  |
| **Total**          | **~1.097B**    | Fixed — `mintRevoked = true` at pool init        |

### Bonding Curve

```
price = virtual_bnb / virtual_tokens
```

- **Initial virtual BNB**: 30 BNB
- **Initial virtual tokens**: ~1.073B (18 decimals)
- **Initial price**: ~0.00003 BNB per token
- **Formula**: tokens_out = vt · net_bnb / (vs + net_bnb)

### Graduation

When `realBNBReserves >= 85 BNB`, the pool is eligible. `migrate()` is **permissionless**:

1. **300M LP tokens** → DEX via adapter
2. **Remaining tokens**: 50% burned, 50% to migration vault (claimable by creator)
3. **All BNB** → DEX pool
4. `graduated = true`

### Fee Structure

Every trade: **1% fee** on BNB volume.

| Share  | Amount | Destination                                    |
| ------ | ------ | ---------------------------------------------- |
| 0.75%  | Platform | `0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA` |
| 0.25%  | Creator  | `accumulatedCreatorFees` (claimable)            |

### Migration Targets on BSC

| Target | Type | Factory | Volume |
|--------|------|---------|--------|
| **PancakeSwap V2** | Constant Product AMM | `0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73` | High |
| **PancakeSwap V3** | Concentrated Liquidity | `0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865` | Highest |
| **Thena V3** | Algebra CL + ve(3,3) | `0x306F06C147f064A010530292A0aAE5d8D230bC3d` | High |
| **Biswap V3** | Algebra CL | `0x7C3d53606f9c03E262f1B7Ea2C2149B6d65D8b11` | Medium |

---

## BSC Addresses

```solidity
// Pulse Platform
address constant PLATFORM_WALLET = 0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA;

// Wrapped BNB
address constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;

// PancakeSwap V2
address constant PCS_V2_FACTORY = 0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73;
address constant PCS_V2_ROUTER  = 0x10ED43C718714eb63d5aA57B78B54704E256024E;

// PancakeSwap V3
address constant PCS_V3_FACTORY     = 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865;
address constant PCS_V3_SWAP_ROUTER = 0x13f4EA83D0bd40E75C8222255bc855a974568Dd4;
address constant PCS_V3_NFT_MGR     = 0x46A15B0b27311cedF172AB29E4f4766fbE7F4364;

// Thena V3
address constant THENA_V3_FACTORY = 0x306F06C147f064A010530292A0aAE5d8D230bC3d;
address constant THENA_V3_ROUTER  = 0xd4ae6eCA985340Dd434D38F470aCCcE4DC48866D;

// Biswap V3
address constant BISWAP_V3_FACTORY = 0x7C3d53606f9c03E262f1B7Ea2C2149B6d65D8b11;
address constant BISWAP_V3_ROUTER  = 0x237c0c520c8A6E2c8EE0dFdA41eFD7693C4c7f20;
```

---

## Solidity API

### PulseFactory

| Function | Description |
|----------|-------------|
| `createPool(name, symbol, target, salt)` | Deploy PulsePool via CREATE2 |
| `predictPoolAddress(...)` | Deterministic address prediction (PDA equivalent) |
| `setDexAdapter(target, adapter)` | Register a DEX adapter |

### PulsePool

| Function | Description |
|----------|-------------|
| `initializePool()` | Mint tokens, revoke mint, accept initial BNB |
| `buy(minTokensOut)` | Buy tokens — send BNB with call |
| `sell(tokenAmount, minBNBOut)` | Sell tokens back to curve |
| `migrate()` | **Permissionless.** Graduate to chosen DEX |
| `claimFees()` | Creator claims 0.25% accumulated fees |
| `transferAuthority(newAuth)` | Transfer fee-claiming wallet |
| `stake(amount)` / `unstake(amount)` | Staking for rewards |
| `claimStakerRewards()` | Claim staking rewards |
| `claimMigrationVault()` | Claim half of remaining tokens post-graduation |

### DEX Adapters

| Adapter | Contract |
|---------|----------|
| PancakeSwap V2 | `PancakeSwapV2Adapter.sol` |
| PancakeSwap V3 | `PancakeSwapV3Adapter.sol` |
| Thena V3 | `ThenaV3Adapter.sol` |
| Biswap V3 | `BiswapV3Adapter.sol` |

---

## Events

Every state change emits typed Solidity events:

- `PoolInitialized` — token created, ready for trading
- `Buy` / `Sell` — trade with fee breakdown, reserve splits
- `GraduationReady` — 85 BNB threshold reached
- `Migrated` — graduation complete (burn, vault, DEX pool)
- `FeesClaimed` / `AuthorityTransferred`
- `Staked` / `Unstaked` / `StakerRewardsClaimed`
- `MigrationVaultClaimed`

---

## Deploy

```bash
# BSC Testnet (Chain 97)
export RPC_URL="https://data-seed-prebsc-1-s1.binance.org:8545"

forge create --rpc-url $RPC_URL --private-key $KEY \
  contracts/PulseGlobalConfig.sol:PulseGlobalConfig \
  --constructor-args 85000000000000000000

# 85 BNB threshold (85 * 10^18 wei)

forge create --rpc-url $RPC_URL --private-key $KEY \
  contracts/PulseFactory.sol:PulseFactory \
  --constructor-args <GLOBAL_CONFIG_ADDRESS>

# Deploy DEX adapters
forge create --rpc-url $RPC_URL --private-key $KEY \
  bsc/contracts/dex-adapters/PancakeSwapV2Adapter.sol:PancakeSwapV2Adapter

forge create --rpc-url $RPC_URL --private-key $KEY \
  bsc/contracts/dex-adapters/PancakeSwapV3Adapter.sol:PancakeSwapV3Adapter

# Register adapters with factory
cast send $FACTORY "setDexAdapter(uint8,address)" 0 $PCSV2_ADAPTER --private-key $KEY
cast send $FACTORY "setDexAdapter(uint8,address)" 1 $PCSV3_ADAPTER --private-key $KEY
```

---

## BSC Mainnet (Chain 56)

```bash
export RPC_URL="https://bsc-dataseed.binance.org"
# Same deployment flow as testnet, but with real BNB
```

---

## Testing

```bash
# Unit tests
forge test --match-path test/bsc/*

# Integration test (PancakeSwap V2 migration)
forge test --match-contract PancakeSwapV2MigrationTest

# Gas profiling
forge test --gas-report --match-path test/bsc/*
```

---

## License

TBD
