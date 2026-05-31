

Pulse on Ethereum Mainnet — a constant-product bonding curve protocol built in Solidity that lets any creator launch a token, run it through a bonding curve, and graduate it to Uniswap, SushiSwap, or Balancer — the most battle-tested DEX ecosystem in DeFi.

Each chain gets one `PulseFactory`, bonded tokens can migrate to four DEX launchpads on Ethereum, and the entire protocol feeds into a cross-chain DAO governed by top token holders.

---

Website: https://pulse.jelly-os.xyz/


---

## The Big Idea (Ethereum Edition)

```
            ┌─────────────────────────────────────────┐
            │           PULSE ON ETHEREUM             │
            │    (one PulseFactory on mainnet)        │
            │                                         │
            │  Creator launches ──►  CP curve trades  │
            │                        25 ETH raised    │
            │                            │             │
            │                 ┌──────────┼──────────┐  │
            │                 ▼          ▼          ▼  │
            │             Uniswap    Uniswap    Sushi  │
            │               V2         V3       Swap  │
            │                       Balancer V2       │
            └─────────────────────────────────────────┘
                                  │
                                  ▼
                     ┌────────────────────────┐
                     │  COMMUNITY BOARD (DAO) │
                     │  Cross-chain top       │
                     │  holders govern Pulse  │
                     └────────────────────────┘
```

1. **One bonding curve per token** — Each token gets its own `PulsePool` deployed via CREATE2 on Ethereum.
2. **Graduate to any ETH DEX** — When the bonding curve hits 25 ETH, it migrates liquidity to Uniswap V2, Uniswap V3, SushiSwap V2, or Balancer V2.
3. **Prestige and legitimacy** — Ethereum is where institutional money lives. Being on ETH mainnet is about credibility.
4. **DAO governance** — Top token holders across ALL chains form a Community Board.
5. **Fees fund development** — 1% per trade: 0.75% to `0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA`, 0.25% to creator.

> ⚠️ **Gas Warning:** Ethereum mainnet gas is expensive. Minimum trade size should be at least 0.05 ETH. Bonding curve economics assume traders are serious.

---

## Current Status: Contracts Complete

Full plan and audit: [PLAN.md](./PLAN.md)

- `PulseGlobalConfig` — singleton config with 25 ETH graduation threshold (reduced from 85)
- `PulseFactory` — CREATE2 factory on ETH (Chain 1)
- `PulsePool` — full bonding curve + ERC-20 + staking + migration
- `PulseToken` — ERC-20 with mint authority held by pool
- 3 DEX adapters for Ethereum

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
price = virtual_eth / virtual_tokens
```

- **Initial virtual ETH**: 30 ETH
- **Initial virtual tokens**: ~1.073B (18 decimals)
- **Initial price**: ~0.00003 ETH per token
- **Formula**: tokens_out = vt · net_eth / (vs + net_eth)



### Graduation

When `realETHReserves >= 25 ETH`, the pool is eligible. `migrate()` is **permissionless**:

1. **300M LP tokens** → DEX via adapter
2. **Remaining tokens**: 50% burned, 50% to migration vault
3. **All ETH** → DEX pool
4. `graduated = true`

### Fee Structure

| Share  | Amount | Destination                                    |
| ------ | ------ | ---------------------------------------------- |
| 0.75%  | Platform | `0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA` |
| 0.25%  | Creator  | `accumulatedCreatorFees` (claimable)            |

### Migration Targets on Ethereum

| Target | Type | Factory | Volume |
|--------|------|---------|--------|
| **Uniswap V3** | Concentrated Liquidity | `0x1F98431c8aD98523631AE4a59f267346ea31F984` | Highest |
| **Uniswap V2** | Constant Product AMM | `0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f` | High |
| **SushiSwap V2** | CPAMM (UniV2 fork) | `0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac` | Medium |
| **Balancer V2** | Weighted Pools | Vault: `0xBA12222222228d8Ba445958a75a0704d566BF2C8` | High |

---

## Ethereum Addresses

```solidity
// Pulse Platform
address constant PLATFORM_WALLET = 0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA;

// Wrapped Ether
address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

// Uniswap V2
address constant UNI_V2_FACTORY = 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f;
address constant UNI_V2_ROUTER  = 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D;

