use std::collections::HashMap;

use crate::{
    ast::{
        self, Boolean, Expression, ExpressionStatement, Identifier, Infix, IntegerLiteral,
        LetStatement, Prefix, Program, ReturnStatement, Statement,
    },
    lexer::{Lexer, Token, TokenType},
    parser::{error::ParserError, precedence::Precedence},
};

type PrefixParseFn<'a> = fn(&mut Parser<'a>) -> Option<Expression<'a>>;
type InfixParseFn<'a> = fn(&mut Parser<'a>, expr: Expression<'a>) -> Option<Expression<'a>>;

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
        parser.register_prefix(TokenType::True, Parser::parse_boolean);
        parser.register_prefix(TokenType::False, Parser::parse_boolean);
        parser.register_prefix(TokenType::LParen, Parser::parse_grouped_expression);

        let infix_func = Parser::parse_infix_expression;
        parser.register_infix(TokenType::Plus, infix_func);
        parser.register_infix(TokenType::Minus, infix_func);
        parser.register_infix(TokenType::Slash, infix_func);
        parser.register_infix(TokenType::Asterisk, infix_func);
        parser.register_infix(TokenType::Eq, infix_func);
        parser.register_infix(TokenType::NotEq, infix_func);
        parser.register_infix(TokenType::Lt, infix_func);
        parser.register_infix(TokenType::Gt, infix_func);

        parser.next_token();
        parser.next_token();

        Ok(parser)
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

    fn parse_boolean(&mut self) -> Option<Expression<'a>> {
        Some(Expression::Boolean(Boolean {
            token: self.cur_token,
            value: self.cur_token_is(TokenType::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression<'a>> {
        self.next_token();

        let exp = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        return exp;
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

    fn parse_infix_expression(&mut self, expr: Expression<'a>) -> Option<Expression<'a>> {
        let token = self.cur_token;
        let left = expr;

        let precedence = self.cur_precedence();
        self.next_token();

        let right = self.parse_expression(precedence)?;

        Some(Expression::Infix(Infix {
            token,
            left: Box::new(left),
            op: token.literal,
            right: Box::new(right),
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

    fn peek_precedence(&self) -> Precedence {
        Precedence::lookup_precedence(self.peek_token.typ)
    }

    fn cur_precedence(&self) -> Precedence {
        Precedence::lookup_precedence(self.cur_token.typ)
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
        let Some(&prefix) = self.prefix_parse_fns.get(&self.cur_token.typ) else {
            self.errors.push(ParserError::NoPrefixParseFnFound {
                typ: self.cur_token.typ,
            });
            return None;
        };

        let mut left_expr = prefix(self)?;

        let p = precedence as u8;
        while !self.peek_token_is(TokenType::Semicolon) && p < (self.peek_precedence() as u8) {
            let Some(&infix) = self.infix_parse_fns.get(&self.peek_token.typ) else {
                return Some(left_expr);
            };

            self.next_token();

            left_expr = infix(self, left_expr)?;
        }

        Some(left_expr)
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

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        let name = Identifier {
            token: self.cur_token,
            value: self.cur_token.literal,
        };

        if !self.expect_peek(TokenType::Assign) {
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

    fn expect_peek(&mut self, typ: TokenType) -> bool {
        if self.peek_token_is(typ) {
            self.next_token();
            true
        } else {
            self.errors.push(ParserError::UnexpectedToken {
                expected: typ,
                got: self.peek_token.typ,
            });
            false
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

    fn register_prefix(&mut self, typ: TokenType, func: PrefixParseFn<'a>) {
        self.prefix_parse_fns.insert(typ, func);
    }

    fn register_infix(&mut self, typ: TokenType, func: InfixParseFn<'a>) {
        self.infix_parse_fns.insert(typ, func);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Node, Statement},
        lexer,
    };

    use super::*;

    struct IdentifierTest<'a> {
        expected_identifier: &'a str,
    }

    fn parse_program_or_panic(input: &str) -> Program<'_> {
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();

        match parser.parse_program() {
            Ok(prog) => prog,
            Err(errors) => {
                let errs = errors
                    .iter()
                    .map(|err| format!("\t{}", err))
                    .collect::<Vec<_>>()
                    .join("\n");

                panic!("parser has {} errors:\n{}", errors.len(), errs)
            }
        }
    }

    fn parse_program_with_len(input: &str, expected_len: usize) -> Program<'_> {
        let prog = parse_program_or_panic(input);

        if prog.statements.len() != expected_len {
            panic!(
                "expected {} statements but received {}",
                expected_len,
                prog.statements.len()
            )
        }

        prog
    }

    fn single_expression<'a>(prog: &'a Program) -> &'a Expression<'a> {
        let Statement::Expression(es) = &prog.statements[0] else {
            panic!(
                "expected expression statement, received {:?}",
                prog.statements[0]
            )
        };

        &es.expr
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
                v,
                integ.token_literal()
            )
        }
    }

    enum Expected<'a> {
        Int(i64),
        Ident(&'a str),
        Bool(bool),
    }

    impl From<i64> for Expected<'_> {
        fn from(v: i64) -> Self {
            Expected::Int(v)
        }
    }

    impl<'a> From<&'a str> for Expected<'a> {
        fn from(v: &'a str) -> Self {
            Expected::Ident(v)
        }
    }

    impl From<bool> for Expected<'_> {
        fn from(v: bool) -> Self {
            Expected::Bool(v)
        }
    }

    fn test_identifier(expr: &Expression, value: &str) {
        let Expression::Identifier(ident) = expr else {
            panic!("expected identifier expression, received {:?}", expr)
        };

        if ident.value != value {
            panic!(
                "ident.value should have been {} but was {}",
                value, ident.value
            )
        }

        if ident.token_literal() != value {
            panic!(
                "ident.token_literal() should have been {} but was {}",
                value,
                ident.token_literal()
            )
        }
    }

    fn test_boolean_literal(expr: &Expression, value: bool) {
        let Expression::Boolean(boolean) = expr else {
            panic!("expected boolean expression, received {:?}", expr)
        };

        if boolean.value != value {
            panic!(
                "boolean.value should have been {} but was {}",
                value, boolean.value
            )
        }

        if boolean.token_literal() != value.to_string() {
            panic!(
                "boolean.token_literal() should have been {} but was {}",
                value,
                boolean.token_literal()
            )
        }
    }

    fn test_literal_expr<'a>(expr: &Expression, expected: impl Into<Expected<'a>>) {
        match expected.into() {
            Expected::Int(v) => test_integer_literal(expr, v),
            Expected::Ident(v) => test_identifier(expr, v),
            Expected::Bool(v) => test_boolean_literal(expr, v),
        }
    }

    fn test_infix_expr<'a>(
        expr: &Expression,
        left: impl Into<Expected<'a>>,
        op: &str,
        right: impl Into<Expected<'a>>,
    ) {
        let Expression::Infix(infix) = expr else {
            panic!("expected infix expression, received {:?}", expr)
        };

        test_literal_expr(infix.left.as_ref(), left);

        if infix.op != op {
            panic!("expected operator {} but was {}", op, infix.op)
        }

        test_literal_expr(infix.right.as_ref(), right);
    }

    #[test]
    fn test_let_statements() {
        let input = "
let x = 5;
let y = 10;
let foobar = 838383;
";

        let prog = parse_program_with_len(input, 3);

        let tests = [
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

        let prog = parse_program_with_len(input, 3);

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

        let prog = parse_program_with_len(input, 1);

        test_literal_expr(single_expression(&prog), "foobar");
    }

    #[test]
    fn test_integer_literal_expressions() {
        let input = "5;";

        let prog = parse_program_with_len(input, 1);

        test_integer_literal(single_expression(&prog), 5);
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        struct PrefixTest {
            input: &'static str,
            op: &'static str,
            int_value: i64,
        }

        let prefix_tests = [
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
            let prog = parse_program_with_len(prefix_test.input, 1);

            let Expression::Prefix(expr) = single_expression(&prog) else {
                panic!(
                    "expected prefix expression, recieved {:?}",
                    single_expression(&prog)
                )
            };

            if expr.op != prefix_test.op {
                panic!(
                    "expr.op should have been {}, recieved {}",
                    prefix_test.op, expr.op
                )
            }

            test_literal_expr(expr.right.as_ref(), prefix_test.int_value);
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        struct InfixTest {
            input: &'static str,
            left_value: i64,
            op: &'static str,
            right_value: i64,
        }

        let infix_tests = [
            InfixTest {
                input: "5 + 5;",
                left_value: 5,
                op: "+",
                right_value: 5,
            },
            InfixTest {
                input: "5 - 5;",
                left_value: 5,
                op: "-",
                right_value: 5,
            },
            InfixTest {
                input: "5 * 5;",
                left_value: 5,
                op: "*",
                right_value: 5,
            },
            InfixTest {
                input: "5 / 5;",
                left_value: 5,
                op: "/",
                right_value: 5,
            },
            InfixTest {
                input: "5 > 5;",
                left_value: 5,
                op: ">",
                right_value: 5,
            },
            InfixTest {
                input: "5 < 5;",
                left_value: 5,
                op: "<",
                right_value: 5,
            },
            InfixTest {
                input: "5 == 5;",
                left_value: 5,
                op: "==",
                right_value: 5,
            },
            InfixTest {
                input: "5 != 5;",
                left_value: 5,
                op: "!=",
                right_value: 5,
            },
        ];

        for infix_test in infix_tests.iter() {
            let prog = parse_program_with_len(infix_test.input, 1);

            test_infix_expr(
                single_expression(&prog),
                infix_test.left_value,
                infix_test.op,
                infix_test.right_value,
            );
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        struct Test {
            pub input: &'static str,
            pub expected: &'static str,
        }

        let tests = [
            Test {
                input: "1 + 2 + 3",
                expected: "((1 + 2) + 3)",
            },
            Test {
                input: "-a * b",
                expected: "((-a) * b)",
            },
            Test {
                input: "!-a",
                expected: "(!(-a))",
            },
            Test {
                input: "a + b + c",
                expected: "((a + b) + c)",
            },
            Test {
                input: "a + b - c",
                expected: "((a + b) - c)",
            },
            Test {
                input: "a * b * c",
                expected: "((a * b) * c)",
            },
            Test {
                input: "a * b / c",
                expected: "((a * b) / c)",
            },
            Test {
                input: "a + b / c",
                expected: "(a + (b / c))",
            },
            Test {
                input: "a + b * c + d / e - f",
                expected: "(((a + (b * c)) + (d / e)) - f)",
            },
            Test {
                input: "3 + 4; -5 * 5",
                expected: "(3 + 4)((-5) * 5)",
            },
            Test {
                input: "5 > 4 == 3 < 4",
                expected: "((5 > 4) == (3 < 4))",
            },
            Test {
                input: "5 < 4 != 3 > 4",
                expected: "((5 < 4) != (3 > 4))",
            },
            Test {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5",
                expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            },
            Test {
                input: "true",
                expected: "true",
            },
            Test {
                input: "false",
                expected: "false",
            },
            Test {
                input: "3 > 5 == true",
                expected: "((3 > 5) == true)",
            },
            Test {
                input: "3 < 5 == false",
                expected: "((3 < 5) == false)",
            },
            Test {
                input: "1 + (2 + 3) + 4",
                expected: "((1 + (2 + 3)) + 4)",
            },
            Test {
                input: "(5 + 5) * 2",
                expected: "((5 + 5) * 2)",
            },
            Test {
                input: "2 / (5 + 5)",
                expected: "(2 / (5 + 5))",
            },
            Test {
                input: "-(5 + 5)",
                expected: "(-(5 + 5))",
            },
            Test {
                input: "!(true == true)",
                expected: "(!(true == true))",
            },
        ];

        for test in tests.iter() {
            let prog = parse_program_or_panic(test.input);

            let actual_str = prog.to_string();

            if actual_str != test.expected {
                panic!("expected {}, recieved {}", test.expected, actual_str)
            }
        }
    }

    #[test]
    fn test_boolean_expressions() {
        let input = "true";

        let prog = parse_program_with_len(input, 1);

        test_literal_expr(single_expression(&prog), true);
    }
}
