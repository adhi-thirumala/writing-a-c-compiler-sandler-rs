use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Lexer error failed: char {char}")]
    LexerError { char: char },
}
