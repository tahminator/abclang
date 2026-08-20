use crate::lexer::TokenType;

#[derive(PartialEq, PartialOrd)]
pub enum Precedence {
    Lowest = 1,
    Or,          // ||
    And,         // &&
    Equals,      // == or !=
    LessGreater, // > or <
    Sum,         // + or -
    Product,     // * or /
    Prefix,      // -X or !X
    Call,        // myFunc(x)
    Index,       // array[index]
}

impl Precedence {
    pub fn lookup_precedence(tok: TokenType) -> Precedence {
        match tok {
            TokenType::Eq | TokenType::NotEq => Precedence::Equals,
            TokenType::Lt | TokenType::Gt => Precedence::LessGreater,
            TokenType::Or => Precedence::Or,
            TokenType::And => Precedence::And,
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Slash | TokenType::Asterisk => Precedence::Product,
            TokenType::LParen => Precedence::Call,
            TokenType::LBracket | TokenType::Dot => Precedence::Index,
            _ => Precedence::Lowest,
        }
    }
}
