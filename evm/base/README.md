Pulse on **Base** — a constant-product bonding curve protocol built in Solidity that lets any creator launch a token, run it through a bonding curve, and graduate it to **Aerodrome or Alien Base** — the dominant DEXes on the fastest-growing Ethereum L2.

Base is the **best EVM chain for bonding curves** — 2-second blocks, near-zero gas costs, and a massive Coinbase retail funnel. This is where Pulse will see the most volume.

---

Website: https://pulse.jelly-os.xyz/


---

## Base Edition

```
            ┌─────────────────────────────────────────┐
            │           PULSE ON BASE                 │
            │    (one PulseFactory on Base L2)        │
            │                                         │
            │  Creator launches ──►  CP curve trades  │
            │                        25 ETH raised    │
            │                            │             │
            │                 ┌──────────┴──────────┐  │
            │                 ▼                     ▼  │
            │             Aerodrome           Alien    │
            │           (ve(3,3) CL AMM)       Base    │
            │                              (Algebra CL)│
            └─────────────────────────────────────────┘
                                  │
                                  ▼
                     ┌────────────────────────┐
                     │  COMMUNITY BOARD (DAO) │
                     │  Cross-chain top       │
                     │  holders govern Pulse  │
                     └────────────────────────┘
```

1. **One bonding curve per token** — Each token gets its own `PulsePool` deployed via CREATE2 on Base.
2. **Graduate to Aerodrome or Alien Base** — When the bonding curve hits 25 ETH, it migrates liquidity to Aerodrome (king of Base) or Alien Base.
3. **Solana-like economics on EVM** — 2s blocks, $0.0003 per trade in gas. Users can trade in micro amounts without losing to gas.
4. **Coinbase ecosystem** — 100M+ users, fiat onramps, massive retail potential.
5. **Fees fund development** — 1% per trade: 0.75% to `0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA`, 0.25% to creator.

> ✅ **Why Base First:** Gas is 1,000x cheaper than ETH mainnet and 500x cheaper than BSC. It's the only EVM chain where bonding curve economics match Solana's cost structure.

---

## Current Status: Contracts Complete

Full plan and audit: [PLAN.md](./PLAN.md)

- `PulseGlobalConfig` — singleton config with 25 ETH graduation threshold
- `PulseFactory` — CREATE2 factory on Base (Chain 8453)
- `PulsePool` — full bonding curve + ERC-20 + staking + migration
- `PulseToken` — ERC-20 with mint authority held by pool
- 2 DEX adapters for Base

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

### Migration Targets on Base

| Target | Type | Factory | Volume |
|--------|------|---------|--------|
| **Aerodrome** | ve(3,3) Volatile + CL pools | `0x420DD381b31aEf6683db6B902084cB0FFECe40Da` | Highest |
| **Alien Base** | Algebra CL AMM | `0x3B9726B169D0FEF2FBF740D525a226fA283ef3F7` | Growing |

---

## Base Addresses

```solidity
// Pulse Platform
address constant PLATFORM_WALLET = 0xd479A4BC8993D3b76Ff52C7C0a01e62784842AfA;

// Wrapped Ether (OP Stack predeploy)
address constant WETH = 0x4200000000000000000000000000000000000006;

// Aerodrome (PRIMARY DEX)
address constant AERODROME_FACTORY = 0x420DD381b31aEf6683db6B902084cB0FFECe40Da;
address constant AERODROME_ROUTER  = 0xcF77a3Ba9A5CA399B7c97c74d54e5b1Beb874E43;
address constant VE_AERO           = 0xeBf418Fe2512e7E6bd9b87a8F0f294aCDC67e6B4;

// Alien Base
address constant ALIEN_BASE_FACTORY = 0x3B9726B169D0FEF2FBF740D525a226fA283ef3F7;
address constant ALIEN_BASE_ROUTER  = 0x8c1A3cF8f83074169FE5D7aD50B978e1cD6b37c7;
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
| `migrate()` | **Permissionless.** Graduate to Aerodrome or Alien Base |
| `claimFees()` | Creator claims 0.25% accumulated fees |
| `transferAuthority(newAuth)` | Transfer fee-claiming wallet |
| `stake(amount)` / `unstake(amount)` | Staking for rewards |
| `claimStakerRewards()` | Claim staking rewards |
| `claimMigrationVault()` | Claim half of remaining tokens post-graduation |

### DEX Adapters

| Adapter | Contract |
|---------|----------|
| Aerodrome | `AerodromeAdapter.sol` |
| Alien Base | `AlienBaseAdapter.sol` |

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

## Gas Comparison (Why Base Wins)

| Operation | Base (0.001 gwei) | BSC (3 gwei) | ETH (30 gwei) |
|-----------|-------------------|--------------|---------------|
| `createPool()` | ~$0.001 | ~$0.50 | ~$15.00 |
| `buy()` | ~$0.0003 | ~$0.15 | ~$5.00 |
| `sell()` | ~$0.0002 | ~$0.12 | ~$4.00 |
| `migrate()` | ~$0.002 | ~$0.80 | ~$25.00 |

Base gas costs are **1,000x cheaper** than ETH mainnet. This is the only EVM chain where you can trade $1 worth of tokens without losing money to gas.

---

## Deploy

```bash
# Base Sepolia (Chain 84532)
export RPC_URL="https://sepolia.base.org"

forge create --rpc-url $RPC_URL --private-key $KEY \
  contracts/PulseGlobalConfig.sol:PulseGlobalConfig \
  --constructor-args 25000000000000000000

# 25 ETH threshold

forge create --rpc-url $RPC_URL --private-key $KEY \
  contracts/PulseFactory.sol:PulseFactory \
  --constructor-args <GLOBAL_CONFIG_ADDRESS>

# Deploy Aerodrome adapter (primary target)
forge create --rpc-url $RPC_URL --private-key $KEY \
  base/contracts/dex-adapters/AerodromeAdapter.sol:AerodromeAdapter

# Register adapter
cast send $FACTORY "setDexAdapter(uint8,address)" 9 $AERO_ADAPTER --private-key $KEY
# MigrationTarget.AERODROME = 9
```

---

## Base Mainnet (Chain 8453)

```bash
export RPC_URL="https://mainnet.base.org"
# Same flow — deployment costs ~$0.01 on Base
```

---

## Testing

```bash
# Unit tests
forge test --match-path test/base/*

# Fork testing (against real Base mainnet state)
forge test --fork-url $BASE_RPC --match-contract AerodromeMigrationTest

# Fuzz testing (cheap gas means trade abuse testing is critical)
forge test --match-path test/base/fuzz/*
```

---

## License

TBD
