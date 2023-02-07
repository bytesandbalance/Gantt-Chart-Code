use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProgramIngestorError {
    #[error("The program input is not valid: {0}")]
    InvalidProgramInput(String),

    #[error("The timestamp could not be parsed")]
    InvalidTimestamp {
        #[from]
        source: chrono::ParseError,
    },

    #[error("The IO operation failed: {source}")]
    IoError {
        #[from]
        source: io::Error,
    },
}
