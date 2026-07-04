use std::fmt;

use parse_display::Display;
use phf::phf_map;

#[derive(Debug, PartialEq, Clone, Copy, Display)]
#[display("Token({literal}, {typ})")]
pub struct Token<'a> {
    pub literal: &'a str,
    pub typ: TokenType,
}

impl<'a> Default for Token<'a> {
    fn default() -> Self {
        Token {
            literal: "",
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
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
