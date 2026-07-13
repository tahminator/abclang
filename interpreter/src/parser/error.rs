use std::num::ParseIntError;

use thiserror::Error;

use crate::lexer::{self, token::TokenType};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("lexer error encountered")]
    LexerError(#[from] lexer::error::LexerError),
    #[error("expected next token to be {expected}, got {got} instead")]
    UnexpectedToken { expected: TokenType, got: TokenType },
    #[error("no prefix parse function for {typ} found")]
    NoPrefixParseFnFound { typ: TokenType },
    #[error("failed to parse int to string")]
    FailedToParseIntToStringError(#[from] ParseIntError),
}
