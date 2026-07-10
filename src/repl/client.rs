use std::io::{BufRead, Write};

use rustyline::{Editor, Helper, history::History};

use crate::{lexer::Lexer, parser::Parser, repl::error::ReplError};

static PROMPT: &str = "<< ";
static DEBUG_PRINT_PREFIX: &str = "dprint ";

pub struct Repl {}

impl Repl {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<H: Helper, I: History>(self, rl: &mut Editor<H, I>) -> Result<(), ReplError> {
        loop {
            let line = rl.readline(PROMPT)?;

            let (line, is_debug) = match line.strip_prefix(DEBUG_PRINT_PREFIX) {
                Some(s) => (s, true),
                _ => (line.as_str(), false),
            };

            rl.add_history_entry(line)?;

            let lexer = Lexer::new(line);
            let mut parser = Parser::new(lexer)?;

            match parser.parse_program() {
                Ok(p) => {
                    if is_debug {
                        println!("{p:#?}")
                    } else {
                        println!("{p}")
                    }
                }
                Err(errors) => {
                    let errs = errors
                        .iter()
                        .map(|err| format!("\t{}", err))
                        .collect::<Vec<_>>()
                        .join("\n");

                    println!("parser has {} errors:\n{}", errors.len(), errs);
                    continue;
                }
            }
        }
    }
}
