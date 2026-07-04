use std::collections::HashMap;

use crate::{
    ast::{
        self, Expression, ExpressionStatement, Identifier, IntegerLiteral, LetStatement, Prefix,
        Program, ReturnStatement, Statement,
    },
    lexer::{
        client::Lexer,
        token::{Token, TokenType},
    },
    parser::error::ParserError,
};

pub enum Precedence {
    Lowest = 1,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        //    myFunc(x)
}

type PrefixParseFn<'a> = fn(&mut Parser<'a>) -> Option<Expression<'a>>;
type InfixParseFn<'a> = fn(&mut Parser<'a>, expr: Expression<'a>) -> Expression<'a>;

struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token<'a>,
    peek_token: Token<'a>,
    errors: Vec<ParserError>,

    prefix_parse_fns: HashMap<TokenType, PrefixParseFn<'a>>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Result<Self, ParserError> {
        let mut parser = Self {
            lexer,
            cur_token: Token::default(),
            peek_token: Token::default(),
            errors: vec![],
            prefix_parse_fns: HashMap::default(),
            infix_parse_fns: HashMap::default(),
        };

        parser.register_prefix(TokenType::Ident, Parser::parse_identifier);
        parser.register_prefix(TokenType::Int, Parser::parse_integer_literal);
        parser.register_prefix(TokenType::Bang, Parser::parse_prefix_expression);
        parser.register_prefix(TokenType::Minus, Parser::parse_prefix_expression);

        parser.next_token();
        parser.next_token();

        Ok(parser)
    }

    fn register_prefix(&mut self, typ: TokenType, func: PrefixParseFn<'a>) {
        self.prefix_parse_fns.insert(typ, func);
    }

    fn parse_identifier(&mut self) -> Option<Expression<'a>> {
        Some(Expression::Identifier(Identifier {
            token: self.cur_token,
            value: self.cur_token.literal,
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expression<'a>> {
        let token = self.cur_token;

        let value = match self.cur_token.literal.parse::<i64>() {
            Ok(v) => Some(v),
            Err(e) => {
                self.errors.push(e.into());
                None
            }
        }?;

        Some(Expression::IntegerLiteral(IntegerLiteral { token, value }))
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression<'a>> {
        let token = self.cur_token;

        self.next_token();

        Some(Expression::Prefix(Prefix {
            token,
            op: token.literal,
            right: Box::new(self.parse_expression(Precedence::Prefix)?),
        }))
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token;
        match self.lexer.next_token() {
            Ok(tok) => self.peek_token = tok,
            Err(e) => {
                self.errors.push(e.into());
                self.peek_token = Token::default();
            }
        }
    }

    fn parse_statement(&mut self) -> Option<Statement<'a>> {
        match self.cur_token.typ {
            TokenType::Let => self.parse_let_statement().map(Statement::Let),
            TokenType::Return => self.parse_return_statement().map(Statement::Return),
            _ => self.parse_expression_statement().map(Statement::Expression),
        }
    }

    fn parse_expression_statement(&mut self) -> Option<ExpressionStatement<'a>> {
        let token = self.cur_token;

        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ExpressionStatement { token, expr })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression<'a>> {
        let prefix = match self.prefix_parse_fns.get(&self.cur_token.typ) {
            Some(v) => Ok(v),
            None => Err(ParserError::NoPrefixParseFnFound {
                typ: self.cur_token.typ,
            }),
        }
        .map_err(|e| self.errors.push(e))
        .ok()?;

        prefix(self)
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement<'a>> {
        let token = self.cur_token;

        self.next_token();

        while !self.cur_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ReturnStatement { token, value: None })
    }

    fn parse_let_statement(&mut self) -> Option<LetStatement<'a>> {
        let token = self.cur_token;

        if !self
            .expect_peek(TokenType::Ident)
            .map_err(|e| self.errors.push(e))
            .ok()?
        {
            return None;
        }

        let name = Identifier {
            token: self.cur_token,
            value: self.cur_token.literal,
        };

        if !self
            .expect_peek(TokenType::Assign)
            .map_err(|e| self.errors.push(e))
            .ok()?
        {
            return None;
        }

        while !self.cur_token_is(TokenType::Semicolon) {
            self.next_token()
        }

        Some(LetStatement {
            name,
            token,
            value: None,
        })
    }

    fn cur_token_is(&self, typ: TokenType) -> bool {
        self.cur_token.typ == typ
    }

    fn peek_token_is(&self, typ: TokenType) -> bool {
        self.peek_token.typ == typ
    }

    fn expect_peek(&mut self, typ: TokenType) -> Result<bool, ParserError> {
        if self.peek_token_is(typ) {
            self.next_token();
            Ok(true)
        } else {
            self.errors.push(ParserError::UnexpectedToken {
                expected: typ,
                got: self.peek_token.typ,
            });
            Ok(false)
        }
    }

    pub fn parse_program(&mut self) -> Result<ast::Program<'a>, Vec<ParserError>> {
        let mut statements = vec![];

        while self.cur_token.typ != TokenType::Eof {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        if !self.errors.is_empty() {
            Err(std::mem::take(&mut self.errors))
        } else {
            Ok(Program { statements })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::format;

    use crate::{
        ast::{Node, Statement, statement},
        lexer,
    };

    use super::*;

    struct IdentifierTest<'a> {
        expected_identifier: &'a str,
    }

    fn test_let_statement(s: &Statement, name: &str) {
        if s.token_literal() != "let" {
            panic!(
                "expected s.token_literal to return \"let\", got {}",
                s.token_literal()
            )
        }

        let Statement::Let(ls) = s else {
            panic!("expected let statement but did not receive that")
        };

        if ls.name.value != name {
            panic!(
                "ls.name.value should have been {} but was {}",
                name, ls.name.value
            )
        }

        if ls.name.token_literal() != name {
            panic!("statement should have been {} but was {:?}", name, ls.name)
        }
    }

    fn test_integer_literal(il: &Expression, v: i64) {
        let Expression::IntegerLiteral(integ) = il else {
            panic!("expected IntegerLiteral expression, received {}", il)
        };

        if integ.value != v {
            panic!(
                "integ.value was expected to be {} but was {}",
                v, integ.value
            )
        }

        if integ.token_literal() != v.to_string() {
            panic!(
                "integ.token_literal() expected to be {} but was {}",
                v.to_string(),
                integ.token_literal()
            )
        }
    }

    #[test]
    fn test_let_statements() {
        let input = "
let x = 5;
let y = 10;
let foobar = 838383;
";

        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();

        let prog = match parser.parse_program() {
            Err(e) if !e.is_empty() => {
                let errs = e
                    .iter()
                    .map(|err| format!("\t{}", err))
                    .collect::<Vec<_>>()
                    .join("\n");

                panic!("parser has {} errors:\n{}", e.len(), errs)
            }
            Ok(prog) if prog.statements.len() != 3 => {
                panic!(
                    "expected 3 statements but received {}",
                    prog.statements.len()
                )
            }
            Ok(p) => p,
            // no need to worry abt this one
            Err(_) => panic!(),
        };

        let tests: [IdentifierTest; 3] = [
            IdentifierTest {
                expected_identifier: "x",
            },
            IdentifierTest {
                expected_identifier: "y",
            },
            IdentifierTest {
                expected_identifier: "foobar",
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            match &prog.statements.get(i) {
                Some(s) => test_let_statement(s, test.expected_identifier),
                None => panic!("program statement index {} missing", i),
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let input = "
return 5;
return 10;
return 993322;
";

        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();

        let prog = match parser.parse_program() {
            Err(e) if !e.is_empty() => {
                let errs = e
                    .iter()
                    .map(|err| format!("\t{}", err))
                    .collect::<Vec<_>>()
                    .join("\n");

                panic!("parser has {} errors:\n{}", e.len(), errs)
            }
            Ok(prog) if prog.statements.len() != 3 => {
                panic!(
                    "expected 3 statements but received {}",
                    prog.statements.len()
                )
            }
            Ok(p) => p,
            // no need to worry abt this one
            Err(_) => panic!(),
        };

        for stmt in prog.statements.iter() {
            let rt = match stmt {
                Statement::Return(rt) => rt,
                _ => panic!("expected return statement, got {:?}", stmt),
            };

            if rt.token_literal() != "return" {
                panic!(
                    "rt.token_literal should have returned \"return\", got {}",
                    rt.token_literal()
                )
            }
        }
    }

    #[test]
    fn test_identifier_expressions() {
        let input = "foobar;";

        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();

        let prog = match parser.parse_program() {
            Err(e) if !e.is_empty() => {
                let errs = e
                    .iter()
                    .map(|err| format!("\t{}", err))
                    .collect::<Vec<_>>()
                    .join("\n");

                panic!("parser has {} errors:\n{}", e.len(), errs)
            }
            Ok(prog) if prog.statements.len() != 1 => {
                panic!(
                    "expected 1 statements but received {}",
                    prog.statements.len()
                )
            }
            Ok(p) => p,
            // no need to worry abt this one
            Err(_) => panic!(),
        };

        let Statement::Expression(es) = &prog.statements[0] else {
            panic!(
                "expected expression statement, received {}",
                prog.statements[0]
            )
        };

        let Expression::Identifier(ident) = &es.expr else {
            panic!("expected identifier expression, recieved {}", es.expr)
        };

        if ident.value != "foobar" {
            panic!(
                "ident.value should have been \"foobar\", recieved {}",
                ident.value
            )
        }

        if ident.token_literal() != "foobar" {
            panic!(
                "ident.token_literal() should have been \"foobar\", recieved {}",
                ident.value
            )
        }
    }

    #[test]
    fn test_integer_literal_expressions() {
        let input = "5;";

        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();

        let prog = match parser.parse_program() {
            Err(e) if !e.is_empty() => {
                let errs = e
                    .iter()
                    .map(|err| format!("\t{}", err))
                    .collect::<Vec<_>>()
                    .join("\n");

                panic!("parser has {} errors:\n{}", e.len(), errs)
            }
            Ok(prog) if prog.statements.len() != 1 => {
                panic!(
                    "expected 1 statements but received {}",
                    prog.statements.len()
                )
            }
            Ok(p) => p,
            // no need to worry abt this one
            Err(_) => panic!(),
        };

        let Statement::Expression(es) = &prog.statements[0] else {
            panic!(
                "expected expression statement, received {}",
                prog.statements[0]
            )
        };

        let Expression::IntegerLiteral(ident) = &es.expr else {
            panic!("expected identifier expression, recieved {}", es.expr)
        };

        if ident.value != 5 {
            panic!("ident.value should have been 5, recieved {}", ident.value)
        }

        if ident.token_literal() != "5" {
            panic!(
                "ident.token_literal() should have been \"5\", recieved {}",
                ident.value
            )
        }
    }

    #[test]
    fn test_parsing_prefix_expressions<'a>() {
        struct PrefixTest<'a> {
            input: &'a str,
            op: &'a str,
            int_value: i64,
        }

        let prefix_tests: [PrefixTest<'a>; 2] = [
            PrefixTest {
                input: "!5;",
                op: "!",
                int_value: 5,
            },
            PrefixTest {
                input: "-15;",
                op: "-",
                int_value: 15,
            },
        ];

        for prefix_test in prefix_tests.iter() {
            let lexer = lexer::Lexer::new(prefix_test.input);
            let mut parser = Parser::new(lexer).unwrap();

            let prog = match parser.parse_program() {
                Err(e) if !e.is_empty() => {
                    let errs = e
                        .iter()
                        .map(|err| format!("\t{}", err))
                        .collect::<Vec<_>>()
                        .join("\n");

                    panic!("parser has {} errors:\n{}", e.len(), errs)
                }
                Ok(prog) if prog.statements.len() != 1 => {
                    panic!(
                        "expected 1 statements but received {}",
                        prog.statements.len()
                    )
                }
                Ok(p) => p,
                // no need to worry abt this one
                Err(_) => panic!(),
            };

            let Statement::Expression(es) = &prog.statements[0] else {
                panic!(
                    "expected expression statement, received {:?}",
                    prog.statements[0]
                )
            };

            let Expression::Prefix(expr) = &es.expr else {
                panic!("expected identifier expression, recieved {:?}", es.expr)
            };

            if expr.op != prefix_test.op {
                panic!(
                    "expr.op should have been {}, recieved {}",
                    prefix_test.op, expr.op
                )
            }

            test_integer_literal(expr.right.as_ref(), prefix_test.int_value);
        }
    }
}
