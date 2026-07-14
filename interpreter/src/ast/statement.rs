use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    rc::Rc,
};

use crate::{
    ast::{Expression, IdentifierExpression, Node},
    lexer::token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Let(LetStatement),
    Assign(AssignStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut w = |x: &dyn Display| write!(f, "{x}");

        match self {
            Statement::Let(stmt) => w(stmt),
            Statement::Assign(stmt) => w(stmt),
            Statement::Return(stmt) => w(stmt),
            Statement::Expression(stmt) => w(stmt),
            Statement::Block(stmt) => w(stmt),
        }
    }
}

impl Node for Statement {
    fn token_literal(&self) -> Rc<str> {
        match self {
            Statement::Let(stmt) => stmt.token_literal(),
            Statement::Assign(stmt) => stmt.token_literal(),
            Statement::Return(stmt) => stmt.token_literal(),
            Statement::Expression(stmt) => stmt.token_literal(),
            Statement::Block(stmt) => stmt.token_literal(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetStatement {
    pub token: Rc<Token>,
    pub name: IdentifierExpression,
    pub value: Option<Expression>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssignStatement {
    pub token: Rc<Token>,
    pub name: IdentifierExpression,
    pub value: Expression,
}

impl Node for AssignStatement {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for AssignStatement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} = {};", self.name.value, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReturnStatement {
    pub token: Rc<Token>,
    pub value: Option<Expression>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for ReturnStatement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionStatement {
    pub token: Rc<Token>,
    pub expr: Expression,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for ExpressionStatement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockStatement {
    pub token: Rc<Token>,
    pub statements: Vec<Statement>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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
