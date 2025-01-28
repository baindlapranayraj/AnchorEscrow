use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{constant::ESCROW_SEED, state::EscrowState};

#[derive(Accounts)]
pub struct WithdrawAll<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    // mint accounts
    pub mint_a: InterfaceAccount<'info, Mint>,

    // token accounts
    #[account(
        init_if_needed, // Maybe the maker given 100% amount to vault and ATA got closed
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = maker
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // escrow pda
    #[account(
        mut,
        seeds = [ESCROW_SEED,maker.key.as_ref(),escrow_state.seed.to_le_bytes().as_ref()],
        bump = escrow_state.escrow_bump,
        close = maker,
    )]
    pub escrow_state: Account<'info, EscrowState>,

    // vault account
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state
    )]
    pub vault_account: InterfaceAccount<'info, TokenAccount>,

    // program accounts
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> WithdrawAll<'info> {
    pub fn withdraw_all(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            authority: self.escrow_state.to_account_info(),
            from: self.vault_account.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };

        let secure_seed = self.escrow_state.seed.to_le_bytes();

        let seed_bytes: &[&[u8]] = &[
            ESCROW_SEED,
            self.maker.key.as_ref(),
            secure_seed.as_ref(),
            &[self.escrow_state.escrow_bump],
        ];

        let seeds_signer = &[&seed_bytes[..]]; // Now it's safe because seed_bytes is bound

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds_signer);
        token_interface::transfer_checked(
            cpi_context,
            self.vault_account.amount,
            self.mint_a.decimals,
        )?;

        // the vault account will be automatically closed

        Ok(())
    }

    pub fn close_vault(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let close_account = token_interface::CloseAccount {
            account: self.vault_account.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow_state.to_account_info(),
        };

        let seed_bind = self.escrow_state.seed.to_le_bytes();

        let seeds = &[
            ESCROW_SEED,
            self.maker.key.as_ref(),
            seed_bind.as_ref(),
            &[self.escrow_state.escrow_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, close_account, signer_seeds);

        token_interface::close_account(cpi_context)?;

        Ok(())
    }
}

// +++++++++++++++++ Key Points +++++++++++++++++

// - the close macro reduces the lamport balance to zero, sending the lamports to a target address,
// and changes the owner of the account to be the system program.
//
// -
