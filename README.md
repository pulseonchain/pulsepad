<p align="center">
  <a href="https://github.com/jelly-chain">
    <img src="https://pbs.twimg.com/community_banner_img/2059546354985111552/RE1wKh4o?format=jpg&name=medium" alt="pulse">
  </p>


<p align="center">
  <a href="https://pulse.jelly-os.xyz/">Website</a> •
  <a href="https://x.com/i/communities/2033089758583275697">X</a> 
</p>

A cross-chain bonding curve protocol built on Solana (Anchor framework) that lets any creator launch a token, run it through a constant-product bonding curve, and graduate it to **multiple** DEX launchpads — not just Pump.fun.

Each chain gets one bonding curve, bonded tokens can migrate to many launchpads, and the entire protocol is designed to evolve into a DAO governed by top token holders.

---
Website: https://pulse.jelly-os.xyz/

CA: BJP94VkAVHdHZZ9pBJPCWTHha3EgJjyZkeFordhXpump

## The Big Idea

```
            ┌─────────────────────────────────────────┐
            │           CTO BONDING CURVE             │
            │    (one bonding curve per chain)        │
            │                                         │
            │  Creator launches ──►  CP curve trades  │
            │                        85 SOL raised     │
            │                            │             │
            │                 ┌──────────┴──────────┐  │
            │                 ▼         ▼           ▼  │
            │           Pump.fun   Raydium    Meteora │
            │           PumpSwap    CPMM      DAMM v1  │
            │                                  DLMM    │
            └─────────────────────────────────────────┘
                                  │
                                  ▼
                     ┌────────────────────────┐
                     │  COMMUNITY BOARD (DAO) │
                     │  Top holders vote on   │
                     │  protocol future       │
                     └────────────────────────┘
```

1. **One bonding curve per chain** — Every supported blockchain gets a single bonding curve contract. Tokens are bought/sold against a constant-product AMM.
2. **Graduate to any launchpad** — When the bonding curve hits its SOL threshold, it migrates liquidity to the creator's DEX of choice: Pump.fun, PumpSwap, Raydium CPMM, Meteora DAMM v1, or Meteora DLMM. More launchpads can be added.
3. **Cross-chain migration** — A token bonded on Chain A can be routed to a launchpad on Chain B via the migration config.
4. **DAO governance** — Top token holders form a Community Board that decides protocol upgrades, fee structures, and supported launchpads.
5. **Fees fund development** — All protocol fees (1% per trade) fund continued development. 0.75% goes to the platform treasury, 0.25% to the token creator.

---

## Current Status: Localnet Testing

The contract is **actively being tested on localnet**. Program ID: `5NLh9rQPR4EAZZpZfAJ3ujszffKjMJJCEGXxCBf4CRea`

What works today:
- Full token creation (4-step: `create_token` → `create_token_accounts` → `create_staker_vault` → `initialize_pool`)
- Buy and sell against the constant-product bonding curve
- Auto-detection of graduation threshold (85 SOL)
- Permissionless migration via pre-configured `MigrationConfig` PDA
- Creator fee claiming and authority transfer
- LP fee claiming (Raydium / PumpSwap / Meteora)
- Token staking and reward distribution (Meteora DAMM v1 / DLMM)
- Migration vault claiming by creator

Known limitations / TODO:
- Migration CPI to DEX programs (Raydium, Meteora, PumpSwap) is scaffolded but not yet connected — DEX accounts are passed as placeholders during localnet testing
- Auto-migration in `buy()` emits an event but does not yet CPI into `migrate()` in the same instruction; a backend crank (or frontend) calls `migrate()` separately once the event fires
- Community Board DAO module is planned but not yet implemented

---

## Architecture

### Token Supply Model

| Bucket             | Amount         | Purpose                                        |
| ------------------ | -------------- | ------------------------------------------------ |
| Bonding Supply     | 700,000,000    | Sold via the bonding curve                       |
| Reserve Supply     | 97,052,391     | Guarantees last-buyer fills near graduation      |
| LP Reserve         | 300,000,000    | Seeded to the DEX at graduation                  |
| **Total**          | **~1.097B**    | Fixed — mint authority is revoked at pool init   |

