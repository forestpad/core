use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("HHuJZNCk51Zz3KtBfAxpzMsMHWR94W9KUoErhsMUVreL");

#[program]
pub mod core_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    // 새로운 기능: 지갑 정보 확인
    pub fn check_wallet_info(ctx: Context<CheckWallet>) -> Result<()> {
        let wallet = &ctx.accounts.wallet;

        // 지갑 정보 출력
        msg!("지갑 주소: {}", wallet.key());
        msg!("지갑 잔액: {} lamports", wallet.lamports());
        msg!("지갑 소유자: {}", wallet.owner);

        // 추가 정보 수집
        let is_signer = wallet.is_signer;
        let is_writable = wallet.is_writable;

        msg!("지갑 서명자 여부: {}", is_signer);
        msg!("지갑 쓰기 가능 여부: {}", is_writable);

        // 현재 시간 정보
        let clock = Clock::get()?;
        msg!("현재 슬롯: {}", clock.slot);
        msg!("유닉스 타임스탬프: {}", clock.unix_timestamp);

        Ok(())
    }

    // 기능 추가: 메시지 저장하기
    pub fn save_message(ctx: Context<SaveMessage>, message: String) -> Result<()> {
        let message_account = &mut ctx.accounts.message_account;
        let author = &ctx.accounts.author;

        // 메시지 계정 초기화
        message_account.author = *author.key;
        message_account.message = message;
        message_account.timestamp = Clock::get()?.unix_timestamp;

        msg!("메시지가 저장되었습니다: {}", message_account.message);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

// 지갑 정보 확인을 위한 계정 구조체
#[derive(Accounts)]
pub struct CheckWallet<'info> {
    #[account(mut)]
    pub wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 메시지 저장을 위한 계정 구조체
#[derive(Accounts)]
pub struct SaveMessage<'info> {
    #[account(mut)]
    pub author: Signer<'info>,

    #[account(
        init,
        payer = author,
        space = 8 + 32 + 4 + 256 + 8, // 계정 헤더 + pubkey + 문자열 길이 + 최대 문자열 크기 + 타임스탬프
        seeds = [b"message", author.key().as_ref()],
        bump
    )]
    pub message_account: Account<'info, MessageAccount>,

    pub system_program: Program<'info, System>,
}

// 메시지 저장을 위한 계정 데이터 구조
#[account]
pub struct MessageAccount {
    pub author: Pubkey,
    pub message: String,
    pub timestamp: i64,
}