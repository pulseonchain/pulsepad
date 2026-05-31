
A cross-chain, agentic bonding curve protocol. The **Pulse EVM** implementation brings the same constant-product bonding curve from Solana to the EVM ecosystem — each token graduates to the most liquid DEX on its chain, with optional on-chain agents managing buybacks, fee distribution, and vault releases.

Each chain gets one `PulseFactory` (immutable, no proxies). Pool creators choose a **graduation tier** (Fast/Standard/Stable), set their **fee rate**, enable **anti-snipe** protection, optionally assign an **AI agent** to manage post-graduation operations, and configure **partial migration** for permanent buyback funds.

---

Website: https://pulse.jelly-os.xyz/

CA (Solana): BJP94VkAVHdHZZ9pBJPCWTHha3EgJjyZkeFordhXpump


---

## Supported Chains

| Chain | Native | Tiers (Fast/Standard/Stable) | Primary DEX | Timelock | Breaker | Status |
|-------|--------|------------------------------|-------------|----------|---------|--------|
| **BNB Smart Chain** | BNB | 15 / 35 / 50 BNB | PancakeSwap V3 | 24h | 500 BNB/block | 🟡 Localnet |
| **Ethereum Mainnet** | ETH | 9 / 16 / 30 ETH | Uniswap V3 | 48h | 100 ETH/block | 🟡 Localnet |
| **Base** | ETH | 9 / 16 / 30 ETH | Aerodrome | 6h | 2000 ETH/block | 🟡 Localnet |

> ⚠️ **Pulse EVM is still localnet.** Contracts written, documented, tested with Foundry — NOT YET DEPLOYED. 

## Agentic Protocol Features

### 🎓 Three Graduation Tiers

| Tier | Solana | BSC | ETH / Base | Purpose |
|------|--------|-----|------------|---------|
| **Fast** | 80 SOL ($6.5K) | 15 BNB ($10.7K) | 9 ETH ($18K) | Quickest path to DEX |
| **Standard** | 150 SOL ($12.2K) | 35 BNB ($24.9K) | 16 ETH ($32K) | Balanced price discovery |
| **Stable** | 240 SOL ($19.6K) | 50 BNB ($35.6K) | 30 ETH ($60K) | Deep liquidity, most stable |

### 🛡️ Anti-Snipe Protection

First **3 minutes** after pool initialization: bonding curve virtual reserves are **3x higher** — making the effective price 3x more expensive. Snipers and bundlers get wrecked. Normal buyers simply buy regularly.

Configurable per pool at creation.

### 💰 Configurable Fees (1-5%)

Creators set their fee rate at pool creation (1% to 5% in 0.5% increments). 

### 🤖 On-Chain Agents

Each pool can optionally deploy an **on-chain agent wallet** — a contract-controlled PDA that:

- **Public name**: `"Agent <TICKER>"` — stored on-chain, publicly searchable
- **Receives fees** instead of the creator (set at prebond config)
- **Claims every 3 hours** — rate-limited to prevent spam
- **Executes buybacks** — uses partial migration buyback fund
- **Autonomous operations** — all verified through `api.jellychain.fun`

Agents are **immutable** after creation, just like the token.

### 🔄 Partial Migration (Buyback Fund)

Instead of migrating 100% of liquidity to the DEX, the creator can keep **10%, 20%, or 30%** in a permanent buyback fund:

- Remaining SOL/BNB/ETH stays in the bonding curve
- Agent can execute `agentBuyback()` to buy tokens from the curve
- Tokens can be burned (deflationary) or routed to treasury
- Creates a **permanent price floor** — buy pressure forever

### 🔒 Vault Claim Cap

Post-graduation migration vault claims are capped at **500K tokens per 24 hours** — for both creators and agents. This prevents dumps and ensures gradual distribution.

---

## Architecture

### Contract Hierarchy

```
PulseGlobalConfig (singleton per chain)
├── Three-tier thresholds (Fast/Standard/Stable)
├── Platform wallet: 0xd479A4...
├── Timelock ownership (chain-specific delays)
└── Pause/unpause

PulseCircuitBreaker (singleton per chain)
├── Volume cap per block
├── Tx frequency limit per address
└── Graduation cooldown

PulseFactory (singleton, CREATE2)
├── createPool(name, symbol, target, salt, tier, feeBps, feesToAgent, agentName, partialPct)
└── predictPoolAddress(...)

PulsePool (one per token, CREATE2 immutable)
├── PulseToken (ERC-20, mint authority revoked)
├── PrebondConfig (immutable prebond parameters)
├── AgentWallet (optional, if feesToAgent)
├── VaultClaimTracker (500K/24h cap)
├── Bonding Curve (constant-product + anti-snipe)
├── Partial Migration / Buyback Fund
└── Staking + Rewards
```

### Token Supply (all chains identical)

| Bucket | Amount | Purpose |
|--------|--------|---------|
| Bonding Supply | 700,000,000 | Sold via curve |
| Reserve Supply | 97,052,391 | Last-buyer guarantee |
| LP Reserve | 300,000,000 | DEX seed at graduation |
| **Total** | **~1.097B** | Fixed — mint revoked |