### Bonding Curve Mechanics

The curve uses a **constant-product (x · k = y) formula** with virtual reserves, mirroring Pump.fun's pricing model:

```
price = virtual_sol / virtual_tokens
```

- **Initial virtual SOL**: 30 SOL (30,000,000,000 lamports)
- **Initial virtual tokens**: ~1.073B (1,073,000,000,000,000 base-units)
- **Initial price**: ~0.00003 SOL per token

On each buy:
- Virtual SOL increases, virtual tokens decrease → price goes up
- Tokens are drawn first from the 700M bonding pool
- The 97M reserve is **only** tapped when a single buy exceeds remaining bonding supply

On each sell:
- Virtual SOL decreases, virtual tokens increase → price goes down
- Tokens sold back return to the bonding pool

The curve has **no price cap** — it asymptotically approaches the virtual reserve boundary. The 97M reserve ensures the last buyer always gets their full fill even when bonding supply is nearly exhausted.

### Graduation

When `real_sol_reserves >= 85 SOL`, the pool is eligible for graduation. The `migrate()` instruction is permissionless:

1. **300M LP tokens** are sent from the LP reserve to the DEX token account
2. **Remaining pool tokens** (bonding leftovers + reserve leftovers) are split 50/50:
   - Half → **burned** (permanently removed)
   - Half → **migration vault** (claimable by creator)
3. **All SOL** is sent to the DEX pool
4. Pool is marked `graduated = true`

### Fee Structure

Every trade is charged a **1% fee** on the SOL volume:

| Share  | Amount | Destination                                      |
| ------ | ------ | ------------------------------------------------ |
| 0.75%  | Platform | Immediately sent to platform treasury wallet     |
| 0.25%  | Creator  | Accumulated in a claimable PDA (`fee_recipient`)  |

Post-graduation LP fees follow the same split, plus a configurable share to stakers for Meteora targets.

### Migration Targets

The `MigrationTarget` enum supports five DEX destinations:

```rust
pub enum MigrationTarget {
    RaydiumCpmm,                   // Raydium CPMM pool
    MeteoraDammV1 { ... },         // Meteora Dynamic AMM v1
    MeteoraDlmm { ... },           // Meteora Dynamic Liquidity Market Maker
    PumpSwapBurn,                  // PumpSwap (LP tokens burned)
    PumpSwapHoldLp,                // PumpSwap (LP tokens held for fee claiming)
}
```

Meteora targets support configurable fee-sharing:
- `lp_share` — LP holders' share of Meteora protocol fees
- `staker_share` — Stakers' share (tokens deposited into staking contract)
- `holder_share` — Airdrop holders' share
- All three must sum to 100

The `MigrationConfig` PDA is created during token creation with pre-configured DEX parameters, making `migrate()` fully permissionless — no caller-supplied DEX accounts needed.

---

## Instruction Reference

### Admin Instructions

| Instruction        | Description                                                  |
| ------------------ | ------------------------------------------------------------ |
| `initialize`       | One-time setup. Creates the GlobalConfig PDA. Must be called by the platform wallet before any tokens can be launched. |

### Token Creation (4 Steps)

| Step | Instruction          | Description                                                  |
| ---- | -------------------- | ------------------------------------------------------------ |
| 1    | `create_token`       | Creates the token mint, Metaplex metadata, and PoolState PDA. Sets migration target. |
| 1b   | `create_token_accounts` | Creates the pool token account and LP reserve token account.  |
| 1c   | `create_staker_vault`| Creates the staker vault, fee vault, fee recipient, migration vault, and MigrationConfig PDA. Pre-configures DEX parameters for permissionless migration. |
| 2    | `initialize_pool`    | Mints 700M + 97M + 300M tokens to their respective accounts, revokes mint and freeze authority (supply forever fixed), and deposits initial SOL (>= 0.02 SOL). |

### Trading

