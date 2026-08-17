use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("failed to convert `{0}` to a valid string")]
    FailedToParseToStringError(#[from] Utf8Error),
    #[error("failed to parse character due to: {0}")]
    FailedToParseCharError(String),
    #[error("failed to find digits after period on a float")]
    FailedToFindDigitsAfterPeriodOnFloat,
    #[error("unknown lexer error")]
    Unknown,
}
