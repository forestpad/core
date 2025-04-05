// use anchor_lang::prelude::*;
// use anchor_lang::solana_program::{
//     program::{invoke, invoke_signed},
//     system_instruction,
// };
// use anchor_spl::{
//     associated_token::{self, AssociatedToken},
//     token::{self, Mint, Token, TokenAccount, TokenInterface, MintInterface},
// };
// use std::convert::TryInto;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    system_instruction,
};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Token},
    token_interface::{TokenInterface, TokenAccount, Mint, MintInterface},
};
use std::convert::TryInto;

declare_id!("4QfE5Y7LiQrGp2TuT84vLrgz823KM7Xaq6iSEVYw5yX6");

// ProjectStatus enum for instruction
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectStatus {
    /// 활성 상태
    Active,
    /// 일시 중지
    Paused,
    /// 완료
    Completed,
    /// 취소
    Cancelled,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        ProjectStatus::Active
    }
}

/// 플랫폼 정보 저장 구조체
#[account]
#[derive(Default)]
pub struct Platform {
    /// 플랫폼 관리자 주소
    pub authority: Pubkey,
    /// 관리자 지갑 주소
    pub admin_wallet: Pubkey,
    /// 플랫폼 수수료 백분율 (100 = 1%)
    pub platform_fee: u16,
    /// 최소 스테이킹 금액 (lamports)
    pub min_stake_amount: u64,
    /// 플랫폼 활성화 상태
    pub is_active: bool,
    /// 총 프로젝트 수
    pub total_projects: u64,
    /// 총 스테이킹된 SOL 양
    pub total_staked_sol: u64,
    /// 생성 시간 (Unix timestamp)
    pub created_at: i64,
    /// PDA 범프
    pub bump: u8,
}

/// 프로젝트 정보 저장 구조체
#[account]
#[derive(Default)]
pub struct Project {
    /// 프로젝트 생성자 주소
    pub creator: Pubkey,
    /// 연결된 플랫폼 주소
    pub platform: Pubkey,
    /// 프로젝트 이름
    pub name: String,
    /// 프로젝트 심볼
    pub symbol: String,
    /// 프로젝트 설명
    pub description: String,
    /// 프로젝트 웹사이트 URL
    pub website: String,
    /// 프로젝트 이미지 URI
    pub image_uri: String,
    /// 펀딩 목표 금액 (lamports)
    pub funding_goal: u64,
    /// 모금된 금액 (lamports)
    pub funds_raised: u64,
    /// 지원자 수
    pub supporters_count: u64,
    /// LST 민트 주소
    pub lst_mint: Pubkey,
    /// 프로젝트 상태
    pub status: ProjectStatus,
    /// 프로젝트 생성 시간 (Unix timestamp)
    pub created_at: i64,
    /// 프로젝트 종료 시간 (Unix timestamp)
    pub end_time: i64,
    /// 자금 청구 여부
    pub funds_claimed: bool,
    /// 프로젝트 매니저 수수료 백분율 (100 = 1%)
    pub manager_fee_percentage: u16,
    /// 수익금 지불 지갑 주소
    pub payout_wallet: Pubkey,
    /// 예상 APY (100 = 1%)
    pub apy_estimate: u16,
    /// 총 분배된 보상 양
    pub total_rewards_distributed: u64,
    /// PDA 범프
    pub bump: u8,
}

/// 스테이킹 정보 저장 구조체
#[account]
#[derive(Default)]
pub struct StakeInfo {
    /// 사용자 주소
    pub user: Pubkey,
    /// 프로젝트 주소
    pub project: Pubkey,
    /// 초기 스테이킹 금액 (lamports)
    pub initial_stake_amount: u64,
    /// 현재 LST 수량
    pub current_lst_amount: u64,
    /// 첫 스테이킹 시간 (Unix timestamp)
    pub first_stake_time: i64,
    /// 마지막 스테이킹 시간 (Unix timestamp)
    pub last_stake_time: i64,
    /// 청구한 보상 양
    pub rewards_claimed: u64,
    /// 마지막 보상 청구 시간
    pub last_claim_time: i64,
    /// PDA 범프
    pub bump: u8,
}

/// 에포크 보상 정보 저장 구조체
#[account]
#[derive(Default)]
pub struct RewardsInfo {
    /// 프로젝트 주소
    pub project: Pubkey,
    /// 에포크 번호
    pub epoch: u64,
    /// 총 보상 금액 (LST)
    pub total_rewards: u64,
    /// 플랫폼 수수료 금액 (LST)
    pub platform_fee: u64,
    /// 프로젝트 배분 금액 (LST)
    pub project_rewards: u64,
    /// 처리 완료 여부
    pub processed: bool,
    /// 스왑된 총 금액 (USDC)
    pub swapped_amount: u64,
    /// 프로젝트 수수료 (USDC)
    pub project_fee: u64,
    /// 프로젝트 배분 금액 (USDC)
    pub project_amount: u64,
    /// 기록 시간 (Unix timestamp)
    pub timestamp: i64,
    /// PDA 범프
    pub bump: u8,
}

/// LST 락업 정보 저장 구조체
#[account]
#[derive(Default)]
pub struct Lockup {
    /// 사용자 주소
    pub user: Pubkey,
    /// 프로젝트 주소
    pub project: Pubkey,
    /// LST 민트 주소
    pub lst_mint: Pubkey,
    /// 락업된 LST 금액
    pub amount: u64,
    /// 락업 시작 시간 (Unix timestamp)
    pub start_time: i64,
    /// 락업 종료 시간 (Unix timestamp)
    pub end_time: i64,
    /// 락업 해제 여부
    pub is_released: bool,
    /// 해제 시간 (해제된 경우)
    pub release_time: i64,
    /// 보너스 비율 (100 = 1%)
    pub bonus_percentage: u16,
    /// PDA 범프
    pub bump: u8,
}

