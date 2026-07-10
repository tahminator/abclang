use std::io::{stdin, stdout};

mod ast;
mod eval;
mod lexer;
mod object;
mod parser;
mod repl;

fn main() {
    println!("welcome to abclang!");
    println!();
    let mut rl = rustyline::DefaultEditor::new().expect("rustyline failed to be initialized");
    repl::client::Repl::new().start(&mut rl).unwrap()
}
