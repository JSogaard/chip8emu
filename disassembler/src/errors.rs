use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Found unknown opcode: {0:#X}")]
    UnknownOpcodeError(u32),
    #[error("Failed to read ROM file")]
    FileReadError(String),
    #[error("Failed to write to assembly file")]
    FileWriteError(String),
}