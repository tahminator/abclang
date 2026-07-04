use crate::lexer::{
    error::LexerError,
    token::{Token, TokenType, lookup_ident},
    utils::{LexerCharExt, is_digit, is_letter},
};

pub struct Lexer<'a> {
    input: &'a [u8],
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

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input: input.as_bytes(),
            pos: 0,
            read_pos: 0,
            ch: 0,
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, LexerError> {
        self.skip_whitespace();

        let tok = match self.ch {
            b'=' => {
                if let Some(b'=') = self.peek_char() {
                    self.read_char();
                    Token {
                        literal: "==",
                        typ: TokenType::Eq,
                    }
                } else {
                    Token {
                        literal: "=",
                        typ: TokenType::Assign,
                    }
                }
            }
            b'+' => Token {
                literal: "+",
                typ: TokenType::Plus,
            },
            b'-' => Token {
                literal: "-",
                typ: TokenType::Minus,
            },
            b'!' => {
                if let Some(b'=') = self.peek_char() {
                    self.read_char();
                    Token {
                        literal: "!=",
                        typ: TokenType::NotEq,
                    }
                } else {
                    Token {
                        literal: "!",
                        typ: TokenType::Bang,
                    }
                }
            }
            b'/' => Token {
                literal: "/",
                typ: TokenType::Slash,
            },
            b'*' => Token {
                literal: "*",
                typ: TokenType::Asterisk,
            },
            b'<' => Token {
                literal: "<",
                typ: TokenType::Lt,
            },
            b'>' => Token {
                literal: ">",
                typ: TokenType::Gt,
            },
            b',' => Token {
                literal: ",",
                typ: TokenType::Comma,
            },
            b';' => Token {
                literal: ";",
                typ: TokenType::Semicolon,
            },
            b'(' => Token {
                literal: "(",
                typ: TokenType::LParen,
            },
            b')' => Token {
                literal: ")",
                typ: TokenType::RParen,
            },
            b'{' => Token {
                literal: "{",
                typ: TokenType::LBrace,
            },
            b'}' => Token {
                literal: "}",
                typ: TokenType::RBrace,
            },
            0 => Token {
                literal: "",
                typ: TokenType::Eof,
            },
            _ if is_letter(self.ch) => {
                let ident = self.read_identifier()?;
                return Ok(Token {
                    literal: ident,
                    typ: lookup_ident(ident),
                });
            }
            _ if is_digit(self.ch) => {
                let num = self.read_number()?;
                return Ok(Token {
                    literal: num,
                    typ: TokenType::Int,
                });
            }
            _ => Token {
                literal: EMPTY,
                typ: TokenType::Illegal,
            },
        };

        self.read_char();
        Ok(tok)
    }

    fn read_identifier(&mut self) -> Result<&'a str, LexerError> {
        let pos = self.pos;
        while is_letter(self.ch) {
            self.read_char();
        }
        self.input[pos..self.pos].as_str()
    }

    fn read_number(&mut self) -> Result<&'a str, LexerError> {
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
                literal: "=",
            },
            Token {
                typ: TokenType::Plus,
                literal: "+",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
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
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Function,
                literal: "fn",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Plus,
                literal: "+",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "result",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Eof,
                literal: "",
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
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Function,
                literal: "fn",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Plus,
                literal: "+",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "result",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Bang,
                literal: "!",
            },
            Token {
                typ: TokenType::Minus,
                literal: "-",
            },
            Token {
                typ: TokenType::Slash,
                literal: "/",
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Lt,
                literal: "<",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Gt,
                literal: ">",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Eof,
                literal: "",
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
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Function,
                literal: "fn",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Plus,
                literal: "+",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "result",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Bang,
                literal: "!",
            },
            Token {
                typ: TokenType::Minus,
                literal: "-",
            },
            Token {
                typ: TokenType::Slash,
                literal: "/",
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Lt,
                literal: "<",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Gt,
                literal: ">",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::If,
                literal: "if",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Lt,
                literal: "<",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Return,
                literal: "return",
            },
            Token {
                typ: TokenType::True,
                literal: "true",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Else,
                literal: "else",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Return,
                literal: "return",
            },
            Token {
                typ: TokenType::False,
                literal: "false",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Eof,
                literal: "",
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
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Function,
                literal: "fn",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Ident,
                literal: "x",
            },
            Token {
                typ: TokenType::Plus,
                literal: "+",
            },
            Token {
                typ: TokenType::Ident,
                literal: "y",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Let,
                literal: "let",
            },
            Token {
                typ: TokenType::Ident,
                literal: "result",
            },
            Token {
                typ: TokenType::Assign,
                literal: "=",
            },
            Token {
                typ: TokenType::Ident,
                literal: "add",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Ident,
                literal: "five",
            },
            Token {
                typ: TokenType::Comma,
                literal: ",",
            },
            Token {
                typ: TokenType::Ident,
                literal: "ten",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Bang,
                literal: "!",
            },
            Token {
                typ: TokenType::Minus,
                literal: "-",
            },
            Token {
                typ: TokenType::Slash,
                literal: "/",
            },
            Token {
                typ: TokenType::Asterisk,
                literal: "*",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Lt,
                literal: "<",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Gt,
                literal: ">",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::If,
                literal: "if",
            },
            Token {
                typ: TokenType::LParen,
                literal: "(",
            },
            Token {
                typ: TokenType::Int,
                literal: "5",
            },
            Token {
                typ: TokenType::Lt,
                literal: "<",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::RParen,
                literal: ")",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Return,
                literal: "return",
            },
            Token {
                typ: TokenType::True,
                literal: "true",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Else,
                literal: "else",
            },
            Token {
                typ: TokenType::LBrace,
                literal: "{",
            },
            Token {
                typ: TokenType::Return,
                literal: "return",
            },
            Token {
                typ: TokenType::False,
                literal: "false",
            },
            Token {
                typ: TokenType::Semicolon,
                literal: ";",
            },
            Token {
                typ: TokenType::RBrace,
                literal: "}",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Eq,
                literal: "==",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::Int,
                literal: "10",
            },
            Token {
                typ: TokenType::NotEq,
                literal: "!=",
            },
            Token {
                typ: TokenType::Int,
                literal: "9",
            },
            Token {
                typ: TokenType::Eof,
                literal: "",
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
