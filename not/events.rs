use anchor_lang::prelude::*;
use crate::state::ProjectStatus;

// 플랫폼 생성 이벤트
#[event]
pub struct PlatformCreatedEvent {
    pub platform: Pubkey,
    pub authority: Pubkey,
    pub admin_wallet: Pubkey,
    pub platform_fee: u16,
    pub min_stake_amount: u64,
}

// 프로젝트 등록 이벤트
#[event]
pub struct ProjectRegisteredEvent {
    pub project: Pubkey,
    pub creator: Pubkey,
    pub platform: Pubkey,
    pub name: String,
    pub symbol: String,
    pub funding_goal: u64,
    pub end_time: i64,
    pub lst_mint: Pubkey,
    pub apy_estimate: u16,
}

// 프로젝트 스테이킹 이벤트
#[event]
pub struct ProjectStakedEvent {
    pub project: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub lst_amount: u64,
    pub is_new_supporter: bool,
    pub timestamp: i64,
}

// 프로젝트 언스테이킹 이벤트
#[event]
pub struct ProjectUnstakedEvent {
    pub project: Pubkey,
    pub user: Pubkey,
    pub sol_amount: u64,
    pub lst_amount: u64,
    pub timestamp: i64,
}

// 에포크 보상 처리 이벤트
#[event]
pub struct EpochRewardsProcessedEvent {
    pub project: Pubkey,
    pub epoch: u64,
    pub total_rewards: u64,
    pub platform_fee: u64,
    pub project_rewards: u64,
    pub timestamp: i64,
}

// 보상 분배 이벤트
#[event]
pub struct RewardsDistributedEvent {
    pub project: Pubkey,
    pub epoch: u64,
    pub lst_amount: u64,
    pub usdc_amount: u64,
    pub project_fee: u64,
    pub project_amount: u64,
    pub timestamp: i64,
}

// 보상 청구 이벤트
#[event]
pub struct RewardsClaimedEvent {
    pub project: Pubkey,
    pub user: Pubkey,
    pub reward_token_amount: u64,
    pub timestamp: i64,
}

// 프로젝트 설정 업데이트 이벤트
#[event]
pub struct ProjectSettingsUpdatedEvent {
    pub project: Pubkey,
    pub manager_fee_percentage: u16,
    pub payout_wallet: Pubkey,
    pub timestamp: i64,
}

// 프로젝트 상태 업데이트 이벤트
#[event]
pub struct ProjectStatusUpdatedEvent {
    pub project: Pubkey,
    pub previous_status: ProjectStatus,
    pub new_status: ProjectStatus,
    pub updated_by: Pubkey,
    pub timestamp: i64,
}

// 락업 생성 이벤트
#[event]
pub struct LockupCreatedEvent {
    pub lockup: Pubkey,
    pub user: Pubkey,
    pub project: Pubkey,
    pub amount: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub bonus_percentage: u16,
}

// 락업 해제 이벤트
#[event]
pub struct LockupReleasedEvent {
    pub lockup: Pubkey,
    pub user: Pubkey,
    pub project: Pubkey,
    pub amount: u64,
    pub release_time: i64,
}

// 크랭크 실행 이벤트
#[event]
pub struct CrankExecutedEvent {
    pub project: Pubkey,
    pub epoch: u64,
    pub executor: Pubkey,
    pub timestamp: i64,
}

// 플랫폼 설정 업데이트 이벤트
#[event]
pub struct PlatformSettingsUpdatedEvent {
    pub platform: Pubkey,
    pub authority: Pubkey,
    pub platform_fee: u16,
    pub min_stake_amount: u64,
    pub admin_wallet: Pubkey,
    pub is_active: bool,
    pub timestamp: i64,
}

// 크랭커 업데이트 이벤트
#[event]
pub struct CrankersUpdatedEvent {
    pub platform: Pubkey,
    pub authorized_crankers: Vec<Pubkey>,
    pub timestamp: i64,
}

// 리스테이킹 설정 이벤트
#[event]
pub struct RestakingConfiguredEvent {
    pub project: Pubkey,
    pub source_lst_mint: Pubkey,
    pub target_lst_mint: Pubkey,
    pub restake_percentage: u16,
    pub timestamp: i64,
}

// 멀티시그 설정 이벤트
#[event]
pub struct MultisigConfiguredEvent {
    pub project: Pubkey,
    pub signers: Vec<Pubkey>,
    pub threshold: u8,
    pub timestamp: i64,
}