| Instruction | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| `buy`       | Buy tokens from the bonding curve. Deducts 1% fee, then executes the constant-product swap. Auto-detects graduation threshold. |
| `sell`      | Sell tokens back to the bonding curve. Deducts 1% fee from the gross SOL output. |

### Graduation

| Instruction | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| `migrate`   | Permissionless. Sends 300M LP tokens + all SOL to the DEX. Burns half of remaining tokens, sends the other half to migration vault. |

### Creator Fee Management

| Instruction          | Description                                                  |
| -------------------- | ------------------------------------------------------------ |
| `claim_fees`         | Creator claims accumulated 0.25% trading fees. Leaves 0.005 SOL as gas reserve. |
| `transfer_authority` | Permanently transfer fee-claiming authority to a new wallet. Old wallet loses all access. |

### Post-Graduation LP Fee Claiming

| Instruction        | Description                                                  |
| ------------------ | ------------------------------------------------------------ |
| `claim_lp_fees`    | Permissionless crank. Claims LP fees from Raydium or PumpSwap, splits 0.75%/0.25%, credits staker vault for Meteora targets. |

### Staking (Meteora)

| Instruction            | Description                                                  |
| ---------------------- | ------------------------------------------------------------ |
| `stake`                | Stake tokens to earn a share of post-graduation creator fees. |
| `unstake`              | Unstake tokens. Snapshots pending rewards before reducing stake. |
| `claim_staker_rewards` | Claim accumulated SOL rewards proportionally to stake amount. |

### Migration Vault

| Instruction             | Description                                                  |
| ----------------------- | ------------------------------------------------------------ |
| `claim_migration_vault` | Creator claims tokens held in the migration vault (half of remaining tokens at migration time). |

---

## Account Architecture

### GlobalConfig (singleton)
```
Seeds: ["global_config"]
Fields: authority, platform_wallet, fee settings, graduation threshold, paused flag
```

### PoolState (one per token)
```
Seeds: ["pool_state", mint]
Fields: mint, creator, current_authority, migration_target,
        virtual_sol_reserves, virtual_token_reserves,
        real_sol_reserves, real_token_reserves, reserve_tokens_remaining,
        graduated, dex_pool, bumps for all PDAs
```

### MigrationConfig (one per token)
```
Seeds: ["migration_config", mint]
Fields: mint, migration_target, dex_program_id, fee_share_config,
        dex_pool, dex_token_account, fee_recipient
All pre-configured at creation so migrate() is permissionless.
```

### Fee Vault (one per token)
```
Seeds: ["fee_vault", mint]
Holds: Bonding curve SOL + fees in transit
Signs: As PDA for transfers to platform and fee_recipient
```

### Fee Recipient (one per token)
```
Seeds: ["fee_recipient", mint]
Holds: Creator's accumulated 0.25% share
Creator calls claim_fees() to withdraw (minus 0.005 SOL gas reserve)
```

### StakerVault (one per token)
```
Seeds: ["staker_vault", mint]
Holds: Total staked amount, accumulated_reward_per_token (u128 precision), SOL lamports for rewards
```

### StakeAccount (one per user per token)
```
Seeds: ["stake", mint, user_pubkey]
Tracks: amount_staked, staked_at, last_claimed, reward_debt
```

### Migration Vault (one per token)
```
Seeds: ["migration_vault", mint]
Holds: Half of remaining pool tokens at graduation
Only current_authority can claim via claim_migration_vault()
```

---

## PDA Seed Reference

| Seed                 | Format                              |
| -------------------- | ----------------------------------- |
| `global_config`      | `b"global_config"`                  |
| `pool_state`         | `b"pool_state" + mint`              |
| `fee_vault`          | `b"fee_vault" + mint`               |
| `fee_recipient`      | `b"fee_recipient" + mint`           |
| `pool_tokens`        | `b"pool_tokens" + mint`             |
| `lp_reserve`         | `b"lp_reserve" + mint`              |
| `stake`              | `b"stake" + mint + user_pubkey`     |
| `lp_token_vault`     | `b"lp_token_vault" + mint`          |
| `migration_vault`    | `b"migration_vault" + mint`         |
| `migration_config`   | `b"migration_config" + mint`        |
| `staker_vault`       | `b"staker_vault" + mint`            |

