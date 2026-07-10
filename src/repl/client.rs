use std::io::{BufRead, Write};

use rustyline::{Editor, Helper, history::History};

use crate::{
    eval::evaluate,
    lexer::Lexer,
    object::{Object, Objecter, environment::Environment},
    parser::Parser,
    repl::error::ReplError,
};

static PROMPT: &str = "<< ";
static DEBUG_PRINT_PARSED_PROG_PREFIX: &str = "dprint ";

pub struct Repl {}

impl Repl {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<H: Helper, I: History>(self, rl: &mut Editor<H, I>) -> Result<(), ReplError> {
        let mut env = Environment::default();

        loop {
            let line = rl.readline(PROMPT)?;
            // TODO: fix hacl
            let line: &'static str = Box::leak(line.into_boxed_str());

            let (line, is_debug) = match line.strip_prefix(DEBUG_PRINT_PARSED_PROG_PREFIX) {
                Some(s) => (s, true),
                _ => (line, false),
            };

            rl.add_history_entry(line)?;

            let lexer = Lexer::new(line);
            let mut parser = Parser::new(lexer)?;

            match parser.parse_program() {
                Ok(p) => {
                    if is_debug {
                        println!("{p:#?}")
                    } else if let Some(output) = match evaluate(&p, &mut env) {
                        Ok(o) if !matches!(o, Object::NULL) => Some(o.inspect_value()),
                        Err(o) => Some(o.inspect_value()),
                        _ => None,
                    } {
                        println!("{output}")
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
