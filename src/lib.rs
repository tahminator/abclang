pub mod ast;
pub mod eval;
pub mod lexer;
pub mod object;
pub mod parser;

#[cfg(not(target_arch = "wasm32"))]
pub mod repl;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
