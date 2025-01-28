use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        self, transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::{constant::ESCROW_SEED, error::EscrowError, state::EscrowState};

#[derive(Accounts)]
pub struct Take<'info> {
    // Signers
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account(mut)]
    pub taker: Signer<'info>,

    //mint accounts
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    //token accounts
    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // No init_if_needed becoz this account have to exist in order to exchnage amount
    #[account(
        mut,
        constraint = taker_ata_b.amount >= escrow_state.amount_require @ EscrowError::EscrowInSufficientAmount, // condition inside of acc.
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    // Vault Account
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state,
    )]
    pub vault_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // escrow state
    #[account(
        mut,
        seeds = [ESCROW_SEED,maker.key.as_ref(),escrow_state.seed.to_le_bytes().as_ref()],
        bump = escrow_state.escrow_bump,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Take<'info> {
    pub fn exchange_amount(&mut self) -> Result<()> {
        // Transfer the tokens from vault to taker_ata_a && taker_ata_b to maker_ata_b

        let cpi_program = self.token_program.to_account_info();

        // vault to taker
        let cpi_accounts = TransferChecked {
            authority: self.escrow_state.to_account_info(),
            from: self.vault_account.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };

        let secure_seed = self.escrow_state.seed.to_le_bytes();

        let seeds = &[
            ESCROW_SEED,
            self.maker.key.as_ref(),
            secure_seed.as_ref(),
            &[self.escrow_state.escrow_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        transfer_checked(
            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer_seeds),
            self.vault_account.amount,
            self.mint_a.decimals,
        )?;

        // taker to maker
        let cpi_accounts = TransferChecked {
            authority: self.taker.to_account_info(),
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
        };

        transfer_checked(
            CpiContext::new(cpi_program, cpi_accounts),
            self.escrow_state.amount_require,
            self.mint_b.decimals,
        )?;

        Ok(())
    }

    pub fn escrow_close(&mut self) -> Result<()> {
        // close the escrow account and vault account
        let cpi_program = self.token_program.to_account_info();
        let vault_close_accounts = token_interface::CloseAccount {
            authority: self.escrow_state.to_account_info(),
            account: self.vault_account.to_account_info(),
            destination: self.maker.to_account_info(),
        };

        let seed_bind = self.escrow_state.seed.to_le_bytes();

        let seeds = &[
            ESCROW_SEED,
            self.maker.key.as_ref(),
            seed_bind.as_ref(),
            &[self.escrow_state.escrow_bump],
        ];

        let signr_seeds = &[&seeds[..]];

        let cpi_context =
            CpiContext::new_with_signer(cpi_program, vault_close_accounts, signr_seeds);

        // Closing Vault Account
        token_interface::close_account(cpi_context)?;

        // TODO:- Close Escrow Account
        //

        Ok(())
    }
}

// +++++++++++++++++ Key Points +++++++++++++++++
// - has_one :- It ensures that a specific field in an account points to another account,
//  and it automatically checks that the referenced account exists and matches
//  the expected public key.
//
// -
