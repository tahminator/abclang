use std::io::{stdin, stdout};

mod ast;
mod lexer;
mod parser;
mod repl;

fn main() {
    println!("welcome to abclang!");
    println!();
    repl::client::Repl::new()
        .start(stdin().lock(), stdout().lock())
        .unwrap()
}
