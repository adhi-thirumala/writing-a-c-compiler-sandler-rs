use std::io;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Lexer error failed: char {char}")]
    LexerError { char: char },

    #[error("Parser error: expected {expected}, found {found}")]
    ParserError { expected: String, found: String },

    #[error("Assmebly generation failed: {0}")]
    AsmGenError(String),

    #[error("Code emission failed: {0}")]
    CodeEmissionError(String),

    #[error("IO error: {0}")]
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}