// Uniswap V3
address constant UNI_V3_FACTORY     = 0x1F98431c8aD98523631AE4a59f267346ea31F984;
address constant UNI_V3_ROUTER      = 0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45;
address constant UNI_V3_NFT_MGR     = 0xC36442b4a4522E871399CD717aBDD847Ab11FE88;

// SushiSwap V2
address constant SUSHI_V2_FACTORY = 0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac;
address constant SUSHI_V2_ROUTER  = 0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F;

// Balancer V2
address constant BALANCER_VAULT = 0xBA12222222228d8Ba445958a75a0704d566BF2C8;
```

---

## Solidity API

### PulseFactory

| Function | Description |
|----------|-------------|
| `createPool(name, symbol, target, salt)` | Deploy PulsePool via CREATE2 |
| `predictPoolAddress(...)` | Deterministic address prediction |
| `setDexAdapter(target, adapter)` | Register a DEX adapter |

### PulsePool

| Function | Description |
|----------|-------------|
| `initializePool()` | Mint tokens, revoke mint, accept initial ETH |
| `buy(minTokensOut)` | Buy tokens — send ETH with call |
| `sell(tokenAmount, minETHOut)` | Sell tokens back to curve |
| `migrate()` | **Permissionless.** Graduate to chosen DEX |
| `claimFees()` | Creator claims 0.25% accumulated fees |
| `transferAuthority(newAuth)` | Transfer fee-claiming wallet |
| `stake(amount)` / `unstake(amount)` | Staking for rewards |
| `claimStakerRewards()` | Claim staking rewards |
| `claimMigrationVault()` | Claim half of remaining tokens post-graduation |

### DEX Adapters

| Adapter | Contract |
|---------|----------|
| Uniswap V2 | `UniswapV2Adapter.sol` |
| Uniswap V3 | `UniswapV3Adapter.sol` |
| SushiSwap V2 | `SushiSwapV2Adapter.sol` |
| Balancer V2 | `BalancerV2Adapter.sol` |

---

## Events

Every state change emits typed Solidity events:

- `PoolInitialized` — token created, ready for trading
- `Buy` / `Sell` — trade with fee breakdown
- `GraduationReady` — 25 ETH threshold reached
- `Migrated` — graduation complete (burn, vault, DEX pool)
- `FeesClaimed` / `AuthorityTransferred`
- `Staked` / `Unstaked` / `StakerRewardsClaimed`
- `MigrationVaultClaimed`

---

## Gas Considerations

| Operation | Estimated Gas | ETH @ 30 gwei | USD @ $2K/ETH |
|-----------|---------------|---------------|---------------|
| `createPool()` | ~3,500,000 | 0.105 ETH | ~$210 |
| `buy()` | ~150,000 | 0.0045 ETH | ~$9 |
| `sell()` | ~120,000 | 0.0036 ETH | ~$7 |
| `migrate()` | ~1,200,000 | 0.036 ETH | ~$72 |
| `claimFees()` | ~60,000 | 0.0018 ETH | ~$3.60 |

**Recommendation:** Set `minTradeETH = 0.05 ether` in the config to keep gas:value ratio reasonable.

---

## Deploy

```bash
# Sepolia Testnet (Chain 11155111)
export RPC_URL="https://sepolia.infura.io/v3/YOUR_KEY"

forge create --rpc-url $RPC_URL --private-key $KEY \
  contracts/PulseGlobalConfig.sol:PulseGlobalConfig \
  --constructor-args 25000000000000000000

# 25 ETH threshold (25 * 10^18 wei)

forge create --rpc-url $RPC_URL --private-key $KEY \
  contracts/PulseFactory.sol:PulseFactory \
  --constructor-args <GLOBAL_CONFIG_ADDRESS>

# Deploy Uniswap V3 adapter (primary target)
forge create --rpc-url $RPC_URL --private-key $KEY \
  eth/contracts/dex-adapters/UniswapV3Adapter.sol:UniswapV3Adapter
```

---

## Ethereum Mainnet (Chain 1)

```bash
export RPC_URL="https://mainnet.infura.io/v3/YOUR_KEY"
# Same flow — costs real ETH for deployment (~$200 for factory, ~$15 for adapters)
```

---

## Testing

```bash
# Unit tests
forge test --match-path test/eth/*

# Fork testing (against real ETH mainnet state)
forge test --fork-url $ETH_RPC --match-contract UniswapV3MigrationTest

# Gas report
forge test --gas-report --match-path test/eth/*
```

---

## License

TBD
