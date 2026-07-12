use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    rc::Rc,
};

use crate::{
    ast::{BlockStatement, Node},
    lexer::token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(IdentifierExpression),
    IntegerLiteral(IntegerLiteralExpression),
    Prefix(PrefixExpression),
    Infix(InfixExpression),
    Boolean(BooleanExpression),
    If(IfExpression),
    FnLiteral(FnLiteralExpression),
    Call(CallExpression),
    String(StringExpression),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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
            Expression::String(expr) => w(expr),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierExpression {
    pub token: Rc<Token>,
    pub value: Rc<str>,
}

impl Node for IdentifierExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for IdentifierExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerLiteralExpression {
    pub token: Rc<Token>,
    pub value: i64,
}

impl Node for IntegerLiteralExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for IntegerLiteralExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixExpression {
    pub token: Rc<Token>,
    pub op: Rc<str>,
    pub right: Rc<Expression>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for PrefixExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}{})", self.op, self.right)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfixExpression {
    pub token: Rc<Token>,
    pub left: Rc<Expression>,
    pub op: Rc<str>,
    pub right: Rc<Expression>,
}

impl<'a> Node for InfixExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for InfixExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanExpression {
    pub token: Rc<Token>,
    pub value: bool,
}

impl Node for BooleanExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for BooleanExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpression {
    pub token: Rc<Token>,
    pub cond: Rc<Expression>,
    pub consequence: Option<BlockStatement>,
    pub alternative: Option<BlockStatement>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for IfExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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
pub struct FnLiteralExpression {
    pub token: Rc<Token>,
    pub params: Vec<IdentifierExpression>,
    pub body: Option<BlockStatement>,
}

impl Node for FnLiteralExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for FnLiteralExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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
pub struct CallExpression {
    pub token: Rc<Token>,
    /**
     * Identifier or FunctionLiteral
     */
    pub function: Rc<Expression>,
    pub args: Vec<Expression>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for CallExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StringExpression {
    pub token: Rc<Token>,
    pub value: Rc<str>,
}

impl Node for StringExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for StringExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.token_literal())
    }
}
