use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    system_instruction,
};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use std::convert::TryInto;

pub mod instruction;
pub mod state;
pub mod error;
pub mod events;

// 명확한 import 방식으로 필요한 것만 가져오기
use crate::instruction::*;
use crate::state::Platform;
use crate::state::Project;
use crate::state::StakeInfo;
use crate::state::Lockup;
use crate::state::RewardsInfo;
use crate::state::CrankInfo;
use crate::state::RestakeConfig;
use crate::state::MultisigConfig;
use crate::state::ProjectStatus;
use crate::error::*;
use crate::events::*;

declare_id!("4QfE5Y7LiQrGp2TuT84vLrgz823KM7Xaq6iSEVYw5yX6");

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
        new_status: instruction::ProjectStatus,
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

        // instruction의 ProjectStatus를 state의 ProjectStatus로 변환
        let new_state_status = match new_status {
        instruction::ProjectStatus::Active => ProjectStatus::Active,
        instruction::ProjectStatus::Paused => ProjectStatus::Paused,
        instruction::ProjectStatus::Completed => ProjectStatus::Completed,
        instruction::ProjectStatus::Cancelled => ProjectStatus::Cancelled,
        };

        // 상태 업데이트
        project.status = new_state_status;

        // 상태 업데이트 이벤트 발행
        emit!(ProjectStatusUpdatedEvent {
        project: project.key(),
        previous_status,
        new_status: new_state_status,
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