use thiserror::Error;

#[derive(Error)]
pub enum Error {
    #[error("Found unknown opcode: {0}")]
    UnknownOpcodeError(u16),
    #[error("Failed to read ROM file")]
    FileReadError,
    #[error("Failed to write to assembly file")]
    FileWriteError,
}