/// 크랭크 정보 저장 구조체
#[account]
#[derive(Default)]
pub struct CrankInfo {
    /// 마지막으로 실행된 에포크 번호
    pub last_executed_epoch: u64,
    /// 마지막 실행 시간 (Unix timestamp)
    pub last_execution_time: i64,
    /// 실행 횟수
    pub execution_count: u64,
    /// 허가된 크랭커 목록
    pub authorized_crankers: Vec<Pubkey>,
    /// PDA 범프
    pub bump: u8,
}

/// 리스테이킹 설정 저장 구조체
#[account]
#[derive(Default)]
pub struct RestakeConfig {
    /// 프로젝트 주소
    pub project: Pubkey,
    /// 소스 LST 민트 주소
    pub source_lst_mint: Pubkey,
    /// 타겟 LST 민트 주소
    pub target_lst_mint: Pubkey,
    /// 리스테이킹 비율 (100 = 1%)
    pub restake_percentage: u16,
    /// 활성화 상태
    pub is_active: bool,
    /// PDA 범프
    pub bump: u8,
}

/// 멀티시그 설정 저장 구조체
#[account]
#[derive(Default)]
pub struct MultisigConfig {
    /// 프로젝트 주소
    pub project: Pubkey,
    /// 서명자 목록
    pub signers: Vec<Pubkey>,
    /// 필요한 서명 임계값
    pub threshold: u8,
    /// 활성화 상태
    pub is_active: bool,
    /// PDA 범프
    pub bump: u8,
}

impl Platform {
    pub const SPACE: usize = 8 +   // 디스크리미네이터
                             32 +  // authority
                             32 +  // admin_wallet
                             2 +   // platform_fee
                             8 +   // min_stake_amount
                             1 +   // is_active
                             8 +   // total_projects
                             8 +   // total_staked_sol
                             8 +   // created_at
                             1 +   // bump
                             64;   // 여유 공간
}

impl Project {
    pub const SPACE: usize = 8 +   // 디스크리미네이터
                             32 +  // creator
                             32 +  // platform
                             36 +  // name (최대 32자)
                             16 +  // symbol (최대 12자)
                             256 + // description (최대 250자)
                             100 + // website (최대 100자)
                             100 + // image_uri (최대 100자)
                             8 +   // funding_goal
                             8 +   // funds_raised
                             8 +   // supporters_count
                             32 +  // lst_mint
                             1 +   // status
                             8 +   // created_at
                             8 +   // end_time
                             1 +   // funds_claimed
                             2 +   // manager_fee_percentage
                             32 +  // payout_wallet
                             2 +   // apy_estimate
                             8 +   // total_rewards_distributed
                             1 +   // bump
                             100;  // 여유 공간
}

impl StakeInfo {
    pub const SPACE: usize = 8 +   // 디스크리미네이터
                             32 +  // user
                             32 +  // project
                             8 +   // initial_stake_amount
                             8 +   // current_lst_amount
                             8 +   // first_stake_time
                             8 +   // last_stake_time
                             8 +   // rewards_claimed
                             8 +   // last_claim_time
                             1 +   // bump
                             32;   // 여유 공간
}

impl RewardsInfo {
    pub const SPACE: usize = 8 +   // 디스크리미네이터
                             32 +  // project
                             8 +   // epoch
                             8 +   // total_rewards
                             8 +   // platform_fee
                             8 +   // project_rewards
                             1 +   // processed
                             8 +   // swapped_amount
                             8 +   // project_fee
                             8 +   // project_amount
                             8 +   // timestamp
                             1 +   // bump
                             32;   // 여유 공간
}

impl Lockup {
    pub const SPACE: usize = 8 +   // 디스크리미네이터
                             32 +  // user
                             32 +  // project
                             32 +  // lst_mint
                             8 +   // amount
                             8 +   // start_time
                             8 +   // end_time
                             1 +   // is_released
                             8 +   // release_time
                             2 +   // bonus_percentage
                             1 +   // bump
                             32;   // 여유 공간
}

impl CrankInfo {
    pub const SPACE: usize = 8 +    // 디스크리미네이터
                             8 +    // last_executed_epoch
                             8 +    // last_execution_time
                             8 +    // execution_count
                             4 + (32 * 10) + // authorized_crankers (최대 10개)
                             1 +    // bump
                             32;    // 여유 공간
}

impl RestakeConfig {
    pub const SPACE: usize = 8 +   // 디스크리미네이터
                             32 +  // project
                             32 +  // source_lst_mint
                             32 +  // target_lst_mint
                             2 +   // restake_percentage
                             1 +   // is_active
                             1 +   // bump
                             32;   // 여유 공간
}

impl MultisigConfig {
    pub const SPACE: usize = 8 +    // 디스크리미네이터
                             32 +   // project
                             4 + (32 * 10) + // signers (최대 10개)
                             1 +    // threshold
                             1 +    // is_active
                             1 +    // bump
                             32;    // 여유 공간
}

#[error_code]
pub enum ForestLabError {
    #[msg("수수료 비율이 유효하지 않습니다")]
    InvalidFeePercentage,
    
    #[msg("플랫폼이 비활성화되어 있습니다")]
    PlatformInactive,
    
    #[msg("프로젝트 이름이 비어 있습니다")]
    EmptyName,

    #[msg("프로젝트 심볼이 비어 있습니다")]
    EmptySymbol,
    