---

## API Endpoints (api.jellychain.fun)

All agent actions and creator operations are verified through the Pulse API:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v1/token/verify` | POST | Verify token creation with creator signature |
| `/v1/agent/claim` | POST | Agent claims fees — requires ECDSA signature |
| `/v1/agent/buyback` | POST | Agent executes buyback with burn/trade params |
| `/v1/agent/transfer` | POST | Agent transfers SOL/tokens to destination |
| `/v1/agent/status` | GET | Agent health, balance, next scheduled action |
| `/v1/agent/strategy` | PUT | Update agent strategy (price floor, burn rate) |
| `/v1/vault/claim` | POST | Creator/agent claims vault tokens (500K cap) |
| `/v1/token/price` | GET | Current bonding curve price + DEX price |
| `/v1/token/holders` | GET | Holder count, concentration, top holders |

---

## Per-Chain Quick Reference

### 🔶 BNB Smart Chain (Chain 56)

| Param | Value |
|-------|-------|
| Tiers | 15 / 35 / 50 BNB |
| Timelock | 24 hours |
| Breaker | 500 BNB/block, 10 tx/addr |
| DEX Adapters | PancakeSwap V2/V3, Thena V3, Biswap V3 |
| Gas | ~$0.15/trade |

### Ξ Ethereum Mainnet (Chain 1)

| Param | Value |
|-------|-------|
| Tiers | 9 / 16 / 30 ETH |
| Timelock | 48 hours |
| Breaker | 100 ETH/block, 5 tx/addr |
| DEX Adapters | Uniswap V2/V3, SushiSwap V2 |
| Gas | ~$5-50/trade — min trade 0.05 ETH recommended |

### 🔵 Base (Chain 8453)

| Param | Value |
|-------|-------|
| Tiers | 9 / 16 / 30 ETH |
| Timelock | 6 hours |
| Breaker | 2000 ETH/block, 50 tx/addr |
| DEX Adapters | Aerodrome, Alien Base |
| Gas | ~$0.0003/trade — micro-trades viable |

---

## Solidity API

### PulseFactory

| Function | Description |
|----------|-------------|
| `createPool(name, symbol, target, salt, tier, feeBps, feesToAgent, agentName, partialPct)` | Full CREATE2 deploy with prebond config |
| `predictPoolAddress(...)` | Deterministic address prediction |
| `setDexAdapter(target, adapter)` | Register a DEX adapter |

### PulsePool

| Function | Description |
|----------|-------------|
| `initializePool()` | Mint tokens, revoke mint, start anti-snipe window |
| `buy(minTokensOut)` | Buy tokens (3x price during anti-snipe) |
| `sell(tokenAmount, minNativeOut)` | Sell tokens back to curve |
| `migrate()` | Graduate to DEX (with optional partial migration) |
| `agentClaim()` | Agent claims accumulated fees (3h cooldown) |
| `agentBuyback(solToSpend, burnPct)` | Agent executes buyback from partial migration fund |
| `claimMigrationVault()` | Claim vault tokens (500K/24h cap) |
| `claimFees()` / `transferAuthority()` | Creator fee management |
| `stake()` / `unstake()` / `claimStakerRewards()` | Staking |

### Modules

| Contract | Purpose |
|----------|---------|
| `PrebondConfig.sol` | Immutable prebond config per pool |
| `AgentWallet.sol` | Agent fee accumulation + claim |
| `VaultClaimTracker.sol` | 500K token / 24h cap enforcement |
| `PulseTimelockController.sol` | Admin timelock with chain-specific delays |
| `PulseCircuitBreaker.sol` | Volume/tx/frequency limits per chain |

---

## Events

- `PoolInitialized` / `Buy` / `Sell`
- `GraduationReady` / `Migrated`
- `BuybackActivated` / `BuybackExecuted`
- `AgentClaimed` / `AgentFunded`
- `FeesClaimed` / `AuthorityTransferred`
- `Staked` / `Unstaked` / `StakerRewardsClaimed`
- `VaultClaimed` / `PrebondConfigured`

---

## Development

```bash
# Foundry
forge build
forge test --match-path bsc/test/
forge test --match-path eth/test/
forge test --match-path base/test/

# Deploy
forge script bsc/script/DeployBSC.s.sol --rpc-url $BSC_RPC --broadcast
forge script eth/script/DeployETH.s.sol --rpc-url $ETH_RPC --broadcast
forge script base/script/DeployBase.s.sol --rpc-url $BASE_RPC --broadcast
```

---

## Security

| Measure | Implementation |
|---------|---------------|
| Immutable contracts | No proxies, no upgrade path |
| ReentrancyGuard | All state-changing functions |
| Solidity 0.8.24 | Built-in overflow protection |
| Timelock | Per-chain admin delays (6h-48h) |
| Circuit breaker | Volume/tx/graduation limits |
| Platform wallet | Hardcoded, same on all chains |
| CREATE2 | Predictable addresses, no collision |

---

## License

TBD
