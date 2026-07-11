use abclang::repl;

fn main() {
    println!("welcome to abclang!");
    println!();
    let mut rl = rustyline::DefaultEditor::new().expect("rustyline failed to be initialized");
    repl::client::Repl::new().start(&mut rl).unwrap()
}
