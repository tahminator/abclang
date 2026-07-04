use std::fmt;

use crate::ast::Statement;

pub trait Node {
    fn token_literal(&self) -> &str;
}

pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Program<'a> {
    pub fn token_literal(&self) -> &str {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            ""
        }
    }
}

impl fmt::Display for Program<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{stmt}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Expression, Identifier, LetStatement},
        lexer::token::{Token, TokenType},
    };

    use super::*;

    #[test]
    fn test_to_string() {
        let prog = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    typ: TokenType::Let,
                    literal: "let",
                },
                name: Identifier {
                    token: Token {
                        typ: TokenType::Ident,
                        literal: "myVar",
                    },
                    value: "myVar",
                },
                value: Some(Expression::Identifier(Identifier {
                    token: Token {
                        typ: TokenType::Ident,
                        literal: "anotherVar",
                    },
                    value: "anotherVar",
                })),
            })],
        };

        let s = prog.to_string();
        if s != "let myVar = anotherVar;" {
            panic!("program.to_string() wrong, got {}", s)
        }
    }
}