    #[msg("펀딩 목표 금액이 유효하지 않습니다")]
    InvalidFundingGoal,
    
    #[msg("펀딩 기간이 유효하지 않습니다")]
    InvalidDuration,
    
    #[msg("프로젝트가 비활성화되어 있습니다")]
    ProjectInactive,
    
    #[msg("최소 스테이킹 금액보다 작습니다")]
    BelowMinimumStakeAmount,
    
    #[msg("에포크 보상이 이미 처리되었습니다")]
    AlreadyProcessedForEpoch,
    
    #[msg("권한이 없습니다")]
    Unauthorized,
    
    #[msg("유효하지 않은 금액입니다")]
    InvalidAmount,
    
    #[msg("이미 처리되었습니다")]
    AlreadyProcessed,
    
    #[msg("자금이 부족합니다")]
    InsufficientFunds,
    
    #[msg("최소 락업 기간보다 짧습니다")]
    LockupTooShort,
    
    #[msg("락업이 이미 해제되었습니다")]
    AlreadyReleased,
    
    #[msg("락업 기간이 끝나지 않았습니다")]
    LockupNotExpired,
    
    #[msg("비율이 유효하지 않습니다")]
    InvalidPercentage,
    
    #[msg("서명자 수가 유효하지 않습니다")]
    InvalidSignerCount,
    
    #[msg("서명자가 너무 많습니다 (최대 10명)")]
    TooManySigners,
    
    #[msg("임계값이 유효하지 않습니다")]
    InvalidThreshold,
    
    #[msg("임계값이 서명자 수보다 큽니다")]
    ThresholdTooHigh,

    #[msg("스테이킹 정보를 찾을 수 없습니다")]
    StakeInfoNotFound,

    #[msg("LST 잔액이 부족합니다")]
    InsufficientLstBalance,
}

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

/// 플랫폼 초기화를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(platform_fee: u16, min_stake_amount: u64, admin_wallet: Pubkey)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = payer,
        space = Platform::SPACE,
        seeds = [b"platform"],
        bump,
    )]
    pub platform: Account<'info, Platform>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 프로젝트 등록을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    description: String,
    website: String,
    image_uri: String,
    funding_goal: u64,
    duration: i64,
    lst_mint: Pubkey,
    apy_estimate: u16
)]
pub struct RegisterProject<'info> {
    #[account(
        init,
        payer = creator,
        space = Project::SPACE,
        seeds = [b"project", name.as_bytes(), creator.key().as_ref()],
        bump,
    )]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"platform"], 
        bump = platform.bump
    )]
    pub platform: Account<'info, Platform>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 프로젝트 스테이킹 기록을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(amount: u64, lst_amount: u64)]
pub struct RecordProjectStake<'info> {
    #[account(mut, constraint = project.status == ProjectStatus::Active)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"platform"], 
        bump = platform.bump
    )]
    pub platform: Account<'info, Platform>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = StakeInfo::SPACE,
        seeds = [b"stake_info", user.key().as_ref(), project.key().as_ref()],
        bump,
    )]
    pub stake_info: Account<'info, StakeInfo>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 언스테이킹 기록을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(amount: u64, lst_amount: u64)]
pub struct RecordProjectUnstake<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"platform"], 
        bump = platform.bump
    )]
    pub platform: Account<'info, Platform>,
    
    #[account(
        mut,
        seeds = [b"stake_info", user.key().as_ref(), project.key().as_ref()],
        bump = stake_info.bump,
        constraint = stake_info.user == user.key() && stake_info.project == project.key()
    )]
    pub stake_info: Account<'info, StakeInfo>,
    
    pub system_program: Program<'info, System>,
}

/// 에포크 보상 처리를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(epoch: u64, total_rewards: u64)]
pub struct ProcessEpochRewards<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(seeds = [b"platform"], bump = platform.bump)]
    pub platform: Account<'info, Platform>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = RewardsInfo::SPACE,
        seeds = [b"rewards_info", project.key().as_ref()],
        bump,
    )]
    pub rewards_info: Account<'info, RewardsInfo>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 스왑 및 보상 분배를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(reward_amount: u64, usdc_amount: u64)]
pub struct SwapAndDistributeRewards<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(seeds = [b"platform"], bump = platform.bump)]
    pub platform: Account<'info, Platform>,
    
    #[account(
        mut,
        seeds = [b"rewards_info", project.key().as_ref()],
        bump = rewards_info.bump,
    )]
    pub rewards_info: Account<'info, RewardsInfo>,
    
    pub system_program: Program<'info, System>,
}

/// 프로젝트 보상 청구를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(reward_token_amount: u64)]
pub struct ClaimProjectRewards<'info> {
    #[account(
        mut,
        seeds = [b"stake_info", user.key().as_ref(), project.key().as_ref()],
        bump = stake_info.bump,
        constraint = stake_info.user == user.key() && stake_info.project == project.key()
    )]
    pub stake_info: Account<'info, StakeInfo>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub project: Account<'info, Project>,
    
    pub system_program: Program<'info, System>,
}

/// 프로젝트 설정 업데이트를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(manager_fee_percentage: u16, payout_wallet: Option<Pubkey>)]
pub struct UpdateProjectSettings<'info> {
    #[account(
        mut,
        constraint = project.creator == authority.key(),
    )]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// 프로젝트 상태 업데이트를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(new_status: ProjectStatus)]
pub struct UpdateProjectStatus<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(seeds = [b"platform"], bump = platform.bump)]
    pub platform: Account<'info, Platform>,
    
    pub system_program: Program<'info, System>,
}

