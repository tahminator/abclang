use std::io::{BufRead, Write};

use crate::{lexer::Lexer, parser::Parser, repl::error::ReplError};

static PROMPT: &str = "<< ";

pub struct Repl {}

impl Repl {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<R: BufRead, W: Write>(self, reader: R, mut writer: W) -> Result<(), ReplError> {
        let mut lines = reader.lines();
        loop {
            write!(writer, "{PROMPT}")?;
            writer.flush()?;

            if let Some(line) = lines.next().transpose()? {
                let lexer = Lexer::new(&line);
                let mut parser = Parser::new(lexer)?;

                match parser.parse_program() {
                    Ok(s) => writeln!(writer, "{s}")?,
                    Err(errors) => {
                        let errs = errors
                            .iter()
                            .map(|err| format!("\t{}", err))
                            .collect::<Vec<_>>()
                            .join("\n");

                        write!(writer, "parser has {} errors:\n{}", errors.len(), errs)?;
                        continue;
                    }
                }
            }
        }
    }
}
