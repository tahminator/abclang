use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
    ast::{BlockStatement, Node},
    lexer::token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    Identifier(IdentifierExpression<'a>),
    IntegerLiteral(IntegerLiteralExpression<'a>),
    Prefix(PrefixExpression<'a>),
    Infix(InfixExpression<'a>),
    Boolean(BooleanExpression<'a>),
    If(IfExpression<'a>),
    FnLiteral(FnLiteralExpression<'a>),
    Call(CallExpression<'a>),
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut w = |x: &dyn Display| write!(f, "{x}");

        match self {
            Expression::Identifier(expr) => w(expr),
            Expression::IntegerLiteral(expr) => w(expr),
            Expression::Prefix(expr) => w(expr),
            Expression::Infix(expr) => w(expr),
            Expression::Boolean(expr) => w(expr),
            Expression::If(expr) => w(expr),
            Expression::FnLiteral(expr) => w(expr),
            Expression::Call(expr) => w(expr),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierExpression<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl<'a> Node for IdentifierExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for IdentifierExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerLiteralExpression<'a> {
    pub token: Token<'a>,
    pub value: i64,
}

impl<'a> Node for IntegerLiteralExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for IntegerLiteralExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl Display for PrefixExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}{})", self.op, self.right)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl Display for InfixExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanExpression<'a> {
    pub token: Token<'a>,
    pub value: bool,
}

impl<'a> Node for BooleanExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for BooleanExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl Display for IfExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let stringify = |block: &BlockStatement| {
            block
                .statements
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };

        let consequence = self
            .consequence
            .as_ref()
            .map(stringify)
            .unwrap_or_else(|| "None".to_string());

        match &self.alternative {
            Some(alt) => write!(
                f,
                "if {} {} else {}",
                self.cond,
                consequence,
                stringify(alt)
            ),
            None => write!(f, "if {} {}", self.cond, consequence,),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnLiteralExpression<'a> {
    pub token: Token<'a>,
    pub params: Vec<IdentifierExpression<'a>>,
    pub body: Option<BlockStatement<'a>>,
}

impl<'a> Node for FnLiteralExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for FnLiteralExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}({}) {}",
            self.token_literal(),
            self.params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.body
                .as_ref()
                .map(|b| b.to_string())
                .unwrap_or_else(|| "None".to_string())
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression<'a> {
    pub token: Token<'a>,
    /**
     * Identifier or FunctionLiteral
     */
    pub function: Box<Expression<'a>>,
    pub args: Vec<Expression<'a>>,
}

impl<'a> Node for CallExpression<'a> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }
}

impl Display for CallExpression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}({})",
            self.function,
            self.args
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}
