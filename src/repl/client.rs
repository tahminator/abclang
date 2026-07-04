use std::io::{BufRead, Write};

use crate::lexer::{client::Lexer, token::TokenType};

static PROMPT: &str = "<< ";

pub struct Repl {}

impl Repl {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<R: BufRead, W: Write>(self, reader: R, mut writer: W) -> std::io::Result<()> {
        let mut lines = reader.lines();
        loop {
            write!(writer, "{PROMPT}")?;
            writer.flush()?;

            let line = match lines.next() {
                Some(Ok(line)) => line,
                _ => return Ok(()),
            };

            let mut lexer = Lexer::new(&line);
            loop {
                match lexer.next_token() {
                    Ok(tok) if tok.typ == TokenType::Eof => break,
                    Ok(tok) => writeln!(writer, "{tok:?}")?,
                    Err(e) => {
                        writeln!(writer, "error: {e}")?;
                        break;
                    }
                }
            }
        }
    }
}
