use std::{rc::Rc, sync::Arc};

use crate::lexer::{
    error::LexerError,
    token::{Token, TokenType, lookup_ident},
    utils::{LexerCharExt, is_digit, is_letter},
};

pub struct Lexer {
    input: Box<[u8]>,
    /**
     * current position in input (points to current char)
     */
    pos: usize,
    /**
     * current reading position in input (after current char)
     */
    read_pos: usize,
    ch: u8,
}

static EMPTY: &str = "";

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Self {
            input: input.as_bytes().into(),
            pos: 0,
            read_pos: 0,
            ch: 0,
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        let tok = match self.ch {
            b'=' => {
                if let Some(b'=') = self.peek_char() {
                    self.read_char();
                    Token {
                        literal: "==".into(),
                        typ: TokenType::Eq,
                    }
                } else {
                    Token {
                        literal: "=".into(),
                        typ: TokenType::Assign,
                    }
                }
            }
            b'+' => Token {
                literal: "+".into(),
                typ: TokenType::Plus,
            },
            b'-' => Token {
                literal: "-".into(),
                typ: TokenType::Minus,
            },
            b'!' => {
                if let Some(b'=') = self.peek_char() {
                    self.read_char();
                    Token {
                        literal: "!=".into(),
                        typ: TokenType::NotEq,
                    }
                } else {
                    Token {
                        literal: "!".into(),
                        typ: TokenType::Bang,
                    }
                }
            }
            b'/' => {
                if let Some(b'/') = self.peek_char() {
                    self.skip_comment();
                    return self.next_token();
                } else {
                    Token {
                        literal: "/".into(),
                        typ: TokenType::Slash,
                    }
                }
            }
            b'*' => Token {
                literal: "*".into(),
                typ: TokenType::Asterisk,
            },
            b'<' => Token {
                literal: "<".into(),
                typ: TokenType::Lt,
            },
            b'>' => Token {
                literal: ">".into(),
                typ: TokenType::Gt,
            },
            b',' => Token {
                literal: ",".into(),
                typ: TokenType::Comma,
            },
            b';' => Token {
                literal: ";".into(),
                typ: TokenType::Semicolon,
            },
            b'(' => Token {
                literal: "(".into(),
                typ: TokenType::LParen,
            },
            b')' => Token {
                literal: ")".into(),
                typ: TokenType::RParen,
            },
            b'{' => Token {
                literal: "{".into(),
                typ: TokenType::LBrace,
            },
            b'}' => Token {
                literal: "}".into(),
                typ: TokenType::RBrace,
            },
            b'"' => Token {
                literal: self.read_string()?.into(),
                typ: TokenType::String,
            },
            b'[' => Token {
                literal: "[".into(),
                typ: TokenType::LBracket,
            },
            b']' => Token {
                literal: "]".into(),
                typ: TokenType::RBracket,
            },
            0 => Token {
                literal: "".into(),
                typ: TokenType::Eof,
            },
            _ if is_letter(self.ch) => {
                let ident = self.read_identifier()?;
                return Ok(Token {
                    literal: ident.into(),
                    typ: lookup_ident(ident),
                });
            }
            _ if is_digit(self.ch) => {
                let num = self.read_number()?;
                return Ok(Token {
                    literal: num.into(),
                    typ: TokenType::Int,
                });
            }
            _ => Token {
                literal: EMPTY.into(),
                typ: TokenType::Illegal,
            },
        };

        self.read_char();
        Ok(tok)
    }

    fn read_string(&mut self) -> Result<&str, LexerError> {
        let pos = self.pos + 1;

        loop {
            self.read_char();
            if self.ch == b'"' || self.ch == 0 {
                break;
            }
        }

        self.input[pos..self.pos].as_str()
    }

    fn skip_comment(&mut self) {
        while self.ch != b'\n' && self.ch != 0 {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> Result<&str, LexerError> {
        let pos = self.pos;
        while is_letter(self.ch) {
            self.read_char();
        }
        self.input[pos..self.pos].as_str()
    }

    fn read_number(&mut self) -> Result<&str, LexerError> {
        let pos = self.pos;
        while is_digit(self.ch) {
            self.read_char();
        }
        self.input[pos..self.pos].as_str()
    }

    fn skip_whitespace(&mut self) {
        while self.ch == b' ' || self.ch == b'\t' || self.ch == b'\n' || self.ch == b'\r' {
            self.read_char();
        }
    }

    fn read_char(&mut self) {
        if self.read_pos >= self.input.len() {
            self.ch = 0
        } else {
            self.ch = self.input[self.read_pos]
        }
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn peek_char(&mut self) -> Option<u8> {
        if self.read_pos >= self.input.len() {
            return None;
        }

        Some(self.input[self.read_pos])
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::token::Token;

    use super::*;

    #[test]
    fn test_next_token_basic() {
        let input = "=+(){},;";

        let mut lexer = Lexer::new(input);

        let outputs: [Token; 8] = [
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(output, token);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_next_token_real_scenario() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);";

        let mut lexer = Lexer::new(input);

        let outputs: [Token; 37] = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Function,
                literal: "fn".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "result".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_next_token_real_scenario2() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;";

        let mut lexer = Lexer::new(input);

        let outputs: [Token; 49] = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Function,
                literal: "fn".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "result".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Bang,
                literal: "!".into(),
            },
            Token {
                typ: TokenType::Minus,
                literal: "-".into(),
            },
            Token {
                typ: TokenType::Slash,
                literal: "/".into(),
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Gt,
                literal: ">".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_next_token_real_scenario3() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}";

        let mut lexer = Lexer::new(input);

        let outputs: [Token; 66] = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Function,
                literal: "fn".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "result".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Bang,
                literal: "!".into(),
            },
            Token {
                typ: TokenType::Minus,
                literal: "-".into(),
            },
            Token {
                typ: TokenType::Slash,
                literal: "/".into(),
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Gt,
                literal: ">".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::If,
                literal: "if".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::True,
                literal: "true".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Else,
                literal: "else".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::False,
                literal: "false".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_next_token_real_scenario4() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10
10 != 9
";

        let mut lexer = Lexer::new(input);

        let outputs: [Token; 72] = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Function,
                literal: "fn".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "result".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Bang,
                literal: "!".into(),
            },
            Token {
                typ: TokenType::Minus,
                literal: "-".into(),
            },
            Token {
                typ: TokenType::Slash,
                literal: "/".into(),
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Gt,
                literal: ">".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::If,
                literal: "if".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::True,
                literal: "true".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Else,
                literal: "else".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::False,
                literal: "false".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Eq,
                literal: "==".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::NotEq,
                literal: "!=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "9".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_next_token_real_scenario5_with_string() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10
10 != 9

\"foobar\"
\"foo bar\"
";

        let mut lexer = Lexer::new(input);

        let outputs = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Function,
                literal: "fn".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "result".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Bang,
                literal: "!".into(),
            },
            Token {
                typ: TokenType::Minus,
                literal: "-".into(),
            },
            Token {
                typ: TokenType::Slash,
                literal: "/".into(),
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Gt,
                literal: ">".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::If,
                literal: "if".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::True,
                literal: "true".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Else,
                literal: "else".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::False,
                literal: "false".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Eq,
                literal: "==".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::NotEq,
                literal: "!=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "9".into(),
            },
            Token {
                typ: TokenType::String,
                literal: "foobar".into(),
            },
            Token {
                typ: TokenType::String,
                literal: "foo bar".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_comment() {
        let input = "// cyz
        let x = 5;
        ";

        let mut lexer = Lexer::new(input);

        let outputs: [Token; 6] = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_next_token_real_scenario5_with_array() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10
10 != 9

\"foobar\"
\"foo bar\"
[1, 2];
";

        let mut lexer = Lexer::new(input);

        let outputs = [
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Function,
                literal: "fn".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "x".into(),
            },
            Token {
                typ: TokenType::Plus,
                literal: "+".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "y".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Let,
                literal: "let".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "result".into(),
            },
            Token {
                typ: TokenType::Assign,
                literal: "=".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "add".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "five".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Bang,
                literal: "!".into(),
            },
            Token {
                typ: TokenType::Minus,
                literal: "-".into(),
            },
            Token {
                typ: TokenType::Slash,
                literal: "/".into(),
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Gt,
                literal: ">".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::If,
                literal: "if".into(),
            },
            Token {
                typ: TokenType::LParen,
                literal: "(".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "5".into(),
            },
            Token {
                typ: TokenType::Lt,
                literal: "<".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::RParen,
                literal: ")".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::True,
                literal: "true".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Else,
                literal: "else".into(),
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{".into(),
            },
            Token {
                typ: TokenType::Return,
                literal: "return".into(),
            },
            Token {
                typ: TokenType::False,
                literal: "false".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Eq,
                literal: "==".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "10".into(),
            },
            Token {
                typ: TokenType::NotEq,
                literal: "!=".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "9".into(),
            },
            Token {
                typ: TokenType::String,
                literal: "foobar".into(),
            },
            Token {
                typ: TokenType::String,
                literal: "foo bar".into(),
            },
            Token {
                typ: TokenType::LBracket,
                literal: "[".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "1".into(),
            },
            Token {
                typ: TokenType::Comma,
                literal: ",".into(),
            },
            Token {
                typ: TokenType::Int,
                literal: "2".into(),
            },
            Token {
                typ: TokenType::RBracket,
                literal: "]".into(),
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";".into(),
            },
            Token {
                typ: TokenType::Eof,
                literal: "".into(),
            },
        ];

        for output in outputs {
            match lexer.next_token() {
                Ok(token) => {
                    assert_eq!(token, output);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }
}
