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

    #[error("Reached or accessed invalid address in RAM")]
    InvalidRamAddressError,

    #[error("No ROM as been loaded yet")]
    MissingRomError,

    #[error("SDL Error:\n{0}")]
    SdlError(String),

    #[error("Failed to read ROM file")]
    RomFileReadError(#[from] std::io::Error),

    #[error("Audio output failed:\n{0}")]
    AudioOutputError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