/// LST 락업 생성을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(amount: u64, duration: i64)]
pub struct CreateLockup<'info> {
    #[account(constraint = project.status == ProjectStatus::Active)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = Lockup::SPACE,
        seeds = [b"lockup", user.key().as_ref(), project.key().as_ref()],
        bump,
    )]
    pub lockup: Account<'info, Lockup>,
    
    #[account(
        mut,
        token::authority = user,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenInterface>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lst_mint,
        associated_token::authority = lockup,
    )]
    pub lockup_vault: InterfaceAccount<'info, TokenInterface>,
    
    pub lst_mint: InterfaceAccount<'info, MintInterface>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 락업 해제를 위한 계정 구조체
#[derive(Accounts)]
pub struct ReleaseLockup<'info> {
    #[account(
        mut,
        seeds = [b"lockup", user.key().as_ref(), project.key().as_ref()],
        bump = lockup.bump,
        constraint = lockup.user == user.key(),
        constraint = !lockup.is_released,
    )]
    pub lockup: Account<'info, Lockup>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(constraint = project.key() == lockup.project)]
    pub project: Account<'info, Project>,
    
    #[account(
        mut,
        token::authority = user,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenInterface>,
    
    #[account(
        mut,
        associated_token::mint = lst_mint,
        associated_token::authority = lockup,
    )]
    pub lockup_vault: InterfaceAccount<'info, TokenInterface>,
    
    pub lst_mint: InterfaceAccount<'info, MintInterface>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// 크랭크 업데이트 실행을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(epoch: u64)]
pub struct ExecuteCrankUpdate<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(seeds = [b"platform"], bump = platform.bump)]
    pub platform: Account<'info, Platform>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = CrankInfo::SPACE,
        seeds = [b"crank_info", project.key().as_ref()],
        bump,
    )]
    pub crank_info: Account<'info, CrankInfo>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 플랫폼 설정 업데이트를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(
    platform_fee: Option<u16>,
    min_stake_amount: Option<u64>,
    admin_wallet: Option<Pubkey>,
    is_active: Option<bool>
)]
pub struct UpdatePlatformSettings<'info> {
    #[account(
        mut,
        seeds = [b"platform"],
        bump = platform.bump,
        constraint = platform.authority == authority.key(),
    )]
    pub platform: Account<'info, Platform>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// 크랭커 관리를 위한 계정 구조체
#[derive(Accounts)]
#[instruction(add_crankers: Vec<Pubkey>, remove_crankers: Vec<Pubkey>)]
pub struct ManageCrankers<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(seeds = [b"platform"], bump = platform.bump)]
    pub platform: Account<'info, Platform>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = CrankInfo::SPACE,
        seeds = [b"crankers"],
        bump,
    )]
    pub crank_info: Account<'info, CrankInfo>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 리스테이킹 설정을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(target_lst_mint: Pubkey, restake_percentage: u16)]
pub struct SetupRestaking<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = RestakeConfig::SPACE,
        seeds = [b"restake_config", project.key().as_ref()],
        bump,
    )]
    pub restake_config: Account<'info, RestakeConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 멀티시그 관리 설정을 위한 계정 구조체
#[derive(Accounts)]
#[instruction(signers: Vec<Pubkey>, threshold: u8)]
pub struct SetupMultisigManagement<'info> {
    #[account(mut)]
    pub project: Account<'info, Project>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = MultisigConfig::SPACE,
        seeds = [b"multisig_config", project.key().as_ref()],
        bump,
    )]
    pub multisig_config: Account<'info, MultisigConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[program]
pub mod forest_lab {
    use super::*;

    /// Forest Lab 플랫폼 초기화 함수
    pub fn initialize_platform(
        ctx: Context<InitializePlatform>,
        platform_fee: u16,
        min_stake_amount: u64,
        admin_wallet: Pubkey,
    ) -> Result<()> {
        // 수수료 비율 검증 (최대 2.5%)
        require!(platform_fee <= 250, ForestLabError::InvalidFeePercentage);

        let platform = &mut ctx.accounts.platform;
        let payer = &ctx.accounts.payer;

        // 플랫폼 상태 초기화
        platform.authority = payer.key();
        platform.admin_wallet = admin_wallet;
        platform.platform_fee = platform_fee;
        platform.min_stake_amount = min_stake_amount;
        platform.is_active = true;
        platform.created_at = Clock::get()?.unix_timestamp;
        platform.bump = ctx.bumps.platform;
        platform.total_projects = 0;
        platform.total_staked_sol = 0;

        // 플랫폼 생성 이벤트 발행
        emit!(PlatformCreatedEvent {
            platform: platform.key(),
            authority: platform.authority,
            admin_wallet,
            platform_fee,
            min_stake_amount,
        });

        Ok(())
    }

    /// 프로젝트 등록 함수
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
        // 플랫폼이 활성화되어 있는지 확인
        require!(ctx.accounts.platform.is_active, ForestLabError::PlatformInactive);

        // 기본 검증
        require!(!name.is_empty(), ForestLabError::EmptyName);
        require!(!symbol.is_empty(), ForestLabError::EmptySymbol);
        require!(funding_goal > 0, ForestLabError::InvalidFundingGoal);
        require!(duration > 0, ForestLabError::InvalidDuration);

        let project = &mut ctx.accounts.project;
        let creator = &ctx.accounts.creator;
        let platform = &mut ctx.accounts.platform;

