use rustyline::{Editor, Helper, history::History};

use interpreter::{
    eval::{
        evaluate,
        object::{Object, Objecter, environment::Environment},
    },
    lexer::Lexer,
    parser::Parser,
};

use crate::error::ReplError;

static PROMPT: &str = "<< ";
static DEBUG_PRINT_PARSED_PROG_PREFIX: &str = "dprint ";

pub struct Repl {}

impl Repl {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<H: Helper, I: History>(self, rl: &mut Editor<H, I>) -> Result<(), ReplError> {
        let env = Environment::new();

        loop {
            let line = rl.readline(PROMPT)?;

            let (line, is_debug) = match line.strip_prefix(DEBUG_PRINT_PARSED_PROG_PREFIX) {
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
                    } else if let Some(output) = match evaluate(&p, &env) {
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

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
