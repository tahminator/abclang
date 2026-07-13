mod client;
mod error;

fn main() {
    println!("welcome to abclang!");
    println!();
    let mut rl = rustyline::DefaultEditor::new().expect("rustyline failed to be initialized");
    client::Repl::new().start(&mut rl).unwrap()
}