        // 프로젝트 정보 초기화
        project.creator = creator.key();
        project.platform = platform.key();
        project.name = name.clone();
        project.symbol = symbol.clone();
        project.description = description;
        project.website = website;
        project.image_uri = image_uri;
        project.funding_goal = funding_goal;
        project.funds_raised = 0;
        project.supporters_count = 0;
        project.lst_mint = lst_mint;
        project.status = ProjectStatus::Active;
        project.created_at = Clock::get()?.unix_timestamp;
        project.end_time = Clock::get()?.unix_timestamp + duration;
        project.funds_claimed = false;
        project.manager_fee_percentage = 250; // 기본 2.5%
        project.payout_wallet = creator.key(); // 기본값은 생성자 지갑
        project.apy_estimate = apy_estimate; // 예상 APY (100 = 1%)
        project.total_rewards_distributed = 0;
        project.bump = ctx.bumps.project;

        // 플랫폼 통계 업데이트
        platform.total_projects = platform.total_projects.saturating_add(1);

        // 프로젝트 등록 이벤트 발행
        emit!(ProjectRegisteredEvent {
            project: project.key(),
            creator: creator.key(),
            platform: platform.key(),
            name,
            symbol: project.symbol.clone(),
            funding_goal,
            end_time: project.end_time,
            lst_mint,
            apy_estimate,
        });

