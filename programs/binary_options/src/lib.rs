use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer as transfer_spl, Mint, Token, TokenAccount, Transfer as TransferSPL},
};
declare_id!("HC2oqz2p6DEWfrahenqdq2moUcga9c9biqRBcdK3XKU1");
#[program]
pub mod binary_options {
    use super::*;
    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        ctx.accounts.state.total_xyz_balance = 0;
        ctx.accounts.state.fee_percentage = 0;
        ctx.accounts.state.admin = ctx.accounts.user.key();
        ctx.accounts.state.xyz_mint = ctx.accounts.xyz_mint.key();
        ctx.accounts.state.auth_bump = ctx.bumps.auth;
        ctx.accounts.state.prediction_counter = 1;
        Ok(())
    }
    pub fn create_prediction(
        ctx: Context<CreatePredictionContext>,
        amount: u64,
        token_mint: Pubkey,
        start_timestamp: u64,
        expiry_timestamp: u64,
        start_price: u64,
        end_price: u64,
        prediction_type: String,
    ) -> Result<()> {
        let cpi_accounts = TransferSPL {
            from: ctx.accounts.maker_ata.to_account_info(),
            to: ctx.accounts.xyz_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        transfer_spl(cpi_ctx, amount)?;
        ctx.accounts.prediction_state.amount = amount;
        ctx.accounts.prediction_state.token_mint = token_mint;
        ctx.accounts.prediction_state.start_timestamp = start_timestamp;
        ctx.accounts.prediction_state.start_price = start_price;

        ctx.accounts.prediction_state.prediction_type = prediction_type;
        ctx.accounts.prediction_state.trader = ctx.accounts.maker_ata.key();
        Ok(())
    }
    pub fn settle_prediction(
        ctx: Context<SettlePredictionContext>,
        new_price: u64,
        id: u64,
        taker: Pubkey,
    ) -> Result<()> {
        let prediction = &mut ctx.accounts.prediction_state;
        let state = &mut ctx.accounts.state;

        if state.admin != *ctx.accounts.user.key {
            return Err(ErrorCode::WrongOwnership.into());
        }

        if prediction.is_settled {
            return Err(ErrorCode::PredictionAlreadySettled.into());
        }

        // Determine if prediction is winning
        prediction.is_winning = match prediction.prediction_type.as_str() {
            "long" => new_price > prediction.start_price,
            "short" => new_price < prediction.start_price,
            _ => return Err(ErrorCode::InvalidPredictionType.into()),
        };
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        if current_timestamp < prediction.expiry_timestamp.try_into().unwrap() {
            return Err(ErrorCode::PredictionNotExpired.into());
        }
        let cpi_accounts = TransferSPL {
            from: ctx.accounts.xyz_vault.to_account_info(),
            to: ctx.accounts.taker_receive_ata.to_account_info(),
            authority: ctx.accounts.auth.to_account_info(),
        };
        let signer_seeds = &[&b"auth"[..], &[ctx.accounts.state.auth_bump]];
        let binding = [&signer_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &binding,
        );
        let amount = prediction.amount + (prediction.amount / 2);
        transfer_spl(cpi_ctx, amount)?;
        prediction.is_settled = true;
        Ok(())
    }
}
#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(
        init,
        payer = user,
        space = 130,
        seeds = [b"binary_options",
        auth.key().as_ref()],
        bump,
    )]
    pub state: Account<'info, BinaryOptionsState>,
    #[account(
        mut,
        seeds = [b"vault",
        state.key().as_ref()],
        token::mint = xyz_mint,
        token::authority = auth,
        bump,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub xyz_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"auth"], bump)]
    /// CHECK: This acc is safe
    pub auth: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct CreatePredictionContext<'info> {
    #[account(init, payer = user, space = 156, seeds = [b"prediction",user.key().as_ref()], bump)]
    pub prediction_state: Account<'info, PredictionState>,
    #[account(seeds = [b"auth"], bump)]
    /// CHECK: This acc is safe
    pub auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub xyz_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = xyz_mint,
        associated_token::authority = user,
    )]
    pub maker_ata: Account<'info, TokenAccount>,
    #[account(seeds = [b"binary_options", auth.key().as_ref()], bump)]
    pub state: Account<'info, BinaryOptionsState>,
    #[account(
        mut,
        seeds = [b"vault",
        state.key().as_ref()],
        token::mint = xyz_mint,
        token::authority = auth,
        bump,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SettlePredictionContext<'info> {
    #[account(seeds = [b"auth"], bump)]
    /// CHECK: This acc is safe
    pub auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub xyz_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account()]
    pub taker_receive_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault",
        state.key().as_ref()],
        token::mint = xyz_mint,
        token::authority = auth,
        bump,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"prediction"], bump)]
    pub prediction_state: Account<'info, PredictionState>,
    #[account(seeds = [b"binary_options", auth.key().as_ref()], bump)]
    pub state: Account<'info, BinaryOptionsState>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[account]
pub struct BinaryOptionsState {
    pub admin: Pubkey,
    pub prediction_counter: u64,
    pub xyz_mint: Pubkey,
    pub xyz_vault: Pubkey,
    pub total_xyz_balance: u64,
    pub fee_percentage: u64,
    pub auth_bump: u8,
    pub vault_bump: u8,
}
#[account]
pub struct PredictionState {
    pub user: Pubkey,
    pub amount: u64,
    pub trader: Pubkey,
    pub token_mint: Pubkey,
    pub start_timestamp: u64,
    pub expiry_timestamp: u64,
    pub start_price: u64,
    pub prediction_type: String,
    pub is_settled: bool,
    pub is_winning: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid time parameter. Must be '30s', '1m', or '5m'.")]
    InvalidTimeParameter,

    #[msg("Invalid prediction type. Must be 'long' or 'short'.")]
    InvalidPredictionType,

    #[msg("Invalid amount format.")]
    InvalidAmountFormat,

    #[msg("Prediction already settled.")]
    PredictionAlreadySettled,

    #[msg("Only Owner Call This Function.")]
    WrongOwnership,

    #[msg("Prediction not expired yet.")]
    PredictionNotExpired,

    #[msg("Prediction not settled yet.")]
    PredictionNotSettled,

    #[msg("Rewards already claimed.")]
    RewardsAlreadyClaimed,

    #[msg("Prediction not winning. No rewards to claim.")]
    PredictionNotWinning,

    #[msg("Insufficient XYZ balance in contract.")]
    InsufficientXYZBalance,

    #[msg("Unauthorized. Only admin can perform this action.")]
    Unauthorized,
}
