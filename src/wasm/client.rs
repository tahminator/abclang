use wasm_bindgen::prelude::*;

use crate::{
    eval,
    lexer::Lexer,
    object::{
        Object, Objecter,
        environment::{Env, Environment},
    },
    parser::Parser,
};

fn run(input: &str, env: &Env) -> String {
    let lexer = Lexer::new(input);

    let mut parser = match Parser::new(lexer) {
        Ok(parser) => parser,
        Err(err) => return format!("lexer/parser error: {err}"),
    };

    match parser.parse_program() {
        Ok(program) => match eval::evaluate(&program, env) {
            Ok(Object::Null(_)) => String::new(),
            Ok(obj) => obj.inspect_value(),
            Err(err) => err.inspect_value(),
        },
        Err(errors) => {
            let errs = errors
                .iter()
                .map(|err| format!("\t{err}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("parser has {} error(s):\n{}", errors.len(), errs)
        }
    }
}

#[wasm_bindgen]
pub struct Interpreter {
    env: Env,
}

#[wasm_bindgen]
impl Interpreter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn evaluate(&self, input: String) -> String {
        run(&input, &self.env)
    }

    pub fn reset(&mut self) {
        self.env = Environment::new();
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
