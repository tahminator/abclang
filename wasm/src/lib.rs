use wasm_bindgen::prelude::*;

use interpreter::{
    eval,
    eval::object::{
        Object, Objecter,
        environment::{Env, Environment},
    },
    lexer::Lexer,
    parser::Parser,
};

fn run(input: &str, env: &Env) -> String {
    let lexer = Lexer::new(input);

    let mut parser = match Parser::new(lexer) {
        Ok(parser) => parser,
        Err(err) => return format!("lexer/parser error: {err}"),
    };

    match parser.parse_program() {
        Ok(program) => {
            let result = eval::evaluate(&program, env);

            let mut out = env.borrow().take_output();

            match result {
                Ok(Object::Null(_)) => {}
                Ok(obj) => out.push_str(&obj.inspect_value()),
                Err(err) => out.push_str(&err.inspect_value()),
            }

            out
        }
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
