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
    // Platform initialization logic
    // ...
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
    // Project registration logic
    // ...
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
    // Staking record logic
    // ...
}
```

### Epoch Rewards Processing

At each epoch boundary, rewards are processed and distributed according to predefined rules.

```rust
pub fn process_epoch_rewards(
    ctx: Context<ProcessEpochRewards>,
    epoch: u64,
    total_rewards: u64,
) -> Result<()> {
    // Reward processing logic
    // ...
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
    // Lockup creation logic
    // ...
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

- Enhanced dashboard for staking analytics
- Integration with Jupiter for automated token swaps
- Cross-chain LST bridging capabilities
- Governance systems for decentralized project management
