use std::fmt::{self, write};

use crate::{
    ast::{BlockStatement, Node, Statement},
    lexer::token::Token,
};

#[derive(Debug)]
pub enum Expression<'a> {
    Identifier(IdentifierExpression<'a>),
    IntegerLiteral(IntegerLiteralExpression<'a>),
    Prefix(PrefixExpression<'a>),
    Infix(InfixExpression<'a>),
    Boolean(BooleanExpression<'a>),
    If(IfExpression<'a>),
}

impl fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Identifier(expr) => write!(f, "{}", expr.value),
            Expression::IntegerLiteral(expr) => write!(f, "{}", expr.value),
            Expression::Prefix(expr) => write!(f, "({}{})", expr.op, expr.right),
            Expression::Infix(expr) => write!(f, "({} {} {})", expr.left, expr.op, expr.right),
            Expression::Boolean(expr) => write!(f, "{}", expr.value),
            Expression::If(expr) => {
                let stringify = |block: &BlockStatement| {
                    block
                        .statements
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let consequence = expr
                    .consequence
                    .as_ref()
                    .map(stringify)
                    .unwrap_or_else(|| "None".to_string());

                match &expr.alternative {
                    Some(alt) => write!(
                        f,
                        "if {} {} else {}",
                        expr.cond,
                        consequence,
                        stringify(alt)
                    ),
                    None => write!(f, "if {} {}", expr.cond, consequence,),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct IdentifierExpression<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl<'a> Node for IdentifierExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct IntegerLiteralExpression<'a> {
    pub token: Token<'a>,
    pub value: i64,
}

impl<'a> Node for IntegerLiteralExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct PrefixExpression<'a> {
    pub token: Token<'a>,
    pub op: &'a str,
    pub right: Box<Expression<'a>>,
}

impl<'a> Node for PrefixExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct InfixExpression<'a> {
    pub token: Token<'a>,
    pub left: Box<Expression<'a>>,
    pub op: &'a str,
    pub right: Box<Expression<'a>>,
}

impl<'a> Node for InfixExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct BooleanExpression<'a> {
    pub token: Token<'a>,
    pub value: bool,
}

impl<'a> Node for BooleanExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

#[derive(Debug)]
pub struct IfExpression<'a> {
    pub token: Token<'a>,
    pub cond: Box<Expression<'a>>,
    pub consequence: Option<BlockStatement<'a>>,
    pub alternative: Option<BlockStatement<'a>>,
}

impl<'a> Node for IfExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}
