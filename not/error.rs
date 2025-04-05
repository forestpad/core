use anchor_lang::prelude::*;

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