pub mod constant;
pub mod error;
pub mod instructions;
pub mod state;
use anchor_lang::prelude::*;

pub use instructions::{make::*, take::*, withdraw::*};

declare_id!("HNyJQUMLraCF5NegRN8LjSkjZofiazQZdNRgXHX8Pm22");

#[program]
pub mod anchor_escrow {

    use super::*;

    pub fn initialize(
        ctx: Context<Maker>,
        seed: u64,
        amount_req: u64,
        amount_deposited: u64,
    ) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        let bump = ctx.bumps;

        ctx.accounts.initialize_escrow(seed, amount_req, bump)?;
        ctx.accounts.deposite_amount(amount_deposited)?;
        Ok(())
    }

    // No need for the deposite amount and seed in argument bcoz escrow state already knows about them.
    pub fn exchange(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.exchange_amount()?;
        ctx.accounts.escrow_close()?;
        Ok(())
    }

    pub fn refund(ctx: Context<WithdrawAll>) -> Result<()> {
        ctx.accounts.withdraw_close()?;
        Ok(())
    }
}

// +++++++++++++++++ Key Points +++++++++++++++++

// - Accounts can store up to 10MB of data, which can consist of either executable program code or program state.x
