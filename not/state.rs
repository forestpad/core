use anchor_lang::prelude::*;

/// 프로젝트 상태 열거형
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