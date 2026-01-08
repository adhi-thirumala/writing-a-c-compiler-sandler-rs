use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Lexer error failed: char {char}")]
    LexerError { char: char },

    #[error("Parser error: expected {expected}, found {found}")]
    ParserError { expected: String, found: String },
}
