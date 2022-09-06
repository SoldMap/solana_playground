use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum CrowdError {
    #[error("Invalid Fee Account")]
    InvalidFeeAccount,

    #[error("Invalid Account")]
    InvalidAuthority,

    #[error("Invalid Signature")]
    InvalidSignature,

    #[error("Already Enabled")]
    AlreadyEnabled,

    #[error("Creator Mismatch")]
    CreatorMismatch,

    #[error("Ivalid Amount")]
    InvalidAmount,
}

impl From<CrowdError> for ProgramError {
    fn from(e: CrowdError) -> Self {
        return ProgramError::Custom(e as u32);
    }
}
