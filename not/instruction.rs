use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use crate::state::*;

/// ProjectStatus enum for instruction
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
        constraint = user_token_account.mint == project.lst_mint,
        constraint = user_token_account.owner == user.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = project.lst_mint,
        associated_token::authority = lockup,
    )]
    pub lockup_vault: Account<'info, TokenAccount>,
    
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
        constraint = user_token_account.mint == lockup.lst_mint,
        constraint = user_token_account.owner == user.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = lockup.lst_mint,
        associated_token::authority = lockup,
    )]
    pub lockup_vault: Account<'info, TokenAccount>,
    
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