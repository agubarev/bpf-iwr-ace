use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum IWRError {
    /// Incorrect authority provided on update or delete
    #[error("incorrect authority")]
    IncorrectAuthority,

    #[error("not enough lamports")]
    NotEnoughLamports,

    #[error("not enough tokens")]
    NotEnoughTokens,

    /// Calculation overflow
    #[error("Calculation overflow")]
    Overflow,
}

impl From<IWRError> for ProgramError {
    fn from(e: IWRError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for IWRError {
    fn type_of() -> &'static str {
        "IWR Error"
    }
}
