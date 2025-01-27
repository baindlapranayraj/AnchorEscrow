pub mod constant;
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

    pub fn exchange(ctx: Context<Take>, seed: u64, amount: u64) -> Result<()> {
        Ok(())
    }
}

// +++++++++++++++++ Key Points +++++++++++++++++

// - Accounts can store up to 10MB of data, which can consist of either executable program code or program state.x
