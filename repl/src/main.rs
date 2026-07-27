mod error;
mod repl;

use crate::repl::Repl;

fn main() {
    println!("welcome to abclang!");
    println!();
    let mut rl = rustyline::DefaultEditor::new().expect("rustyline failed to be initialized");
    Repl::new().start(&mut rl).unwrap()
}
