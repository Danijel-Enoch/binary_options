use anchor_lang::prelude::*;
use anchor_spl::{
    token::{
        Transfer as TransferSPL, Token, TokenAccount, Mint, transfer as transfer_spl,
    },
    associated_token::AssociatedToken,
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
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );
        transfer_spl(cpi_ctx, amount)?;
        ctx.accounts.prediction_state.amount = amount;
        ctx.accounts.prediction_state.token_mint = token_mint;
        ctx.accounts.prediction_state.start_timestamp = start_timestamp;
        ctx.accounts.prediction_state.expiry_timestamp = expiry_timestamp;
        ctx.accounts.prediction_state.start_price = start_price;
        ctx.accounts.prediction_state.end_price = end_price;
        ctx.accounts.prediction_state.prediction_type = prediction_type;
        ctx.accounts.prediction_state.trader = ctx.accounts.maker_ata.key();
        Ok(())
    }
    pub fn settle_prediction(
        ctx: Context<SettlePredictionContext>,
        is_winning: bool,
        id: u64,
        taker: Pubkey,
    ) -> Result<()> {
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
    #[account(seeds = [b"auth"], bump)]
    /// CHECK: This acc is safe
    pub auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub xyz_mint: Account<'info, Mint>,
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct CreatePredictionContext<'info> {
    #[account(init, payer = user, space = 156, seeds = [b"prediction"], bump)]
    pub prediction_state: Account<'info, PredictionState>,
    #[account(mut)]
    pub xyz_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"auth"], bump)]
    /// CHECK: This acc is safe
    pub auth: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = xyz_mint,
        associated_token::authority = user,
    )]
    pub maker_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault",
        state.key().as_ref()],
        token::mint = xyz_mint,
        token::authority = auth,
        bump,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,
    #[account(seeds = [b"binary_options", auth.key().as_ref()], bump)]
    pub state: Account<'info, BinaryOptionsState>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SettlePredictionContext<'info> {
    #[account(mut, seeds = [b"prediction"], bump)]
    pub prediction_state: Account<'info, PredictionState>,
    #[account(
        mut,
        seeds = [b"vault",
        state.key().as_ref()],
        token::mint = xyz_mint,
        token::authority = auth,
        bump,
    )]
    pub xyz_vault: Account<'info, TokenAccount>,
    #[account(seeds = [b"auth"], bump)]
    /// CHECK: This acc is safe
    pub auth: UncheckedAccount<'info>,
    #[account(seeds = [b"binary_options", auth.key().as_ref()], bump)]
    pub state: Account<'info, BinaryOptionsState>,
    #[account()]
    pub taker_receive_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub xyz_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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
    pub end_price: u64,
    pub prediction_type: String,
    pub is_settled: bool,
    pub is_winning: bool,
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
