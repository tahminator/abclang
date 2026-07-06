use std::fmt;

use crate::{
    ast::{Expression, IdentifierExpression, Node},
    lexer::token::Token,
};

#[derive(Debug)]
pub enum Statement<'a> {
    Let(LetStatement<'a>),
    Return(ReturnStatement<'a>),
    Expression(ExpressionStatement<'a>),
    Block(BlockStatement<'a>),
}

impl fmt::Display for Statement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Let(stmt) => {
                write!(
                    f,
                    "{} {} = {};",
                    stmt.token_literal(),
                    stmt.name.value,
                    stmt.value
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "None".to_string())
                )
            }
            Statement::Return(stmt) => {
                write!(
                    f,
                    "{} {}",
                    stmt.token_literal(),
                    stmt.value
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "None".to_string())
                )
            }
            Statement::Expression(stmt) => {
                write!(f, "{}", stmt.expr,)
            }
            Statement::Block(stmt) => {
                write!(
                    f,
                    "{}",
                    stmt.statements
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        }
    }
}

impl<'a> Node for Statement<'a> {
    fn token_literal(&self) -> &str {
        match self {
            Statement::Let(stmt) => stmt.token_literal(),
            Statement::Return(stmt) => stmt.token_literal(),
            Statement::Expression(stmt) => stmt.token_literal(),
            Statement::Block(stmt) => stmt.token_literal(),
        }
    }
}

#[derive(Debug)]
pub struct LetStatement<'a> {
    pub token: Token<'a>,
    pub name: IdentifierExpression<'a>,
    pub value: Option<Expression<'a>>,
}

impl<'a> Node for LetStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct ReturnStatement<'a> {
    pub token: Token<'a>,
    pub value: Option<Expression<'a>>,
}

impl<'a> Node for ReturnStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct ExpressionStatement<'a> {
    pub token: Token<'a>,
    pub expr: Expression<'a>,
}

impl<'a> Node for ExpressionStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct BlockStatement<'a> {
    pub token: Token<'a>,
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Node for BlockStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}
