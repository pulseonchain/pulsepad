// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {PulseToken} from "./PulseToken.sol";
import {PulseGlobalConfig} from "./PulseGlobalConfig.sol";
import {PulseCircuitBreaker} from "./modules/PulseCircuitBreaker.sol";
import {PrebondConfig, AgentWallet, VaultClaimTracker} from "./PrebondConfig.sol";
import {PulseConstants, PulseTier} from "./PulseConstants.sol";
import {BondingCurveMath} from "./lib/BondingCurveMath.sol";
import {IPulseDexAdapter} from "./interfaces/IPulseDexAdapter.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title PulsePool
/// @notice Bonding curve pool with ERC-20, agent routing, anti-snipe, partial migration.
contract PulsePool is ReentrancyGuard {
    using SafeERC20 for IERC20;

    uint256 public constant BONDING_SUPPLY       = 700_000_000 * 1e18;
    uint256 public constant RESERVE_SUPPLY       =  97_052_391 * 1e18;
    uint256 public constant LP_RESERVE_SUPPLY    = 300_000_000 * 1e18;
    uint256 public constant INITIAL_VIRTUAL_NATIVE = 30 ether;
    uint256 public constant INITIAL_VIRTUAL_TOKEN  = 1_073_000_000 * 1e18;
    uint256 public constant BPS_DENOMINATOR = 10_000;

    // ═══ IMMUTABLES ═══════════════════════════════════════════════════════════
    PulseGlobalConfig public immutable globalConfig;
    PulseCircuitBreaker public immutable circuitBreaker;
    PulseToken public immutable token;
    address public immutable mint;

    /// @notice Pre-bonding configuration (tier, agent, anti-snipe, fee, partial migration)
    PrebondConfig public immutable prebond;

    /// @notice Agent wallet (deployed only if feesToAgent == true)
    AgentWallet public immutable agent;

    /// @notice Vault claim tracker (500K token/24h cap)
    VaultClaimTracker public immutable vaultTracker;

    // ═══ MIGRATION TARGETS ════════════════════════════════════════════════════
    enum MigrationTarget {
        PANCAKESWAP_V2, PANCAKESWAP_V3, THENA_V3, BISWAP_V3,
        UNISWAP_V2, UNISWAP_V3, SUSHISWAP_V2, SUSHISWAP_V3, BALANCER_V2,
        AERODROME, ALIEN_BASE
    }

    // ═══ BONDING CURVE STATE ═════════════════════════════════════════════════
    uint256 public virtualNativeReserves = INITIAL_VIRTUAL_NATIVE;
    uint256 public virtualTokenReserves = INITIAL_VIRTUAL_TOKEN;
    uint256 public realNativeReserves;
    uint256 public realTokenReserves = BONDING_SUPPLY;
    uint256 public reserveTokensRemaining = RESERVE_SUPPLY;
    uint256 public lpReserveBalance = LP_RESERVE_SUPPLY;

    // ═══ POOL CONFIG ══════════════════════════════════════════════════════════
    address public creator;
    address public currentAuthority;
    MigrationTarget public migrationTarget;
    address public dexAdapter;
    address public dexPool;
    bool public graduated;
    bool public initialized;
    uint256 public createdAt;
    uint256 public poolInitAt;          // for anti-snipe window

    // ═══ NEW: Tier & Config ═══════════════════════════════════════════════════
    PulseTier public graduationTier;
    uint256 public poolFeeBps = 100;    // 1% default, config at creation

    // ═══ NEW: Partial Migration ═══════════════════════════════════════════════
    uint8 public partialMigrationPct;
    bool public buybackActive;
    uint256 public buybackSolReserves;
    uint256 public buybackTokenReserves;
    uint256 public buybackVirtualSol;
    uint256 public buybackVirtualToken;
    uint256 public lastBuybackAt;

    // ═══ FEES ═════════════════════════════════════════════════════════════════
    uint256 public accumulatedCreatorFees;

    // ═══ STAKING ══════════════════════════════════════════════════════════════
    uint256 public totalStaked;
    uint256 public accumulatedRewardPerToken;
    mapping(address => StakerInfo) public stakers;
    struct StakerInfo { uint256 amountStaked; uint256 rewardDebt; uint256 stakedAt; }

    // ═══ VAULT ════════════════════════════════════════════════════════════════
    uint256 public migrationVaultBalance;

    // ═══ EVENTS ═══════════════════════════════════════════════════════════════
    event PoolInitialized(address indexed creator, address indexed token, MigrationTarget target);
    event Buy(address indexed buyer, uint256 nativeAmount, uint256 tokensOut, uint256 fromBonding, uint256 fromReserve, uint256 platformFee, uint256 creatorFee, uint256 virtualNative, uint256 virtualToken, uint256 realNative, uint256 ts);
    event Sell(address indexed seller, uint256 tokenAmount, uint256 nativeOut, uint256 platformFee, uint256 creatorFee, uint256 virtualNative, uint256 virtualToken, uint256 realNative, uint256 ts);
    event GraduationReady(address indexed token, uint256 reserves);
    event Migrated(address indexed token, MigrationTarget target, uint256 nativeIn, uint256 tokensIn, uint256 burned, uint256 vault, address dexPool, uint256 ts);
    event BuybackActivated(address indexed token, uint256 keptSol, uint256 keptTokens, uint8 pct, uint256 ts);
    event BuybackExecuted(address indexed token, uint256 solSpent, uint256 tokensBurned, uint256 tokensTreasury, uint256 ts);
    event AgentClaimed(address indexed token, string agentName, uint256 amount, uint256 ts);
    event FeesClaimed(address indexed authority, uint256 amount);
    event AuthorityTransferred(address indexed old, address indexed new_);
    event Staked(address indexed user, uint256 amount);
    event Unstaked(address indexed user, uint256 amount);
    event StakerRewardsClaimed(address indexed user, uint256 amount);
    event VaultClaimed(address indexed claimer, uint256 amount, uint256 total24h);

    // ═══ MODIFIERS ════════════════════════════════════════════════════════════
    modifier onlyAuthority() { require(msg.sender == currentAuthority, "PP: not authority"); _; }
    modifier notGraduated() { require(!graduated, "PP: graduated"); _; }
    modifier onlyInitialized() { require(initialized, "PP: not init"); _; }
    modifier whenNotPaused() { require(!globalConfig.paused(), "PP: paused"); _; }

    // ═══ CONSTRUCTOR ══════════════════════════════════════════════════════════
    constructor(
        string memory _name, string memory _symbol, address _creator, MigrationTarget _target,
        address _dexAdapter, address _globalConfig, address _circuitBreaker,
        PulseTier _tier, uint256 _feeBps, bool _feesToAgent, string memory _agentName,
        uint8 _partialPct
    ) {
        require(_creator != address(0) && _dexAdapter != address(0) && _globalConfig != address(0) && _circuitBreaker != address(0), "PP: zero addr");
        globalConfig = PulseGlobalConfig(_globalConfig);
        circuitBreaker = PulseCircuitBreaker(_circuitBreaker);

        token = new PulseToken(_name, _symbol, address(this));
        mint = address(token);

        creator = _creator;
        currentAuthority = _creator;
        migrationTarget = _target;
        dexAdapter = _dexAdapter;
        createdAt = block.timestamp;
        graduationTier = _tier;
        poolFeeBps = _feeBps;
        partialMigrationPct = _partialPct;

        // Deploy PrebondConfig
        prebond = new PrebondConfig(address(this), mint, _tier, _feeBps, _feesToAgent,
            address(0), _agentName, true, _partialPct, _creator);

        // Deploy AgentWallet if fees go to agent
        if (_feesToAgent) {
            agent = new AgentWallet(address(this), mint, _agentName);
        } else {
            agent = AgentWallet(payable(address(0)));
        }

        // Deploy VaultClaimTracker
        vaultTracker = new VaultClaimTracker(mint, _creator);
    }

    // ═══ INITIALIZE ═══════════════════════════════════════════════════════════
    function initializePool() external payable {
        require(!initialized, "PP: already init");
        require(msg.sender == creator, "PP: only creator");
        require(msg.value >= 0.02 ether, "PP: min 0.02");

        token.mint(address(this), BONDING_SUPPLY);
        token.mint(address(this), RESERVE_SUPPLY);
        token.mint(address(this), LP_RESERVE_SUPPLY);
        token.revokeMint();

        uint256 netNative = msg.value;
        realNativeReserves = netNative;
        virtualNativeReserves = INITIAL_VIRTUAL_NATIVE + netNative;
        initialized = true;
        poolInitAt = block.timestamp; // anti-snipe window starts

        emit PoolInitialized(creator, mint, migrationTarget);
    }

    // ═══ ANTI-SNIPE HELPERS ═══════════════════════════════════════════════════
    function isAntiSnipeActive() public view returns (bool) {
        return poolInitAt > 0 && block.timestamp < poolInitAt + PulseConstants.ANTI_SNIPE_SECS;
    }

    function effectiveVirtualNative() public view returns (uint256) {
        if (isAntiSnipeActive()) return virtualNativeReserves * PulseConstants.ANTI_SNIPE_MULTIPLIER;
        return virtualNativeReserves;
    }

    function graduationThreshold() public view returns (uint256) {
        if (graduationTier == PulseTier.Fast)     return globalConfig.fastThreshold();
        if (graduationTier == PulseTier.Standard) return globalConfig.standardThreshold();
        return globalConfig.stableThreshold();
    }

    // ═══ BUY ══════════════════════════════════════════════════════════════════
    function buy(uint256 minTokensOut) external payable nonReentrant onlyInitialized notGraduated whenNotPaused {
        uint256 nativeAmount = msg.value;
        require(nativeAmount > 0, "PP: zero");
        require(!isAntiSnipeActive(), "PP: anti-snipe active");

        circuitBreaker.checkVolume(nativeAmount);
        circuitBreaker.checkTxFrequency(msg.sender);

        uint256 evs = effectiveVirtualNative();
        require(virtualNativeReserves > 0 && virtualTokenReserves > 0, "PP: zero reserves");
        require(realTokenReserves > 0 || reserveTokensRemaining > 0, "PP: no tokens");

        (uint256 totalFee, uint256 platformFee, uint256 creatorFee) = BondingCurveMath.calcFees(nativeAmount, poolFeeBps);
        uint256 netNative = nativeAmount - totalFee;

        uint256 tokensOut = BondingCurveMath.calcTokensOut(virtualTokenReserves, evs, netNative);
        require(tokensOut >= minTokensOut, "PP: slippage");

        uint256 available = realTokenReserves + reserveTokensRemaining;
        require(tokensOut <= available, "PP: insufficient tokens");

        // Platform fee
        if (platformFee > 0) {
            (bool ok, ) = PulseConstants.PLATFORM_WALLET.call{value: platformFee}("");
            require(ok, "PP: platform fee failed");
        }

        // Creator/Agent fee
        if (creatorFee > 0) {
            if (address(agent) != address(0)) {
                agent.accumulate(creatorFee);
            } else {
                accumulatedCreatorFees += creatorFee;
            }
        }

        uint256 fromBonding; uint256 fromReserve;
        if (tokensOut <= realTokenReserves) { fromBonding = tokensOut; fromReserve = 0; realTokenReserves -= tokensOut; }
        else { fromBonding = realTokenReserves; fromReserve = tokensOut - realTokenReserves; realTokenReserves = 0; reserveTokensRemaining -= fromReserve; }
        token.transfer(msg.sender, tokensOut);

        virtualNativeReserves += netNative;
        virtualTokenReserves -= tokensOut;
        realNativeReserves += netNative;

        if (realNativeReserves >= graduationThreshold()) emit GraduationReady(mint, realNativeReserves);

        emit Buy(msg.sender, nativeAmount, tokensOut, fromBonding, fromReserve, platformFee, creatorFee, virtualNativeReserves, virtualTokenReserves, realNativeReserves, block.timestamp);
    }

    // ═══ SELL ═════════════════════════════════════════════════════════════════
    function sell(uint256 tokenAmount, uint256 minNativeOut) external nonReentrant onlyInitialized notGraduated whenNotPaused {
        circuitBreaker.checkTxFrequency(msg.sender);
        require(tokenAmount > 0, "PP: zero tokens");

        uint256 grossOut = BondingCurveMath.calcNativeOut(virtualNativeReserves, virtualTokenReserves, tokenAmount);
        require(grossOut <= realNativeReserves, "PP: insufficient sol");

        (uint256 totalFee, uint256 platformFee, uint256 creatorFee) = BondingCurveMath.calcFees(grossOut, poolFeeBps);
        uint256 netOut = grossOut - totalFee;
        require(netOut >= minNativeOut, "PP: slippage");

        token.safeTransferFrom(msg.sender, address(this), tokenAmount);

        if (platformFee > 0) { (bool ok, ) = PulseConstants.PLATFORM_WALLET.call{value: platformFee}(""); require(ok); }
        if (creatorFee > 0) {
            if (address(agent) != address(0)) agent.accumulate(creatorFee);
            else accumulatedCreatorFees += creatorFee;
        }

        (bool sent, ) = msg.sender.call{value: netOut}(""); require(sent, "PP: send failed");

        virtualNativeReserves -= grossOut;
        virtualTokenReserves += tokenAmount;
        realNativeReserves -= grossOut;
        realTokenReserves += tokenAmount;

        emit Sell(msg.sender, tokenAmount, netOut, platformFee, creatorFee, virtualNativeReserves, virtualTokenReserves, realNativeReserves, block.timestamp);
    }

    // ═══ MIGRATE ══════════════════════════════════════════════════════════════
    function migrate() external nonReentrant onlyInitialized {
        require(!graduated, "PP: already graduated");
        uint256 threshold = graduationThreshold();
        require(realNativeReserves >= threshold, "PP: not ready");
        circuitBreaker.checkGraduation();

        graduated = true;

        uint256 solForDex = realNativeReserves;
        uint256 keepSol = 0; uint256 keepTokens = 0;
        if (partialMigrationPct > 0) {
            keepSol = (realNativeReserves * partialMigrationPct) / 100;
            keepTokens = (realTokenReserves * partialMigrationPct) / 100;
            solForDex = realNativeReserves - keepSol;
        }

        token.approve(dexAdapter, LP_RESERVE_SUPPLY);
        IPulseDexAdapter adapter = IPulseDexAdapter(dexAdapter);
        (address _dexPool, ) = adapter.deployPoolAndAddLiquidity{value: solForDex}(mint, LP_RESERVE_SUPPLY, solForDex, creator);
        dexPool = _dexPool;
        lpReserveBalance = 0;

        uint256 remaining = realTokenReserves + reserveTokensRemaining;
        uint256 burnAmount = remaining / 2;
        uint256 vaultAmount = remaining - burnAmount;
        if (burnAmount > 0) token.burn(burnAmount);
        migrationVaultBalance = vaultAmount;
        realTokenReserves = 0;
        reserveTokensRemaining = 0;
        realNativeReserves = 0;

        if (partialMigrationPct > 0) {
            buybackActive = true;
            buybackSolReserves = keepSol;
            buybackTokenReserves = keepTokens;
            buybackVirtualSol = virtualNativeReserves;
            buybackVirtualToken = virtualTokenReserves;
            emit BuybackActivated(mint, keepSol, keepTokens, partialMigrationPct, block.timestamp);
        }

        emit Migrated(mint, migrationTarget, solForDex, LP_RESERVE_SUPPLY, burnAmount, vaultAmount, _dexPool, block.timestamp);
    }

    // ═══ CREATOR FEES ═════════════════════════════════════════════════════════
    function claimFees() external nonReentrant onlyAuthority {
        uint256 amount = accumulatedCreatorFees;
        require(amount > 0.005 ether, "PP: below reserve");
        uint256 claimable = amount - 0.005 ether;
        accumulatedCreatorFees = 0.005 ether;
        (bool ok, ) = currentAuthority.call{value: claimable}(""); require(ok);
        emit FeesClaimed(currentAuthority, claimable);
    }

    function transferAuthority(address newAuthority) external onlyAuthority {
        require(newAuthority != address(0) && newAuthority != currentAuthority, "PP: invalid");
        emit AuthorityTransferred(currentAuthority, newAuthority);
        currentAuthority = newAuthority;
    }

    // ═══ AGENT CLAIM ══════════════════════════════════════════════════════════
    function agentClaim() external nonReentrant {
        require(address(agent) != address(0), "PP: no agent");
        uint256 amount = agent.claim(currentAuthority);
        emit AgentClaimed(mint, agent.agentName(), amount, block.timestamp);
    }

    // ═══ AGENT BUYBACK ════════════════════════════════════════════════════════
    function agentBuyback(uint256 solToSpend, uint8 burnPct) external nonReentrant {
        require(buybackActive && solToSpend <= buybackSolReserves && burnPct <= 100, "PP: buyback invalid");
        require(block.timestamp >= lastBuybackAt + 1 hours, "PP: buyback too soon");

        uint256 tokensOut = BondingCurveMath.calcTokensOut(buybackVirtualToken, buybackVirtualSol, solToSpend);
        require(tokensOut > 0 && tokensOut <= buybackTokenReserves, "PP: buyback empty");

        buybackVirtualSol += solToSpend;
        buybackVirtualToken -= tokensOut;
        buybackSolReserves -= solToSpend;
        buybackTokenReserves -= tokensOut;
        lastBuybackAt = block.timestamp;

        uint256 burnAmount = (tokensOut * burnPct) / 100;
        uint256 treasuryAmount = tokensOut - burnAmount;
        if (burnAmount > 0) token.burn(burnAmount);

        emit BuybackExecuted(mint, solToSpend, burnAmount, treasuryAmount, block.timestamp);
    }

    // ═══ STAKING ══════════════════════════════════════════════════════════════
    function stake(uint256 amount) external nonReentrant {
        require(amount > 0, "PP: zero stake");
        StakerInfo storage s = stakers[msg.sender];
        if (s.amountStaked > 0) {
            uint256 pending = BondingCurveMath.pendingRewards(accumulatedRewardPerToken, s.rewardDebt, s.amountStaked);
            if (pending > 0) { (bool ok, ) = msg.sender.call{value: pending}(""); require(ok); }
        }
        token.safeTransferFrom(msg.sender, address(this), amount);
        s.amountStaked += amount;
        s.rewardDebt = accumulatedRewardPerToken;
        if (s.stakedAt == 0) s.stakedAt = block.timestamp;
        totalStaked += amount;
        emit Staked(msg.sender, amount);
    }

    function unstake(uint256 amount) external nonReentrant {
        StakerInfo storage s = stakers[msg.sender];
        require(s.amountStaked >= amount, "PP: insufficient");
        uint256 pending = BondingCurveMath.pendingRewards(accumulatedRewardPerToken, s.rewardDebt, s.amountStaked);
        if (pending > 0) { (bool ok, ) = msg.sender.call{value: pending}(""); require(ok); }
        s.amountStaked -= amount;
        s.rewardDebt = accumulatedRewardPerToken;
        totalStaked -= amount;
        token.transfer(msg.sender, amount);
        emit Unstaked(msg.sender, amount);
    }

    function claimStakerRewards() external nonReentrant {
        StakerInfo storage s = stakers[msg.sender];
        uint256 pending = BondingCurveMath.pendingRewards(accumulatedRewardPerToken, s.rewardDebt, s.amountStaked);
        require(pending > 0, "PP: no rewards");
        s.rewardDebt = accumulatedRewardPerToken;
        (bool ok, ) = msg.sender.call{value: pending}(""); require(ok);
        emit StakerRewardsClaimed(msg.sender, pending);
    }

    // ═══ VAULT CLAIM (500K/24h cap) ══════════════════════════════════════════
    function claimMigrationVault() external onlyAuthority {
        require(graduated && migrationVaultBalance > 0, "PP: no vault");
        uint256 amount = migrationVaultBalance > 500_000 * 1e18 ? 500_000 * 1e18 : migrationVaultBalance;
        vaultTracker.recordClaim(amount);
        migrationVaultBalance -= amount;
        token.transfer(currentAuthority, amount);
        emit VaultClaimed(currentAuthority, amount, vaultTracker.tokensClaimed24h());
    }

    // ═══ LP FEES ═════════════════════════════════════════════════════════════
    function claimLpFees() external nonReentrant {
        require(graduated && dexPool != address(0), "PP: not graduated");
        IPulseDexAdapter adapter = IPulseDexAdapter(dexAdapter);
        (uint256 nativeFees, ) = adapter.claimLpFees(dexPool, mint);
        if (nativeFees > 0) {
            (, uint256 pf, uint256 cf) = BondingCurveMath.calcFees(nativeFees, poolFeeBps);
            if (pf > 0) { (bool ok, ) = PulseConstants.PLATFORM_WALLET.call{value: pf}(""); require(ok); }
            accumulatedCreatorFees += cf;
        }
    }

    // ═══ VIEW ════════════════════════════════════════════════════════════════
    function currentPrice() external view returns (uint256) {
        if (virtualTokenReserves == 0) return 0;
        return (virtualNativeReserves * 1e18) / virtualTokenReserves;
    }
    function isReadyToGraduate() external view returns (bool) {
        return !graduated && realNativeReserves >= graduationThreshold();
    }
    function pendingStakerRewards(address user) external view returns (uint256) {
        StakerInfo storage s = stakers[user];
        return BondingCurveMath.pendingRewards(accumulatedRewardPerToken, s.rewardDebt, s.amountStaked);
    }

    receive() external payable {}
}