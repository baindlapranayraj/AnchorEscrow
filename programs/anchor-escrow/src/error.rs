use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Insufficient funds in this account")]
    EscrowInSufficientAmount,
}
