use phf::{Map, phf_map};

use crate::lexer::TokenType;

pub enum Precedence {
    Lowest = 1,
    Equals,      // == or !=
    LessGreater, // > or <
    Sum,         // + or -
    Product,     // * or /
    Prefix,      // -X or !X
    Call,        //    myFunc(x)
}

impl Precedence {
    pub fn lookup_precedence(tok: TokenType) -> Precedence {
        match tok {
            TokenType::Eq => Precedence::Equals,
            TokenType::NotEq => Precedence::Equals,
            TokenType::Lt => Precedence::LessGreater,
            TokenType::Gt => Precedence::LessGreater,
            TokenType::Plus => Precedence::Sum,
            TokenType::Minus => Precedence::Sum,
            TokenType::Slash => Precedence::Product,
            TokenType::Asterisk => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }
}
