use std::{fmt, rc::Rc};

use parse_display::Display;
use phf::phf_map;

#[derive(Debug, PartialEq, Clone, Display)]
#[display("Token({literal}, {typ})")]
pub struct Token {
    pub literal: Rc<str>,
    pub typ: TokenType,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            literal: "".into(),
            typ: TokenType::Eof,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TokenType {
    Illegal,
    Eof,

    Ident,
    Int,

    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,

    Lt,
    Gt,

    Eq,
    NotEq,

    Comma,
    Semicolon,

    LParen,
    RParen,
    LBrace,
    RBrace,

    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,

    Comment,
    String,

    LBracket,
    RBracket,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TokenType::Illegal => "ILLEGAL",
            TokenType::Eof => "EOF",
            TokenType::Ident => "IDENT",
            TokenType::Int => "INT",
            TokenType::Assign => "=",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Bang => "!",
            TokenType::Asterisk => "*",
            TokenType::Slash => "/",
            TokenType::Lt => "<",
            TokenType::Gt => ">",
            TokenType::Comma => ",",
            TokenType::Semicolon => ";",
            TokenType::LParen => "(",
            TokenType::RParen => ")",
            TokenType::LBrace => "{",
            TokenType::RBrace => "}",
            TokenType::Function => "fn",
            TokenType::Let => "let",
            TokenType::True => "true",
            TokenType::False => "false",
            TokenType::If => "if",
            TokenType::Else => "else",
            TokenType::Return => "return",
            TokenType::Eq => "==",
            TokenType::NotEq => "!=",
            TokenType::Comment => "//",
            TokenType::String => "String",
            TokenType::LBracket => "[",
            TokenType::RBracket => "]",
        };
        f.write_str(s)
    }
}

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "fn" => TokenType::Function,
    "let" => TokenType::Let,
    "true" => TokenType::True,
    "false" => TokenType::False,
    "if" => TokenType::If,
    "else" => TokenType::Else,
    "return" => TokenType::Return,
};

pub fn lookup_ident(ident: &str) -> TokenType {
    KEYWORDS.get(ident).copied().unwrap_or(TokenType::Ident)
}
