use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::str::FromStr;

declare_id!("3HJ4KCfRRxwcAeqciBNMb6vj3Q788xKBUMiEge64oJ4K");

#[program]
pub mod binary_options {
    use super::*;

    // Initialize the contract with XYZ tokens
    pub fn initialize(ctx: Context<Initialize>, xyz_amount: u64) -> Result<()> {
        let contract = &mut ctx.accounts.contract;
        contract.admin = ctx.accounts.admin.key();
        contract.xyz_vault = ctx.accounts.xyz_vault.key();
        contract.xyz_mint = ctx.accounts.xyz_mint.key();
        contract.total_xyz_balance = xyz_amount;
        contract.fee_percentage = 10; // 10% fee

        // Transfer initial XYZ tokens to the vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.admin_xyz_account.to_account_info(),
            to: ctx.accounts.xyz_vault.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, xyz_amount)?;

        Ok(())
    }

    // Create a prediction
    pub fn create_prediction(
        ctx: Context<CreatePrediction>,
        amount: String,
        token: String,
        time: String,
        current_price: f64,
        prediction_type: String,
    ) -> Result<()> {
        let contract = &ctx.accounts.contract;
        let user = &ctx.accounts.user;
        let prediction = &mut ctx.accounts.prediction;

        // Validate time parameter
        if time != "30s" && time != "1m" && time != "5m" {
            return Err(ErrorCode::InvalidTimeParameter.into());
        }

        // Validate prediction type
        if prediction_type != "long" && prediction_type != "short" {
            return Err(ErrorCode::InvalidPredictionType.into());
        }

        // Parse amount to u64
        let amount_value = match amount.parse::<u64>() {
            Ok(value) => value,
            Err(_) => return Err(ErrorCode::InvalidAmountFormat.into()),
        };

        // Calculate expiry timestamp based on time parameter
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        let expiry_timestamp = match time.as_str() {
            "30s" => current_timestamp + 30,
            "1m" => current_timestamp + 60,
            "5m" => current_timestamp + 300,
            _ => return Err(ErrorCode::InvalidTimeParameter.into()),
        };

        // Transfer tokens from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount_value)?;

        // Set prediction details
        prediction.user = user.key();
        prediction.amount = amount_value;
        prediction.token = token;
        prediction.token_mint = ctx.accounts.token_mint.key();
        prediction.start_timestamp = current_timestamp;
        prediction.expiry_timestamp = expiry_timestamp;
        prediction.start_price = current_price;
        prediction.prediction_type = prediction_type;
        prediction.is_settled = false;
        prediction.is_claimed = false;
        prediction.is_winning = false;

        Ok(())
    }

    // Settle prediction result
    pub fn settle_prediction(ctx: Context<SettlePrediction>, end_price: f64) -> Result<()> {
        let prediction = &mut ctx.accounts.prediction;

        // Ensure prediction hasn't been settled yet
        if prediction.is_settled {
            return Err(ErrorCode::PredictionAlreadySettled.into());
        }

        // Ensure prediction has expired
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        if current_timestamp < prediction.expiry_timestamp {
            return Err(ErrorCode::PredictionNotExpired.into());
        }

        // Determine if prediction is winning
        prediction.is_winning = match prediction.prediction_type.as_str() {
            "long" => end_price > prediction.start_price,
            "short" => end_price < prediction.start_price,
            _ => return Err(ErrorCode::InvalidPredictionType.into()),
        };

        prediction.end_price = end_price;
        prediction.is_settled = true;

        Ok(())
    }

    // Claim rewards for a winning prediction
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let contract = &ctx.accounts.contract;
        let prediction = &mut ctx.accounts.prediction;

        // Validation checks
        if !prediction.is_settled {
            return Err(ErrorCode::PredictionNotSettled.into());
        }

        if prediction.is_claimed {
            return Err(ErrorCode::RewardsAlreadyClaimed.into());
        }

        if !prediction.is_winning {
            return Err(ErrorCode::PredictionNotWinning.into());
        }

        // Calculate reward amount (2x investment minus fee)
        let reward_amount = prediction.amount * 2;
        let fee_amount = reward_amount * contract.fee_percentage as u64 / 100;
        let payout_amount = reward_amount - fee_amount;

        // Return original token amount back to user
        let token_cpi_accounts = Transfer {
            from: ctx.accounts.token_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.contract.to_account_info(),
        };

        let token_cpi_program = ctx.accounts.token_program.to_account_info();
        let token_cpi_ctx = CpiContext::new_with_signer(
            token_cpi_program,
            token_cpi_accounts,
            &[&[b"contract", &[*ctx.bumps.get("contract").unwrap()]]],
        );

        token::transfer(token_cpi_ctx, prediction.amount)?;

        // Transfer XYZ tokens as profit
        let xyz_cpi_accounts = Transfer {
            from: ctx.accounts.xyz_vault.to_account_info(),
            to: ctx.accounts.user_xyz_account.to_account_info(),
            authority: ctx.accounts.contract.to_account_info(),
        };

        let xyz_cpi_program = ctx.accounts.token_program.to_account_info();
        let xyz_cpi_ctx = CpiContext::new_with_signer(
            xyz_cpi_program,
            xyz_cpi_accounts,
            &[&[b"contract", &[*ctx.bumps.get("contract").unwrap()]]],
        );

        token::transfer(xyz_cpi_ctx, payout_amount)?;

        // Update contract state
        let contract = &mut ctx.accounts.contract;
        contract.total_xyz_balance = contract
            .total_xyz_balance
            .checked_sub(payout_amount)
            .ok_or(ErrorCode::InsufficientXYZBalance)?;

        // Mark prediction as claimed
        prediction.is_claimed = true;

        Ok(())
    }

    // Allow admin to withdraw fees
    pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: u64) -> Result<()> {
        let contract = &mut ctx.accounts.contract;

        // Ensure caller is admin
        if contract.admin != ctx.accounts.admin.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        // Ensure amount is valid
        if amount > contract.total_xyz_balance {
            return Err(ErrorCode::InsufficientXYZBalance.into());
        }

        // Transfer XYZ tokens
        let cpi_accounts = Transfer {
            from: ctx.accounts.xyz_vault.to_account_info(),
            to: ctx.accounts.admin_xyz_account.to_account_info(),
            authority: ctx.accounts.contract.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            &[&[b"contract", &[*ctx.bumps]]],
        );

        token::transfer(cpi_ctx, amount)?;

        // Update contract state
        contract.total_xyz_balance = contract
            .total_xyz_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::InsufficientXYZBalance)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + Contract::SIZE, seeds = [b"contract"], bump)]
    pub contract: Account<'info, Contract>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub admin_xyz_account: Account<'info, TokenAccount>,

    pub xyz_mint: Account<'info, token::Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [b"xyz_vault"],
        bump,
        token::mint = xyz_mint,
        token::authority = contract,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreatePrediction<'info> {
    #[account(mut, seeds = [b"contract"], bump)]
    pub contract: Account<'info, Contract>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + Prediction::SIZE,
        seeds = [b"prediction", user.key().as_ref(), &Clock::get().unwrap().unix_timestamp.to_le_bytes()],
        bump
    )]
    pub prediction: Account<'info, Prediction>,

    pub token_mint: Account<'info, token::Mint>,

    #[account(mut, constraint = user_token_account.owner == user.key() && user_token_account.mint == token_mint.key())]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_vault", token_mint.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = contract,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettlePrediction<'info> {
    #[account(mut, seeds = [b"contract"], bump)]
    pub contract: Account<'info, Contract>,

    #[account(
        mut,
        seeds = [b"prediction", prediction.user.as_ref(), &prediction.start_timestamp.to_le_bytes()],
        bump,
        constraint = !prediction.is_settled
    )]
    pub prediction: Account<'info, Prediction>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut, seeds = [b"contract"], bump)]
    pub contract: Account<'info, Contract>,

    #[account(
        mut,
        seeds = [b"prediction", user.key().as_ref(), &prediction.start_timestamp.to_le_bytes()],
        bump,
        constraint = prediction.user == user.key() && prediction.is_settled && !prediction.is_claimed && prediction.is_winning
    )]
    pub prediction: Account<'info, Prediction>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() && user_token_account.mint == prediction.token_mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_vault", prediction.token_mint.as_ref()],
        bump,
        token::mint = prediction.token_mint,
        token::authority = contract,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_xyz_account.owner == user.key() && user_xyz_account.mint == contract.xyz_mint
    )]
    pub user_xyz_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"xyz_vault"],
        bump,
        token::mint = contract.xyz_mint,
        token::authority = contract,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut, seeds = [b"contract"], bump)]
    pub contract: Account<'info, Contract>,

    #[account(mut, constraint = admin.key() == contract.admin)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        constraint = admin_xyz_account.owner == admin.key() && admin_xyz_account.mint == contract.xyz_mint
    )]
    pub admin_xyz_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"xyz_vault"],
        bump,
        token::mint = contract.xyz_mint,
        token::authority = contract,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Contract {
    pub admin: Pubkey,
    pub xyz_mint: Pubkey,
    pub xyz_vault: Pubkey,
    pub total_xyz_balance: u64,
    pub fee_percentage: u8,
}

impl Contract {
    pub const SIZE: usize = 32 + // admin pubkey
                            32 + // xyz_mint pubkey
                            32 + // xyz_vault pubkey
                            8 +  // total_xyz_balance
                            1; // fee_percentage
}

#[account]
pub struct Prediction {
    pub user: Pubkey,
    pub amount: u64,
    pub token: String,
    pub token_mint: Pubkey,
    pub start_timestamp: i64,
    pub expiry_timestamp: i64,
    pub start_price: f64,
    pub end_price: f64,
    pub prediction_type: String,
    pub is_settled: bool,
    pub is_claimed: bool,
    pub is_winning: bool,
}

impl Prediction {
    pub const SIZE: usize = 32 + // user pubkey
                            8 +  // amount
                            36 + // token string (max 32 chars + 4 bytes for length)
                            32 + // token_mint pubkey
                            8 +  // start_timestamp
                            8 +  // expiry_timestamp
                            8 +  // start_price
                            8 +  // end_price
                            10 + // prediction_type string (max 6 chars + 4 bytes for length)
                            1 +  // is_settled
                            1 +  // is_claimed
                            1; // is_winning
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
