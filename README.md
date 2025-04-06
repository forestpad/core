# Forest Pad
Forest Pad is an innovative platform built on Solana that bridges the gap between Liquid Staking Tokens (LSTs) and project fundraising. By leveraging Solana's SPL Stake Pool, Forest Lab enables projects to create their own LSTs, manage staking rewards, and build engaged communities.

## Core Concepts
- **LST Creation**: Projects can launch their own LSTs through Solana's SPL Stake Pool
- **Reward Management**: Automated epoch reward distribution with customizable fee structures
- **Staking & Lockups**: Users can stake SOL to receive project-specific LSTs and lock them for bonus rewards
- **Restaking Capabilities**: Advanced yield strategies through restaking mechanisms
- **Multisig Management**: Enhanced security with multi-signature control for key project operations

## Technical Stack
- **Blockchain**: Solana
- **Smart Contract Framework**: Anchor Framework
- **Language**: Rust
- **Token Standard**: SPL
- **Integration**: Sanctum API for SPL Stake Pool management
- **Frontend Integration**: TypeScript/JavaScript with Anchor client

## Key Features

### Platform Management
The platform acts as a central hub for managing all projects within the Forest Lab ecosystem, handling global settings and collecting platform fees.

```rust
pub fn initialize_platform(
    ctx: Context<InitializePlatform>,
    platform_fee: u16,
    min_stake_amount: u64,
    admin_wallet: Pubkey,
) -> Result<()> {
    let platform = &mut ctx.accounts.platform;
    
    // Validate fee percentages don't exceed 10000 (100%)
    require!(platform_fee <= 10000, PlatformError::InvalidFeePercentage);
    
    // Initialize platform with the given parameters
    platform.admin = admin_wallet;
    platform.platform_fee = platform_fee;
    platform.min_stake_amount = min_stake_amount;
    platform.total_staked = 0;
    platform.project_count = 0;
    platform.is_paused = false;
    platform.version = 1;
    
    // Emit platform initialization event
    emit!(PlatformInitialized {
        admin: admin_wallet,
        platform_fee,
        min_stake_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Project Registration
Projects can register on the platform with their own branding, LST token details, and fundraising goals.

```rust
pub fn register_project(
    ctx: Context<RegisterProject>,
    name: String,
    symbol: String,
    description: String,
    website: String,
    image_uri: String,
    funding_goal: u64,
    duration: i64,
    lst_mint: Pubkey,
    apy_estimate: u16,
) -> Result<()> {
    let platform = &mut ctx.accounts.platform;
    let project = &mut ctx.accounts.project;
    let authority = &ctx.accounts.authority;
    
    // Validate inputs
    require!(!name.is_empty(), PlatformError::InvalidProjectName);
    require!(!symbol.is_empty(), PlatformError::InvalidTokenSymbol);
    require!(funding_goal > 0, PlatformError::InvalidFundingGoal);
    require!(duration > 0, PlatformError::InvalidDuration);
    require!(apy_estimate <= 2000, PlatformError::InvalidAPYEstimate);
    
    // Initialize project data
    project.name = name;
    project.symbol = symbol;
    project.description = description;
    project.website = website;
    project.image_uri = image_uri;
    project.authority = authority.key();
    project.lst_mint = lst_mint;
    project.funding_goal = funding_goal;
    project.total_staked = 0;
    project.unique_stakers = 0;
    project.start_time = Clock::get()?.unix_timestamp;
    project.end_time = Clock::get()?.unix_timestamp + duration;
    project.apy_estimate = apy_estimate;
    project.rewards_claimed = 0;
    project.platform_fees_collected = 0;
    project.project_fees_collected = 0;
    project.is_active = true;
    project.current_epoch = Clock::get()?.epoch;
    
    // Update platform statistics
    platform.project_count += 1;
    
    // Emit project registration event
    emit!(ProjectRegistered {
        project: project.key(),
        authority: authority.key(),
        name: name.clone(),
        symbol: symbol.clone(),
        funding_goal,
        duration,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Staking Management
The platform handles staking operations through the Sanctum API, recording all staking activities on-chain for transparency.

```rust
pub fn record_project_stake(
    ctx: Context<RecordProjectStake>,
    amount: u64,
    lst_amount: u64,
) -> Result<()> {
    let platform = &mut ctx.accounts.platform;
    let project = &mut ctx.accounts.project;
    let user_stake = &mut ctx.accounts.user_stake;
    let user = &ctx.accounts.user;
    
    // Validate stake amount
    require!(amount >= platform.min_stake_amount, PlatformError::InsufficientStakeAmount);
    
    // Check if this is the first stake from this user
    let is_new_staker = user_stake.total_staked == 0;
    
    // Update user stake record
    user_stake.user = user.key();
    user_stake.project = project.key();
    user_stake.total_staked += amount;
    user_stake.lst_balance += lst_amount;
    user_stake.last_stake_time = Clock::get()?.unix_timestamp;
    user_stake.last_stake_epoch = Clock::get()?.epoch;
    
    // Update project statistics
    project.total_staked += amount;
    if is_new_staker {
        project.unique_stakers += 1;
    }
    
    // Update platform statistics
    platform.total_staked += amount;
    
    // Emit staking event
    emit!(StakeRecorded {
        user: user.key(),
        project: project.key(),
        amount,
        lst_amount,
        timestamp: Clock::get()?.unix_timestamp,
        epoch: Clock::get()?.epoch,
    });
    
    Ok(())
}
```

### Epoch Rewards Processing
At each epoch boundary, rewards are processed and distributed according to predefined rules. The platform initially collects 100% of epoch rewards to enable precise SOL pegging and custom redistribution.

```rust
pub fn process_epoch_rewards(
    ctx: Context<ProcessEpochRewards>,
    epoch: u64,
    total_rewards: u64,
) -> Result<()> {
    let platform = &mut ctx.accounts.platform;
    let project = &mut ctx.accounts.project;
    let reward_vault = &ctx.accounts.reward_vault;
    
    // Validate inputs
    require!(epoch == Clock::get()?.epoch - 1, PlatformError::InvalidEpoch);
    require!(project.current_epoch < epoch, PlatformError::EpochAlreadyProcessed);
    
    // Calculate fee splits
    let platform_fee = (total_rewards * platform.platform_fee as u64) / 10000;
    let project_fee = (total_rewards * project.project_fee as u64) / 10000;
    let user_rewards = total_rewards - platform_fee - project_fee;
    
    // Update project reward statistics
    project.current_epoch = epoch;
    project.total_rewards_for_epoch = total_rewards;
    project.user_rewards_for_epoch = user_rewards;
    project.platform_fees_collected += platform_fee;
    project.project_fees_collected += project_fee;
    
    // Calculate APY based on rewards and total staked
    let annualized_rewards = total_rewards * 365 / 2; // Assuming ~2 days per epoch
    let apy = if project.total_staked > 0 {
        (annualized_rewards * 10000) / project.total_staked
    } else {
        0
    };
    
    // Update APY estimate (smoothed)
    project.apy_estimate = ((project.apy_estimate as u64 * 9 + apy) / 10) as u16;
    
    // Emit reward processing event
    emit!(EpochRewardsProcessed {
        project: project.key(),
        epoch,
        total_rewards,
        user_rewards,
        platform_fee,
        project_fee,
        apy: project.apy_estimate,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Lockup System
Users can lock their LSTs for a period to earn bonus rewards, encouraging long-term commitment.

```rust
pub fn create_lockup(
    ctx: Context<CreateLockup>,
    amount: u64,
    duration: i64,
) -> Result<()> {
    let project = &ctx.accounts.project;
    let user_stake = &mut ctx.accounts.user_stake;
    let lockup = &mut ctx.accounts.lockup;
    let user = &ctx.accounts.user;
    
    // Validate inputs
    require!(amount > 0, PlatformError::InvalidLockupAmount);
    require!(duration >= 7 * 24 * 60 * 60, PlatformError::InvalidLockupDuration); // Minimum 7 days
    require!(user_stake.lst_balance >= amount, PlatformError::InsufficientLSTBalance);
    
    // Calculate bonus multiplier based on duration
    // 1-3 months: 1.2x, 3-6 months: 1.5x, 6+ months: 2x
    let bonus_multiplier = if duration <= 90 * 24 * 60 * 60 {
        120 // 1.2x
    } else if duration <= 180 * 24 * 60 * 60 {
        150 // 1.5x
    } else {
        200 // 2x
    };
    
    // Initialize lockup data
    lockup.user = user.key();
    lockup.project = project.key();
    lockup.amount = amount;
    lockup.start_time = Clock::get()?.unix_timestamp;
    lockup.end_time = Clock::get()?.unix_timestamp + duration;
    lockup.bonus_multiplier = bonus_multiplier;
    lockup.is_active = true;
    lockup.rewards_claimed = 0;
    
    // Update user stake
    user_stake.locked_amount += amount;
    user_stake.lst_balance -= amount;
    
    // Emit lockup creation event
    emit!(LockupCreated {
        user: user.key(),
        project: project.key(),
        lockup: lockup.key(),
        amount,
        duration,
        bonus_multiplier,
        end_time: lockup.end_time,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Reward Claim
Users can claim their accumulated rewards from staking and lockups.

```rust
pub fn claim_rewards(
    ctx: Context<ClaimRewards>,
    amount: u64,
) -> Result<()> {
    let project = &mut ctx.accounts.project;
    let user_stake = &mut ctx.accounts.user_stake;
    let user = &ctx.accounts.user;
    let reward_vault = &ctx.accounts.reward_vault;
    
    // Validate inputs
    require!(amount > 0, PlatformError::InvalidClaimAmount);
    require!(user_stake.pending_rewards >= amount, PlatformError::InsufficientRewards);
    
    // Update user stake
    user_stake.pending_rewards -= amount;
    user_stake.total_rewards_claimed += amount;
    user_stake.last_claim_time = Clock::get()?.unix_timestamp;
    
    // Update project statistics
    project.rewards_claimed += amount;
    
    // Emit reward claim event
    emit!(RewardsClaimed {
        user: user.key(),
        project: project.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Restaking Implementation
Advanced yield strategy through restaking mechanisms.

```rust
pub fn restake_rewards(
    ctx: Context<RestakeRewards>,
    amount: u64,
) -> Result<()> {
    let project = &mut ctx.accounts.project;
    let user_stake = &mut ctx.accounts.user_stake;
    let user = &ctx.accounts.user;
    
    // Validate inputs
    require!(amount > 0, PlatformError::InvalidRestakeAmount);
    require!(user_stake.pending_rewards >= amount, PlatformError::InsufficientRewards);
    
    // Calculate LST amount to mint (using current exchange rate)
    let lst_amount = (amount * project.lst_exchange_rate) / 10000;
    
    // Update user stake
    user_stake.pending_rewards -= amount;
    user_stake.total_staked += amount;
    user_stake.lst_balance += lst_amount;
    user_stake.last_stake_time = Clock::get()?.unix_timestamp;
    
    // Update project statistics
    project.total_staked += amount;
    
    // Emit restaking event
    emit!(RewardsRestaked {
        user: user.key(),
        project: project.key(),
        amount,
        lst_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

## Setup and Deployment
### Prerequisites
- Rust 1.70+ and Cargo
- Solana CLI 1.16+
- Anchor Framework 0.30.1+
- Node.js 16+ and Yarn

### Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/team-seoulana/core.git
   cd core
   ```
2. Install dependencies:
   ```bash
   yarn install
   ```
3. Configure Anchor.toml:
   ```toml
   [toolchain]
   anchor_version = "0.31.0"
   
   [programs.devnet]  # or mainnet, localnet
   forest_lab = "4QfE5Y7LiQrGp2TuT84vLrgz823KM7Xaq6iSEVYw5yX6"
   ```

### Build and Test
1. Build the program:
   ```bash
   anchor build
   ```
2. Run tests:
   ```bash
   anchor test
   ```
### Deployment
1. Deploy to the Solana devnet:
   ```bash
   anchor deploy --provider.cluster devnet
   ```
2. To deploy to Solana mainnet:
   ```bash
   anchor deploy --provider.cluster mainnet
   ```

## Contract Structure
The contract is organized into multiple files for better code organization:

- **lib.rs**: Main program logic and instruction handlers
- **instruction.rs**: Account validation structures
- **state.rs**: Data structures for on-chain state
- **error.rs**: Custom error definitions
- **events.rs**: Event definitions for off-chain monitoring

## Integration with Sanctum

This project relies on Sanctum's API for SPL Stake Pool operations. The Forest Lab contract handles the business logic and record-keeping, while actual staking operations are performed through Sanctum.

## Future Developments

### Enhanced Fee Redistribution System
Forest Pad's unique approach to fee management involves collecting 100% of epoch rewards initially through the platform's management account. This enables several key advantages. The fee redistribution system provides these benefits.
1. **Perfect SOL Pegging**: By controlling the entire reward flow, Forest Pad maintains precise SOL pegging
2. **Customizable Revenue Models**: Projects can adjust reward distributions based on their unique needs (typically 2.5% to platform and flexible allocation of the remainder)
3. **Enhanced Liquidity Management**: Prevents unexpected liquidity migrations by providing structural incentives

### Project Pledging System
To help projects build stronger communities around their LSTs, Forest Pad is developing a pledging system that allows LST holders to lock their tokens for additional project-specific rewards:
* Encourage long-term LST holding through additional incentives
* Distribute project tokens to their most committed supporters
* Create marketing campaigns around LST locking events

### JitoSOL Integration
Future versions will expand to support Jito's MEV rewards, allowing for even greater yields through JitoSOL integration. This will provide additional value streams for LST holders while maintaining the same seamless user experience.

* Capture additional MEV rewards beyond standard staking yields
* Provide more competitive APYs to LST holders
* Offer projects enhanced revenue streams

## README: Understanding Forest Pad's Fee System
Forest Pad introduces a revolutionary approach to LST management through its comprehensive fee redistribution system. Unlike traditional LST platforms with rigid fee structures, Forest Pad collects epoch rewards and enables dynamic redistribution:

### Core Mechanism
1. **Initial Collection**: All epoch rewards from validators flow to Forest Pad's account
2. **Manager Control**: Project managers (or automated cranks) trigger redistribution
3. **Flexible Allocation**: Standard split is 2.5% to platform, with the remainder divided between project treasury and LST holders according to project needs

### For Projects
By using Forest Pad, projects gain:
* Complete control over reward distribution percentages
* Ability to adjust distributions as project needs change
* Strong LST holder retention through optimized incentives
* Seamless integration with project-specific marketing campaigns
