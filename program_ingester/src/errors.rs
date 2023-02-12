/// This code defines an error enum for the ProgramIngester module, named ProgramIngesterError. The enum has three variants:
/// InvalidProgramInput: This variant is used when the input to the program is not valid, and it carries a string message describing the error.
/// InvalidTimestamp: This variant is used when the timestamp in the input cannot be parsed, and it carries an underlying error of type chrono::ParseError.
/// IoError: This variant is used when an I/O operation fails, and it carries an underlying error of type io::Error.
/// The Error trait and the #[derive(Error, Debug)] attribute are from the thiserror crate,
/// and they allow for convenient error handling and formatting of error messages.

use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProgramIngesterError {
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
