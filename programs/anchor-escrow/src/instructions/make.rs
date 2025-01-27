use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    constant::{ANCHOR_DISCRIMINATOR, ESCROW_SEED},
    state::EscrowState,
};

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Maker<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint_a,
        token::authority = maker,
        // token::token_program = token_program, // 2)
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + EscrowState::INIT_SPACE,
        seeds = [
          ESCROW_SEED,
          maker.key.as_ref(),
          seed.to_le_bytes().as_ref()
         ],
        bump
    )]
    pub escrow_state: Account<'info, EscrowState>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state, // 1)
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // All Program Accounts Required
    pub token_program: Interface<'info, TokenInterface>, // 3)
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Maker<'info> {
    // Initializing the Escrow state
    pub fn initialize_escrow(
        &mut self,
        seed: u64,
        amount_req: u64,
        bumps: MakerBumps,
    ) -> Result<()> {
        self.escrow_state.set_inner(EscrowState {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            amount_require: amount_req,
            escrow_bump: bumps.escrow_state,
        });
        Ok(())
    }

    pub fn deposite_amount(&mut self, deposite_amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_ctx, deposite_amount, self.mint_a.decimals)?;

        Ok(())
    }
}

// ++++++++++++++++++++++++++ Key Learnigs ++++++++++++++++++++++++++
// 1) The reason the vault account is given authority to escrow_state instead of the maker is to ensure
//   that the escrow process is controlled and managed by the escrow contract.
//
// 2) No need assign token program,anchor already knows about it.
//
// 3) Interface is for any Token Program can included hear it could be either token program or Token2022 Program.