---

## Events

Every state-changing instruction emits a typed Anchor event for off-chain indexing:

- `TokenCreatedEvent` — token minted
- `BuyEvent` — trade executed (includes bonding vs reserve split)
- `SellEvent` — sell executed
- `MigrateEvent` — graduation completed (includes burn and vault amounts)
- `FeeClaimedEvent` / `AuthorityTransferredEvent`
- `LpFeeClaimedEvent` — post-graduation LP fee distribution
- `StakeEvent` / `UnstakeEvent` / `StakeRewardClaimedEvent`
- `GraduationReadyEvent` — emitted when threshold is reached

---

## Error Codes

| Code                        | Meaning                                              |
| --------------------------- | ---------------------------------------------------- |
| `Paused`                    | Program is paused by admin                           |
| `Unauthorized`              | Signer is not the current authority                  |
| `AlreadyGraduated`          | Token has already migrated to a DEX                  |
| `NotReadyToGraduate`        | SOL threshold not yet reached                        |
| `ZeroSolAmount` / `ZeroTokenAmount` | Invalid trade amount                         |
| `InsufficientPoolTokens`    | Buy exceeds available bonding + reserve supply       |
| `InsufficientPoolSol`       | Sell exceeds pool SOL balance                        |
| `SlippageExceeded`          | Output below user's minimum                          |
| `BelowMinReserve`           | Creator fee balance below 0.005 SOL gas reserve      |
| `InvalidMigrationConfig`    | Migration target or accounts are misconfigured       |
| `InvalidShareSum`           | Meteora shares don't sum to 100                      |
| `MathOverflow`              | Arithmetic overflow in curve calculation             |
| `InsufficientStake`         | Unstake amount exceeds staked balance                |

---

## Roadmap

### Phase 1: Localnet (Current)
- [x] Core bonding curve (constant-product with virtual reserves)
- [x] 4-step token creation
- [x] Buy / sell with fee splitting
- [x] Pre-configured MigrationConfig for permissionless graduation
- [x] Creator fee claiming and authority transfer
- [x] LP fee claiming scaffold (Raydium, PumpSwap, Meteora)
- [x] Staking + reward distribution for Meteora targets
- [x] Migration vault for post-graduation token claims

### Phase 2: Devnet Testing
- [ ] Connect real DEX CPI calls (Raydium, Meteora, PumpSwap)
- [ ] Auto-migration within the `buy()` instruction
- [ ] Frontend DApp for token creation and trading
- [ ] Backend crank service for LP fee distribution

### Phase 3: DAO Governance
- [ ] Community Board module
- [ ] Top holder voting for protocol upgrades
- [ ] Fee parameter governance
- [ ] Supported launchpad voting (add new DEX targets)

### Phase 4: Multi-Chain
- [ ] Deploy bonding curve contracts on each supported chain
- [ ] Standardized migration configs per chain
- [ ] Cross-chain messaging for token graduation

---

## Building & Testing

```bash
# Build
anchor build

# Run local test validator
solana-test-validator --ledger ./test-ledger

# Deploy to localnet
anchor deploy

# Run tests
anchor test
```

---

## Fee Reinvestment Model

All protocol fees (0.75% of every trade + post-graduation LP fees) are directed to the platform treasury and used exclusively for:

1. **Protocol development** — continuing to build out the bonding curve, migration, and DAO modules
2. **Security audits** — funding professional audits before mainnet launch
3. **Ecosystem growth** — supporting the Community Board and governance infrastructure

Fees are not diverted to any private entity — they exist to build and sustain the protocol.

---

## The Community Board & DAO Vision

Once launched, the protocol will transition governance to a **Community Board** of top token holders from all bonded tokens. This board will:

- Vote on protocol upgrades (new features, fee adjustments)
- Propose and ratify new DEX migration targets
- Direct fee allocation (development grants, liquidity incentives)
- Shape the long-term roadmap

The DAO is not live yet. It's the destination — fees today are building toward it.

---

## License

TBD