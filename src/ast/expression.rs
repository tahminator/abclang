use std::fmt;

use crate::{ast::Node, lexer::token::Token};

#[derive(Debug)]
pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    IntegerLiteral(IntegerLiteral<'a>),
    Prefix(Prefix<'a>),
    Infix(Infix<'a>),
    Boolean(Boolean<'a>),
}

impl fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Identifier(expr) => write!(f, "{}", expr.value),
            Expression::IntegerLiteral(expr) => write!(f, "{}", expr.value),
            Expression::Prefix(expr) => write!(f, "({}{})", expr.op, expr.right),
            Expression::Infix(expr) => write!(f, "({} {} {})", expr.left, expr.op, expr.right),
            Expression::Boolean(expr) => write!(f, "{}", expr.value),
        }
    }
}

#[derive(Debug)]
pub struct Identifier<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl<'a> Node for Identifier<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct IntegerLiteral<'a> {
    pub token: Token<'a>,
    pub value: i64,
}

impl<'a> Node for IntegerLiteral<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct Prefix<'a> {
    pub token: Token<'a>,
    pub op: &'a str,
    pub right: Box<Expression<'a>>,
}

impl<'a> Node for Prefix<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct Infix<'a> {
    pub token: Token<'a>,
    pub left: Box<Expression<'a>>,
    pub op: &'a str,
    pub right: Box<Expression<'a>>,
}

impl<'a> Node for Infix<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct Boolean<'a> {
    pub token: Token<'a>,
    pub value: bool,
}

impl<'a> Node for Boolean<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}
