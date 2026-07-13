use rustyline::error::ReadlineError;
use thiserror::Error;

use interpreter::{eval::object::ErrorObject, parser::error::ParserError};

#[derive(Debug, Error)]
pub enum ReplError {
    #[error("i/o error")]
    Io(#[from] std::io::Error),
    #[error("parser error")]
    Parser(#[from] ParserError),
    #[error("readline error")]
    Readline(#[from] ReadlineError),
    #[error("evaluate error: {err}")]
    Evaluate { err: ErrorObject },
}
