use std::{fmt, rc::Rc};

use crate::ast::Statement;

pub trait Node {
    fn token_literal(&self) -> Rc<str>;
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Node for Program {
    fn token_literal(&self) -> Rc<str> {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            "".into()
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{stmt}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Expression, IdentifierExpression, LetStatement},
        lexer::token::{Token, TokenType},
    };

    use super::*;

    #[test]
    fn test_to_string() {
        let prog = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    typ: TokenType::Let,
                    literal: "let".into(),
                }
                .into(),
                name: IdentifierExpression {
                    token: Token {
                        typ: TokenType::Ident,
                        literal: "myVar".into(),
                    }
                    .into(),
                    value: "myVar".into(),
                },
                value: Some(Expression::Identifier(IdentifierExpression {
                    token: Token {
                        typ: TokenType::Ident,
                        literal: "anotherVar".into(),
                    }
                    .into(),
                    value: "anotherVar".into(),
                })),
            })],
        };

        let s = prog.to_string();
        if s != "let myVar = anotherVar;" {
            panic!("program.to_string() wrong, got {}", s)
        }
    }
}
