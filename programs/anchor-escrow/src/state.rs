use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct EscrowState {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount_require: u64,
    pub escrow_bump: u8,
}
// size:- 8 + 32 + 32 + 32 + 8 + 8;

// - No need to store about the amount maker as deposited,becoz it is already stored inside of
//  escrow vault.
