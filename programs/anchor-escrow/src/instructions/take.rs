use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

#[derive(Accounts)]
pub struct Take<'info> {
    // Signers
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(mut)]
    pub taker: Signer<'info>,

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
