use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use crate::{
    ast::{BlockStatement, Node},
    lexer::token::Token,
};

macro_rules! expr {
    ($($variant:ident($ty:ty)),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum Expression {
            $($variant($ty),)*
        }

        impl Display for Expression {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                match self {
                    $(Expression::$variant(expr) => write!(f, "{expr}"),)*
                }
            }
        }
    };
}

expr! {
    Identifier(IdentifierExpression),
    IntegerLiteral(IntegerLiteralExpression),
    Prefix(PrefixExpression),
    Infix(InfixExpression),
    Boolean(BooleanExpression),
    If(IfExpression),
    FnLiteral(FnLiteralExpression),
    Call(CallExpression),
    String(StringExpression),
    Array(ArrayExpression),
    Index(IndexExpression),
    Hash(HashExpression),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InfixExpression {
    pub token: Rc<Token>,
    pub left: Rc<Expression>,
    pub op: Rc<str>,
    pub right: Rc<Expression>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for InfixExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayExpression {
    pub token: Rc<Token>,
    pub elements: Vec<Expression>,
}

impl Node for ArrayExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for ArrayExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "[{}]",
            self.elements
                .iter()
                .map(|el| el.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexExpression {
    pub token: Rc<Token>,
    pub left: Rc<Expression>,
    pub index: Rc<Expression>,
}

impl Node for IndexExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for IndexExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}[{}])", self.left, self.index,)
    }
}

#[derive(Debug, Clone, Eq)]
pub struct HashExpression {
    pub token: Rc<Token>,
    pub pairs: HashMap<Expression, Expression>,
}

impl Hash for HashExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.token.hash(state);

        let mut acc: u64 = 0;
        for (key, value) in &self.pairs {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            value.hash(&mut hasher);
            acc = acc.wrapping_add(hasher.finish());
        }
        acc.hash(state);
    }
}

impl PartialEq for HashExpression {
    fn eq(&self, other: &Self) -> bool {
        if self.token != other.token {
            return false;
        }

        if self.pairs.len() != other.pairs.len() {
            return false;
        }

        self.pairs
            .iter()
            .all(|(key, value)| other.pairs.get(key) == Some(value))
    }
}

impl Node for HashExpression {
    fn token_literal(&self) -> Rc<str> {
        self.token.literal.clone()
    }
}

impl Display for HashExpression {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{{{}}}",
            self.pairs
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
