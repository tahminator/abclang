use rustyline::error::ReadlineError;
use thiserror::Error;

use crate::parser::error::ParserError;

#[derive(Debug, Error)]
pub enum ReplError {
    #[error("i/o error")]
    IoError(#[from] std::io::Error),
    #[error("parser error")]
    ParserError(#[from] ParserError),
    #[error("readline error")]
    ReadlineError(#[from] ReadlineError),
}
