pub use thiserror::Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Program contained an invalid opcode:\n{0}")]
    InvalidOpcodeError(String),

    #[error("Program contained an unknown opcode:\n{0:#X}")]
    UnknownOpcodeError(u16),

    #[error("Invalid ROM size")]
    InvalidRomSizeError,

    #[error("Stack overflow")]
    StackOverflowError,

    #[error("Stack underflow")]
    StackUnderflowError,

    #[error("Reached invalid address in RAM")]
    InvalidRamAddressError,

    #[error("No ROM as been loaded yet")]
    MissingRomError
}

pub type Result<T> = std::result::Result<T, Error>;
