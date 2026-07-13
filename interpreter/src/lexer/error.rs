use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("failed to convert `{0}` to a valid string")]
    FailedToParseToStringError(#[from] Utf8Error),
    #[error("unknown lexer error")]
    Unknown,
}
