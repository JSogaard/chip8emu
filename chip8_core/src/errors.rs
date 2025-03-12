pub use thiserror::Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Program contained an invalid opcode: {0}")]
    InvalidOpcodeError(String),
}

pub type Result<T> = std::result::Result<T, Error>;