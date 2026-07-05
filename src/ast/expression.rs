use std::fmt::{self, write};

use crate::{
    ast::{BlockStatement, Node, Statement},
    lexer::token::Token,
};

#[derive(Debug)]
pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    IntegerLiteral(IntegerLiteral<'a>),
    Prefix(Prefix<'a>),
    Infix(Infix<'a>),
    Boolean(Boolean<'a>),
    If(If<'a>),
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

#[derive(Debug)]
pub struct If<'a> {
    pub token: Token<'a>,
    pub cond: Box<Expression<'a>>,
    pub consequence: Option<BlockStatement<'a>>,
    pub alternative: Option<BlockStatement<'a>>,
}

impl<'a> Node for If<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}
