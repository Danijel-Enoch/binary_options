use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

declare_id!("68VQnp2MXC1G6ei7kn1RAveW8rx357T4tRnhC4s4xQoj");

#[program]
pub mod bin_ops {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + BinaryOptionsMainState::INIT_SPACE)]
    pub state: Account<'info, BinaryOptionsMainState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct BinaryOptionsMainState {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub fee_percentage: u8,
}
