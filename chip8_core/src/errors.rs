pub use thiserror::Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Program contained an invalid opcode:\n {0}")]
    InvalidOpcodeError(String),

    #[error("Program contained an unknown opcode:\n {0:#X}")]
    UnknownOpcodeError(u16),

    #[error("Invalid ROM size")]
    InvalidRomSizeError,

    #[error("Stack overflow")]
    StackOverflowError,
}

pub type Result<T> = std::result::Result<T, Error>;
