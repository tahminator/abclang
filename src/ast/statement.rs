use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
    ast::{Expression, IdentifierExpression, Node},
    lexer::token::Token,
};

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Let(LetStatement<'a>),
    Return(ReturnStatement<'a>),
    Expression(ExpressionStatement<'a>),
    Block(BlockStatement<'a>),
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut w = |x: &dyn Display| write!(f, "{x}");

        match self {
            Statement::Let(stmt) => w(stmt),
            Statement::Return(stmt) => w(stmt),
            Statement::Expression(stmt) => w(stmt),
            Statement::Block(stmt) => w(stmt),
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

#[derive(Debug, Clone)]
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

impl Display for LetStatement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {} = {};",
            self.token_literal(),
            self.name.value,
            self.value
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement<'a> {
    pub token: Token<'a>,
    pub value: Option<Expression<'a>>,
}

impl<'a> Node for ReturnStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for ReturnStatement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}",
            self.token_literal(),
            self.value
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement<'a> {
    pub token: Token<'a>,
    pub expr: Expression<'a>,
}

impl<'a> Node for ExpressionStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for ExpressionStatement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.expr)
    }
}

#[derive(Debug, Clone)]
pub struct BlockStatement<'a> {
    pub token: Token<'a>,
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Node for BlockStatement<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for BlockStatement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            self.statements
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
