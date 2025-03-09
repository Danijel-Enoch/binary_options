use anchor_lang::prelude::*;
declare_id!("HC2oqz2p6DEWfrahenqdq2moUcga9c9biqRBcdK3XKU1");
#[program]
pub mod binary_options {
    use super::*;
    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        ctx.accounts.state.vote = 0;
        Ok(())
    }
    pub fn claim_rewards(ctx: Context<ClaimRewardsContext>) -> Result<()> {
        Ok(())
    }
    pub fn settle_prediction(ctx: Context<SettlePredictionContext>) -> Result<()> {
        Ok(())
    }
    pub fn create_prediction(ctx: Context<CreatePredictionContext>) -> Result<()> {
        Ok(())
    }
    pub fn withdraw_fees(ctx: Context<WithdrawFeesContext>) -> Result<()> {
        Ok(())
    }
}
#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(init, payer = user, space = 17, seeds = [b"vote"], bump)]
    pub state: Account<'info, VoteState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct ClaimRewardsContext {}
#[derive(Accounts)]
pub struct SettlePredictionContext {}
#[derive(Accounts)]
pub struct CreatePredictionContext {}
#[derive(Accounts)]
pub struct WithdrawFeesContext {}
#[account]
pub struct VoteState {
    pub vote: i64,
    pub bump: u8,
}