        Ok(())
    }

    /// 스테이킹 기록 함수 (Sanctum API 호출 후 실행)
    pub fn record_project_stake(
        ctx: Context<RecordProjectStake>,
        amount: u64,
        lst_amount: u64,
    ) -> Result<()> {
        let project = &mut ctx.accounts.project;
        let user = &ctx.accounts.user;
        let platform = &mut ctx.accounts.platform;

        // 프로젝트가 활성 상태인지 확인
        require!(
            project.status == ProjectStatus::Active,
            ForestLabError::ProjectInactive
        );

        // 플랫폼이 활성화되어 있는지 확인
        require!(platform.is_active, ForestLabError::PlatformInactive);

        // 최소 스테이킹 금액 확인
        require!(
            amount >= platform.min_stake_amount,
            ForestLabError::BelowMinimumStakeAmount
        );

        // 스테이킹 기록 생성 또는 업데이트
        let stake_info = &mut ctx.accounts.stake_info;
        
        let is_new_stake = stake_info.user == Pubkey::default();
        
        if is_new_stake {
            // 새 스테이킹 기록 초기화
            stake_info.user = user.key();
            stake_info.project = project.key();
            stake_info.initial_stake_amount = amount;
            stake_info.current_lst_amount = lst_amount;
            stake_info.first_stake_time = Clock::get()?.unix_timestamp;
            stake_info.bump = ctx.bumps.stake_info;
            stake_info.rewards_claimed = 0;
            stake_info.last_claim_time = 0;
        } else {
            // 기존 스테이킹 기록 업데이트
            stake_info.initial_stake_amount = stake_info.initial_stake_amount.saturating_add(amount);
            stake_info.current_lst_amount = stake_info.current_lst_amount.saturating_add(lst_amount);
        }

        stake_info.last_stake_time = Clock::get()?.unix_timestamp;

        // 프로젝트 정보 업데이트
        let is_new_supporter = is_new_stake;
        if is_new_supporter {
            project.supporters_count = project.supporters_count.saturating_add(1);
        }
        project.funds_raised = project.funds_raised.saturating_add(amount);
        
        // 플랫폼 통계 업데이트
        platform.total_staked_sol = platform.total_staked_sol.saturating_add(amount);

        // 스테이킹 이벤트 발행
        emit!(ProjectStakedEvent {
            project: project.key(),
            user: user.key(),
            amount,
            lst_amount,
            is_new_supporter,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// 언스테이킹 기록 함수 (Sanctum API 호출 후 실행)
    pub fn record_project_unstake(
        ctx: Context<RecordProjectUnstake>,
        amount: u64,
        lst_amount: u64,
    ) -> Result<()> {
        let project = &mut ctx.accounts.project;
        let user = &ctx.accounts.user;
        let platform = &mut ctx.accounts.platform;
        let stake_info = &mut ctx.accounts.stake_info;

        // 스테이킹 정보가 존재하는지 확인
        require!(
            stake_info.user == user.key() && stake_info.project == project.key(),
            ForestLabError::StakeInfoNotFound
        );

        // LST 잔액이 충분한지 확인
        require!(
            stake_info.current_lst_amount >= lst_amount,
            ForestLabError::InsufficientLstBalance
        );

        // 스테이킹 정보 업데이트
        let proportion = lst_amount as f64 / stake_info.current_lst_amount as f64;
        let sol_amount_to_remove = (stake_info.initial_stake_amount as f64 * proportion) as u64;

        stake_info.initial_stake_amount = stake_info.initial_stake_amount.saturating_sub(sol_amount_to_remove);
        stake_info.current_lst_amount = stake_info.current_lst_amount.saturating_sub(lst_amount);
        
        // 프로젝트 정보 업데이트
        project.funds_raised = project.funds_raised.saturating_sub(sol_amount_to_remove);
        
        // 스테이킹 정보가 0이 되면 지원자 수 감소
        if stake_info.current_lst_amount == 0 {
            project.supporters_count = project.supporters_count.saturating_sub(1);
        }
        
        // 플랫폼 통계 업데이트
        platform.total_staked_sol = platform.total_staked_sol.saturating_sub(sol_amount_to_remove);

        // 언스테이킹 이벤트 발행
        emit!(ProjectUnstakedEvent {
            project: project.key(),
            user: user.key(),
            sol_amount: sol_amount_to_remove,
            lst_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// 에포크 보상 처리 및 기록 함수
    pub fn process_epoch_rewards(
        ctx: Context<ProcessEpochRewards>,
        epoch: u64,
        total_rewards: u64,
    ) -> Result<()> {
        let project = &mut ctx.accounts.project;
        let platform = &ctx.accounts.platform;
        let rewards_info = &mut ctx.accounts.rewards_info;
        let authority = &ctx.accounts.authority;

        // 권한 확인 (플랫폼 관리자만)
        require!(
            platform.authority == authority.key() || platform.admin_wallet == authority.key(),
            ForestLabError::Unauthorized
        );

        // 이 에포크에 이미 처리되었는지 확인
        if rewards_info.epoch > 0 {
            require!(
                rewards_info.epoch != epoch,
                ForestLabError::AlreadyProcessedForEpoch
            );
        }

        // Forest Lab의 수수료 계산 (기본 2.5%)
        let platform_fee_amount = total_rewards
            .saturating_mul(platform.platform_fee as u64)
            .saturating_div(10000);
            
        // 프로젝트에 분배할 보상 계산
        let project_rewards = total_rewards.saturating_sub(platform_fee_amount);

        // 보상 정보 초기화 또는 업데이트
        rewards_info.project = project.key();
        rewards_info.epoch = epoch;
        rewards_info.total_rewards = total_rewards;
        rewards_info.platform_fee = platform_fee_amount;
        rewards_info.project_rewards = project_rewards;
        rewards_info.processed = false; // 스왑 처리 상태 초기화
        rewards_info.timestamp = Clock::get()?.unix_timestamp;
        if rewards_info.bump == 0 {
            rewards_info.bump = ctx.bumps.rewards_info;
        }
        
        // 프로젝트의 누적 보상 업데이트
        project.total_rewards_distributed = project.total_rewards_distributed.saturating_add(project_rewards);
        
        // 현재 APY 계산 업데이트 (선택적)
        if project.funds_raised > 0 {
            // 간단한 APY 계산 로직 (연간 예상 수익률로 변환)
            // 주의: 이는 매우 단순화된 계산으로, 실제 APY 계산은 더 복잡할 수 있음
            let annualized_rewards = project_rewards.saturating_mul(365 * 2); // 약 2일마다 에포크 가정
            let apy = annualized_rewards
                .saturating_mul(10000)
                .saturating_div(project.funds_raised);
            
            // APY 업데이트 (가중 평균으로)
            project.apy_estimate = ((project.apy_estimate as u64).saturating_mul(9).saturating_add(apy as u64) / 10) as u16;
        }

        // 보상 처리 이벤트 발행
        emit!(EpochRewardsProcessedEvent {
            project: project.key(),
            epoch,
            total_rewards,
            platform_fee: platform_fee_amount,
            project_rewards,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// 에포크 보상을 USDC로 스왑하고 프로젝트에 분배하는 함수
    pub fn swap_and_distribute_rewards(
        ctx: Context<SwapAndDistributeRewards>,
        reward_amount: u64,
        usdc_amount: u64,
    ) -> Result<()> {
        let rewards_info = &mut ctx.accounts.rewards_info;
        let project = &ctx.accounts.project;
        let authority = &ctx.accounts.authority;
        let platform = &ctx.accounts.platform;
        
        // 권한 확인 (플랫폼 관리자만)
        require!(
            platform.authority == authority.key() || platform.admin_wallet == authority.key(),
            ForestLabError::Unauthorized
        );
        
        // 이미 처리되었는지 확인
        require!(!rewards_info.processed, ForestLabError::AlreadyProcessed);
        
        // 스왑된 USDC 금액 확인
        require!(usdc_amount > 0, ForestLabError::InvalidAmount);
        
        // 프로젝트 수수료 계산
        let project_fee = usdc_amount
            .saturating_mul(project.manager_fee_percentage as u64)
            .saturating_div(10000);
        
        // 프로젝트에 전송할 금액
        let project_amount = usdc_amount.saturating_sub(project_fee);
        
        // USDC를 프로젝트 지갑으로 전송
        // 여기서는 이미 스왑이 완료되었다고 가정하고, 분배만 기록
        
        // 보상 정보 업데이트
        rewards_info.processed = true;
        rewards_info.swapped_amount = usdc_amount;
        rewards_info.project_fee = project_fee;
        rewards_info.project_amount = project_amount;
        
        // 스왑 및 분배 이벤트 발행
        emit!(RewardsDistributedEvent {
            project: project.key(),
            epoch: rewards_info.epoch,
            lst_amount: reward_amount,
            usdc_amount,
            project_fee,
            project_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// 사용자가 프로젝트 보상을 청구하는 함수
    pub fn claim_project_rewards(
        ctx: Context<ClaimProjectRewards>,
        reward_token_amount: u64,
    ) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;
        let user = &ctx.accounts.user;
        let project = &ctx.accounts.project;
        
        // 스테이킹 정보가 존재하는지 확인
        require!(
            stake_info.user == user.key() && stake_info.project == project.key(),
            ForestLabError::StakeInfoNotFound
        );
        
        // 청구 가능한 보상이 있는지 확인
        require!(
            reward_token_amount > 0,
            ForestLabError::InvalidAmount
        );
        
        // 토큰 이체는 프론트엔드에서 처리 (청구 기록만 업데이트)
        stake_info.rewards_claimed = stake_info.rewards_claimed.saturating_add(reward_token_amount);
        stake_info.last_claim_time = Clock::get()?.unix_timestamp;
        
        // 보상 청구 이벤트 발행
        emit!(RewardsClaimedEvent {
            project: project.key(),
            user: user.key(),
            reward_token_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// 프로젝트 수수료 설정 업데이트 함수
    pub fn update_project_fee(
        ctx: Context<UpdateProjectSettings>,
        manager_fee_percentage: u16,
        payout_wallet: Option<Pubkey>,
    ) -> Result<()> {
        let project = &mut ctx.accounts.project;
        let authority = &ctx.accounts.authority;
        
        // 권한 확인 (프로젝트 생성자만)
        require!(
            project.creator == authority.key(),
            ForestLabError::Unauthorized
        );
        
        // 수수료 비율 유효성 검증 (최대 100%)
        require!(
            manager_fee_percentage <= 10000,
            ForestLabError::InvalidFeePercentage
        );
        
        // 프로젝트 수수료 설정 업데이트
        project.manager_fee_percentage = manager_fee_percentage;
        
        // 지불 지갑 업데이트 (선택적)
        if let Some(wallet) = payout_wallet {
            project.payout_wallet = wallet;
        }
        
        // 설정 업데이트 이벤트 발행
        emit!(ProjectSettingsUpdatedEvent {
            project: project.key(),
            manager_fee_percentage,
            payout_wallet: project.payout_wallet,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// 프로젝트 상태 업데이트 함수
    pub fn update_project_status(
        ctx: Context<UpdateProjectStatus>,
        new_status: ProjectStatus,
    ) -> Result<()> {
        let project = &mut ctx.accounts.project;
        let authority = &ctx.accounts.authority;
        let platform = &ctx.accounts.platform;
        
        // 권한 확인 (프로젝트 생성자 또는 플랫폼 관리자)
        require!(
            project.creator == authority.key() || 
            platform.authority == authority.key() || 
            platform.admin_wallet == authority.key(),
            ForestLabError::Unauthorized
        );
        
        // 이전 상태 저장
        let previous_status = project.status;
        
        // 상태 업데이트
        project.status = new_status;
        
        // 상태 업데이트 이벤트 발행
        emit!(ProjectStatusUpdatedEvent {
            project: project.key(),
            previous_status,
            new_status,
            updated_by: authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// LST 락업 생성 함수
    pub fn create_lockup(
        ctx: Context<CreateLockup>,
        amount: u64,
        duration: i64,
    ) -> Result<()> {
        // 최소 락업 기간 확인 (1일)
        require!(duration >= 86400, ForestLabError::LockupTooShort);

        let lockup = &mut ctx.accounts.lockup;
        let user = &ctx.accounts.user;
        let project = &ctx.accounts.project;

        // 락업 정보 초기화
        lockup.user = user.key();
        lockup.project = project.key();
        lockup.lst_mint = project.lst_mint;
        lockup.amount = amount;
        lockup.start_time = Clock::get()?.unix_timestamp;
        lockup.end_time = Clock::get()?.unix_timestamp + duration;
        lockup.is_released = false;
        lockup.release_time = 0;
        lockup.bonus_percentage = match duration {
            d if d >= 365 * 86400 => 1000, // 1년 이상: 10% 보너스
            d if d >= 180 * 86400 => 500,  // 6개월 이상: 5% 보너스
            d if d >= 90 * 86400 => 250,   // 3개월 이상: 2.5% 보너스
            _ => 0,                        // 기본: 보너스 없음
        };
        lockup.bump = ctx.bumps.lockup;

        // 사용자의 LST 토큰을 락업 볼트로 전송
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.lockup_vault.to_account_info(),
                    authority: user.to_account_info(),
                },
            ),
            amount,
        )?;

        // 락업 생성 이벤트 발행
        emit!(LockupCreatedEvent {
            lockup: lockup.key(),
            user: user.key(),
            project: project.key(),
            amount,
            start_time: lockup.start_time,
            end_time: lockup.end_time,
            bonus_percentage: lockup.bonus_percentage,
        });

        Ok(())
    }

    /// 락업 해제 함수
    pub fn release_lockup(
        ctx: Context<ReleaseLockup>,
    ) -> Result<()> {
        let lockup = &mut ctx.accounts.lockup;
        let user = &ctx.accounts.user;
        let lockup_vault = &ctx.accounts.lockup_vault;

        // 락업이 이미 해제되었는지 확인
        require!(!lockup.is_released, ForestLabError::AlreadyReleased);

        // 락업 기간이 끝났는지 확인
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time >= lockup.end_time,
            ForestLabError::LockupNotExpired
        );

        // 락업된 LST를 사용자에게 반환
        let seeds = &[
            b"lockup".as_ref(),
            lockup.user.as_ref(),
            lockup.project.as_ref(),
            &[lockup.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: lockup_vault.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: lockup.to_account_info(),
                },
                signer,
            ),
            lockup.amount,
        )?;

        // 락업 정보 업데이트
        lockup.is_released = true;
        lockup.release_time = current_time;

        // 락업 해제 이벤트 발행
        emit!(LockupReleasedEvent {
            lockup: lockup.key(),
            user: user.key(),
            project: lockup.project,
            amount: lockup.amount,
            release_time: current_time,
        });

        Ok(())
    }

    /// 크랭크 업데이트 실행 함수
    pub fn execute_crank_update(
        ctx: Context<ExecuteCrankUpdate>,
        epoch: u64,
    ) -> Result<()> {
        let crank_info = &mut ctx.accounts.crank_info;
        let project = &ctx.accounts.project;
        let authority = &ctx.accounts.authority;
        let platform = &ctx.accounts.platform;
        
        // 권한 확인 (플랫폼 관리자 또는 허가된 크랭커)
        let is_admin = platform.authority == authority.key() || platform.admin_wallet == authority.key();
        require!(
            is_admin || crank_info.authorized_crankers.contains(&authority.key()),
            ForestLabError::Unauthorized
        );
        
        // 크랭크 정보 업데이트
        crank_info.last_executed_epoch = epoch;
        crank_info.last_execution_time = Clock::get()?.unix_timestamp;
        crank_info.execution_count = crank_info.execution_count.saturating_add(1);
        
        // 이 함수는 SPL Stake Pool의 update_stake_pool 명령을 호출하는
        // 외부 프로그램을 트리거하기 위한 이벤트를 발행함
        
        // 크랭크 실행 이벤트 발행
        emit!(CrankExecutedEvent {
            project: project.key(),
            epoch,
            executor: authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// 플랫폼 설정 업데이트 함수
    pub fn update_platform_settings(
        ctx: Context<UpdatePlatformSettings>,
        platform_fee: Option<u16>,
        min_stake_amount: Option<u64>,
        admin_wallet: Option<Pubkey>,
        is_active: Option<bool>,
    ) -> Result<()> {
        let platform = &mut ctx.accounts.platform;
        let authority = &ctx.accounts.authority;

        // 권한 확인
        require!(
            platform.authority == authority.key(),
            ForestLabError::Unauthorized
        );

        // 설정 업데이트
        if let Some(fee) = platform_fee {
            require!(fee <= 250, ForestLabError::InvalidFeePercentage); // 최대 2.5%
            platform.platform_fee = fee;
        }

        if let Some(amount) = min_stake_amount {
            platform.min_stake_amount = amount;
        }

        if let Some(admin) = admin_wallet {
            platform.admin_wallet = admin;
        }

        if let Some(active) = is_active {
            platform.is_active = active;
        }

        // 설정 업데이트 이벤트 발행
        emit!(PlatformSettingsUpdatedEvent {
            platform: platform.key(),
            authority: authority.key(),
            platform_fee: platform.platform_fee,
            min_stake_amount: platform.min_stake_amount,
            admin_wallet: platform.admin_wallet,
            is_active: platform.is_active,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// 크랭커 권한 관리 함수
    pub fn manage_crankers(
        ctx: Context<ManageCrankers>,
        add_crankers: Vec<Pubkey>,
        remove_crankers: Vec<Pubkey>,
    ) -> Result<()> {
        let crank_info = &mut ctx.accounts.crank_info;
        let authority = &ctx.accounts.authority;
        let platform = &ctx.accounts.platform;
        
        // 권한 확인 (플랫폼 관리자만)
        require!(
            platform.authority == authority.key() || platform.admin_wallet == authority.key(),
            ForestLabError::Unauthorized
        );
        
        // 크랭커 추가
        for cranker in add_crankers {
            if !crank_info.authorized_crankers.contains(&cranker) {
                crank_info.authorized_crankers.push(cranker);
            }
        }
        
        // 크랭커 제거
        for cranker in remove_crankers {
            crank_info.authorized_crankers.retain(|&x| x != cranker);
        }
        
        // 크랭커 관리 이벤트 발행
        emit!(CrankersUpdatedEvent {
            platform: platform.key(),
            authorized_crankers: crank_info.authorized_crankers.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
    
    /// 리스테이킹 설정 함수
    pub fn setup_restaking(
        ctx: Context<SetupRestaking>,
        target_lst_mint: Pubkey,
        restake_percentage: u16,
    ) -> Result<()> {
        let restake_config = &mut ctx.accounts.restake_config;
        let project = &ctx.accounts.project;
        let authority = &ctx.accounts.authority;
        
        // 권한 확인 (프로젝트 생성자만)
        require!(
            project.creator == authority.key(),
            ForestLabError::Unauthorized
        );
        
        // 리스테이킹 비율 유효성 검증 (최대 100%)
        require!(
            restake_percentage <= 10000,
            ForestLabError::InvalidPercentage
        );
        
        // 리스테이킹 설정 초기화 또는 업데이트
        restake_config.project = project.key();
        restake_config.source_lst_mint = project.lst_mint;
        restake_config.target_lst_mint = target_lst_mint;
        restake_config.restake_percentage = restake_percentage;
        restake_config.is_active = true;
        restake_config.bump = ctx.bumps.restake_config;
        
        // 리스테이킹 설정 이벤트 발행
        emit!(RestakingConfiguredEvent {
            project: project.key(),
            source_lst_mint: project.lst_mint,
            target_lst_mint,
            restake_percentage,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// 멀티시그 프로젝트 관리 설정 함수
    pub fn setup_multisig_management(
        ctx: Context<SetupMultisigManagement>,
        signers: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        let multisig_config = &mut ctx.accounts.multisig_config;
        let project = &ctx.accounts.project;
        let authority = &ctx.accounts.authority;
        
        // 권한 확인 (프로젝트 생성자만)
        require!(
            project.creator == authority.key(),
            ForestLabError::Unauthorized
        );
        
        // 서명자 수 유효성 검증
        require!(!signers.is_empty(), ForestLabError::InvalidSignerCount);
        require!(signers.len() <= 10, ForestLabError::TooManySigners);
        
        // 임계값 유효성 검증
        require!(threshold > 0, ForestLabError::InvalidThreshold);
        require!(
            threshold as usize <= signers.len(),
            ForestLabError::ThresholdTooHigh
        );
        
        // 멀티시그 설정 초기화
        multisig_config.project = project.key();
        multisig_config.signers = signers.clone();
        multisig_config.threshold = threshold;
        multisig_config.is_active = true;
        multisig_config.bump = ctx.bumps.multisig_config;
        
        // 멀티시그 설정 이벤트 발행
        emit!(MultisigConfiguredEvent {
            project: project.key(),
            signers: signers.clone(),
            threshold,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
}