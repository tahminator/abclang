use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;

use crate::lexer::{self, token::TokenType};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("lexer error encountered: {0}")]
    LexerError(#[from] lexer::error::LexerError),
    #[error("expected next token to be {expected}, got {got} instead")]
    UnexpectedToken { expected: TokenType, got: TokenType },
    #[error("no prefix parse function for {typ} found")]
    NoPrefixParseFnFound { typ: TokenType },
    #[error("invalid assignment target: {target}, expected an identifier or index expression")]
    InvalidAssignmentTarget { target: String },
    #[error("failed to parse int to string")]
    FailedToParseIntToStringError(#[from] ParseIntError),
    #[error("failed to parse float to string")]
    FailedToParseFloatToStringError(#[from] ParseFloatError),
}
