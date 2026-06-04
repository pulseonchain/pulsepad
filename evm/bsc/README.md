

Pulse on 
BNB Smart Chain  — a constant-product bonding curve protocol built in Solidity that lets any creator launch a token, run it through a bonding curve, and graduate it to PancakeSwap, Thena, or Biswap — not just one DEX.

Each chain gets one `PulseFactory`, bonded tokens can migrate to four DEX launchpads on BSC, and the entire protocol feeds into a cross-chain DAO governed by top token holders.

BNB

0x203a2b2a377cff93f526dc58ca879a6a1bc5ffff


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
            │              15, 35 or 50 BNB raised    │
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
2. **Graduate to any BSC DEX** — When the bonding curve hits 50 BNB, it migrates liquidity to PancakeSwap V2, PancakeSwap V3, Thena V3, or Biswap V3.
3. **BSC-native experience** — 3-second blocks, ~$0.15 per trade in gas, massive retail user base in Asia.
4. **DAO governance** — Top token holders across ALL chains form a Community Board.
5. **Fees fund development** — 1% per trade: 0.75% to `0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA`, 0.25% to creator.

---

## Graduation Tiers (BSC)

| Tier | Threshold | USD (@ $711 BNB) | Use Case |
|------|-----------|------------------|----------|
| **Fast** | 15 BNB | $10,670 | Quick launch, viral memes |
| **Standard** | 35 BNB | $24,897 | Balanced price discovery |
| **Stable** | 50 BNB | $35,561 | Deep liquidity, serious projects |

## Anti-Snipe

First 3 minutes & Bonding Guard: bonding curve virtual BNB is **3x**. Snipers pay 3x. Normal buyers are exempt through organic trading. this reduces Rugs and Bundling. 

## Configurable Fees

Pool creators set fees at 1%–5%. 
## On-Chain Agents

Optional at creation. Name format: `"Agent <TICKER>"`. Fees route to agent instead of creator. Agent claims every 3 hours. Buyback execution via `agentBuyback()`.

## Partial Migration

Keep 10%/20%/30% of BNB in a permanent buyback fund. Agent executes buys when price dips. Tokens burned or routed to treasury.

## Vault Cap

500K tokens per 24 hours — creators and agents both capped. 

## Current Status: Contracts Complete

Full plan and audit: [PLAN.md](./PLAN.md)

```
PulseGlobalConfig (15/35/50 BNB tiers, 24h timelock)
PulseFactory (CREATE2 immutable pools)
PulsePool
├── PulseToken (ERC-20, mint revoked)
├── PrebondConfig (tier, fee, agent, anti-snipe)
├── AgentWallet (optional, 3h claim cooldown)
├── VaultClaimTracker (500K/24h cap)
├── Bonding Curve + Partial Migration
└── Staking
```

---
## Defense-in-Depth

| Layer | Value |
|-------|-------|
| Timelock | 24 hours on admin |
| Circuit Breaker | 500 BNB/block, 10 tx/addr, 30min grad cooldown |
| Immutable | No proxies, no upgrade path |
| ReentrancyGuard | All state-changing functions |

## API (api.jellychain.fun)

`agent/claim` | `agent/buyback` | `agent/transfer` | `vault/claim` | `token/verify` | `agent/status`

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

When `realBNBReserves >= 50 BNB`, the pool is eligible. `migrate()` is **permissionless**:

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
- `GraduationReady` — 50 BNB threshold reached
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

# 50 BNB threshold (50 * 10^18 wei)

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
