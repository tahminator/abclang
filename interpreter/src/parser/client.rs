use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{
        self, ArrayExpression, BlockStatement, BooleanExpression, CallExpression, Expression,
        ExpressionStatement, FnLiteralExpression, ForExpression, HashExpression,
        IdentifierExpression, IfExpression, IndexExpression, InfixExpression,
        IntegerLiteralExpression, LetStatement, PrefixExpression, Program, ReturnStatement,
        Statement, StringExpression,
    },
    lexer::{Lexer, Token, TokenType},
    parser::{error::ParserError, precedence::Precedence},
};

type PrefixParseFn = fn(&mut Parser) -> Option<Expression>;
type InfixParseFn = fn(&mut Parser, expr: Expression) -> Option<Expression>;

pub struct Parser {
    lexer: Lexer,
    cur_token: Rc<Token>,
    peek_token: Rc<Token>,
    errors: Vec<ParserError>,

    prefix_parse_fns: HashMap<TokenType, PrefixParseFn>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Result<Self, ParserError> {
        let mut parser = Self {
            lexer,
            cur_token: Token::default().into(),
            peek_token: Token::default().into(),
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
        parser.register_prefix(TokenType::If, Parser::parse_if_expression);
        parser.register_prefix(TokenType::For, Parser::parse_for_expression);
        parser.register_prefix(TokenType::Function, Parser::parse_function_literal);
        parser.register_prefix(TokenType::String, Parser::parse_string_literal);
        parser.register_prefix(TokenType::LBracket, Parser::parse_array_literal);
        parser.register_prefix(TokenType::LBrace, Parser::parse_hash_literal);

        let infix_func = Parser::parse_infix_expression;
        parser.register_infix(TokenType::Plus, infix_func);
        parser.register_infix(TokenType::Minus, infix_func);
        parser.register_infix(TokenType::Slash, infix_func);
        parser.register_infix(TokenType::Asterisk, infix_func);
        parser.register_infix(TokenType::Eq, infix_func);
        parser.register_infix(TokenType::NotEq, infix_func);
        parser.register_infix(TokenType::Lt, infix_func);
        parser.register_infix(TokenType::Gt, infix_func);

        parser.register_infix(TokenType::LParen, Parser::parse_call_expression);
        parser.register_infix(TokenType::LBracket, Parser::parse_index_expression);

        parser.next_token();
        parser.next_token();

        Ok(parser)
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        Some(Expression::Identifier(IdentifierExpression {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_string_literal(&mut self) -> Option<Expression> {
        Some(Expression::String(StringExpression {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let mut pairs = HashMap::new();

        while !self.peek_token_is(TokenType::RBrace) {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;

            if !self.expect_peek(TokenType::Colon) {
                return None;
            }

            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.insert(key, value);

            if !self.peek_token_is(TokenType::RBrace) && !self.expect_peek(TokenType::Comma) {
                return None;
            }
        }

        if !self.expect_peek(TokenType::RBrace) {
            None
        } else {
            Some(Expression::Hash(HashExpression { token, pairs }))
        }
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenType::RBracket)?;

        Some(Expression::Array(ArrayExpression { token, elements }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        let value = match self.cur_token.literal.parse::<i64>() {
            Ok(v) => Some(v),
            Err(e) => {
                self.errors.push(e.into());
                None
            }
        }?;

        Some(Expression::IntegerLiteral(IntegerLiteralExpression {
            token,
            value,
        }))
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LParen) {
            return None;
        }

        let params = self.parse_function_params();

        if !self.expect_peek(TokenType::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expression::FnLiteral(FnLiteralExpression {
            token,
            params,
            body,
        }))
    }

    fn parse_boolean(&mut self) -> Option<Expression> {
        Some(Expression::Boolean(BooleanExpression {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();

        let exp = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        exp
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LParen) {
            return None;
        }

        self.next_token();
        let cond = self.parse_expression(Precedence::Lowest)?.into();

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        if !self.expect_peek(TokenType::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement();

        let mut alternative: Option<BlockStatement> = None;

        if self.peek_token_is(TokenType::Else) {
            self.next_token();

            if !self.expect_peek(TokenType::LBrace) {
                return None;
            }

            alternative = self.parse_block_statement();
        }

        Some(Expression::If(IfExpression {
            token,
            cond,
            consequence,
            alternative,
        }))
    }

    fn parse_for_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        let mut idents = vec![IdentifierExpression {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }];

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();

            if !self.expect_peek(TokenType::Ident) {
                return None;
            }

            idents.push(IdentifierExpression {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            });
        }

        if !self.expect_peek(TokenType::In) {
            return None;
        }

        self.next_token();
        let iterable = self.parse_expression(Precedence::Lowest)?.into();

        if !self.expect_peek(TokenType::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expression::For(ForExpression {
            token,
            idents,
            iterable,
            body,
        }))
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();

        self.next_token();

        Some(Expression::Prefix(PrefixExpression {
            op: token.literal.clone(),
            token,
            right: self.parse_expression(Precedence::Prefix)?.into(),
        }))
    }

    fn parse_infix_expression(&mut self, expr: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let left = expr.into();

        let precedence = self.cur_precedence();
        self.next_token();

        let right = self.parse_expression(precedence)?.into();

        Some(Expression::Infix(InfixExpression {
            left,
            op: token.literal.clone(),
            right,
            token,
        }))
    }

    fn parse_call_expression(&mut self, expr: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let function = expr.into();
        let args = self.parse_expression_list(TokenType::RParen)?;

        Some(Expression::Call(CallExpression {
            function,
            token,
            args,
        }))
    }

    fn parse_index_expression(&mut self, expr: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let left = expr.into();

        self.next_token();

        let index = self.parse_expression(Precedence::Lowest)?.into();

        if !self.expect_peek(TokenType::RBracket) {
            return None;
        }

        Some(Expression::Index(IndexExpression { token, left, index }))
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        match self.lexer.next_token() {
            Ok(tok) => self.peek_token = tok.into(),
            Err(e) => {
                self.errors.push(e.into());
                self.peek_token = Token::default().into();
            }
        }
    }

    fn peek_precedence(&self) -> Precedence {
        Precedence::lookup_precedence(self.peek_token.typ)
    }

    fn cur_precedence(&self) -> Precedence {
        Precedence::lookup_precedence(self.cur_token.typ)
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.typ {
            TokenType::Let => self.parse_let_statement().map(Statement::Let),
            TokenType::Return => self.parse_return_statement().map(Statement::Return),
            _ => self.parse_expression_statement().map(Statement::Expression),
        }
    }

    fn parse_expression_statement(&mut self) -> Option<ExpressionStatement> {
        let token = self.cur_token.clone();

        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ExpressionStatement { token, expr })
    }

    fn parse_function_params(&mut self) -> Vec<IdentifierExpression> {
        let mut idents = vec![];

        if self.peek_token_is(TokenType::RParen) {
            self.next_token();
            return idents;
        }

        self.next_token();

        idents.push(IdentifierExpression {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();

            idents.push(IdentifierExpression {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            });
        }

        if !self.expect_peek(TokenType::RParen) {
            return vec![];
        }

        idents
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Option<Vec<Expression>> {
        let mut list = vec![];

        if self.peek_token_is(end) {
            self.next_token();
            return Some(list);
        }

        self.next_token();
        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(end) {
            return None;
        }

        Some(list)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let Some(&prefix) = self.prefix_parse_fns.get(&self.cur_token.typ) else {
            self.errors.push(ParserError::NoPrefixParseFnFound {
                typ: self.cur_token.typ,
            });
            return None;
        };

        let mut left_expr = prefix(self)?;

        while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
            let Some(&infix) = self.infix_parse_fns.get(&self.peek_token.typ) else {
                return Some(left_expr);
            };

            self.next_token();

            left_expr = infix(self, left_expr)?;
        }

        Some(left_expr)
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let token = self.cur_token.clone();

        self.next_token();

        let value = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ReturnStatement { token, value })
    }

    // fn parse_call_arguments(&mut self) -> Vec<Expression> {
    //     let mut args = vec![];
    //
    //     if self.peek_token_is(TokenType::RParen) {
    //         self.next_token();
    //         return args;
    //     }
    //
    //     self.next_token();
    //     let Some(arg) = self.parse_expression(Precedence::Lowest) else {
    //         return args;
    //     };
    //     args.push(arg);
    //
    //     while self.peek_token_is(TokenType::Comma) {
    //         self.next_token();
    //         self.next_token();
    //         let Some(arg) = self.parse_expression(Precedence::Lowest) else {
    //             return args;
    //         };
    //         args.push(arg);
    //     }
    //
    //     if !self.expect_peek(TokenType::RParen) {
    //         return vec![];
    //     }
    //
    //     args
    // }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let token = self.cur_token.clone();
        let mut statements = vec![];

        self.next_token();

        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt)
            }
            self.next_token();
        }

        Some(BlockStatement { token, statements })
    }

    fn parse_let_statement(&mut self) -> Option<LetStatement> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        let name = IdentifierExpression {
            value: self.cur_token.literal.clone(),
            token: self.cur_token.clone(),
        };

        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        self.next_token();

        let value = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(LetStatement { name, token, value })
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

    pub fn parse_program(&mut self) -> Result<ast::Program, Vec<ParserError>> {
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

    fn register_prefix(&mut self, typ: TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(typ, func);
    }

    fn register_infix(&mut self, typ: TokenType, func: InfixParseFn) {
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

    fn parse_program_or_panic(input: &str) -> Program {
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

    fn parse_program_with_len(input: &str, expected_len: usize) -> Program {
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

    fn single_expression(prog: &Program) -> &Expression {
        let Statement::Expression(es) = &prog.statements[0] else {
            panic!(
                "expected expression statement, received {:?}",
                prog.statements[0]
            )
        };

        &es.expr
    }

    fn test_let_statement(s: &Statement, name: &str) {
        if s.token_literal().as_ref() != "let" {
            panic!(
                "expected s.token_literal to return \"let\", got {}",
                s.token_literal()
            )
        }

        let Statement::Let(ls) = s else {
            panic!("expected let statement but did not receive that")
        };

        if ls.name.value.as_ref() != name {
            panic!(
                "ls.name.value should have been {} but was {}",
                name, ls.name.value
            )
        }

        if ls.name.token_literal().as_ref() != name {
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

        if integ.token_literal().to_string() != v.to_string() {
            panic!(
                "integ.token_literal() expected to be {} but was {}",
                v,
                integ.token_literal()
            )
        }
    }

    #[derive(Clone, Copy)]
    enum Expected {
        Int(i64),
        Ident(&'static str),
        Bool(bool),
    }

    impl From<i64> for Expected {
        fn from(v: i64) -> Self {
            Expected::Int(v)
        }
    }

    impl From<&'static str> for Expected {
        fn from(v: &'static str) -> Self {
            Expected::Ident(v)
        }
    }

    impl From<bool> for Expected {
        fn from(v: bool) -> Self {
            Expected::Bool(v)
        }
    }

    fn test_identifier(expr: &Expression, value: &str) {
        let Expression::Identifier(ident) = expr else {
            panic!("expected identifier expression, received {:?}", expr)
        };

        if ident.value.as_ref() != value {
            panic!(
                "ident.value should have been {} but was {}",
                value, ident.value
            )
        }

        if ident.token_literal().as_ref() != value {
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

        if boolean.token_literal().to_string() != value.to_string() {
            panic!(
                "boolean.token_literal() should have been {} but was {}",
                value,
                boolean.token_literal()
            )
        }
    }

    fn test_literal_expr(expr: &Expression, expected: impl Into<Expected>) {
        match expected.into() {
            Expected::Int(v) => test_integer_literal(expr, v),
            Expected::Ident(v) => test_identifier(expr, v),
            Expected::Bool(v) => test_boolean_literal(expr, v),
        }
    }

    fn test_infix_expr(
        expr: &Expression,
        left: impl Into<Expected>,
        op: &str,
        right: impl Into<Expected>,
    ) {
        let Expression::Infix(infix) = expr else {
            panic!("expected infix expression, received {:?}", expr)
        };

        test_literal_expr(infix.left.as_ref(), left);

        if infix.op.as_ref() != op {
            panic!("expected operator {} but was {}", op, infix.op)
        }

        test_literal_expr(infix.right.as_ref(), right);
    }

    #[test]
    fn test_let_statements() {
        struct Test {
            input: &'static str,
            expected_identifier: &'static str,
            expected_value: Expected,
        }
        let tests = [
            Test {
                input: "let x = 5;",
                expected_identifier: "x",
                expected_value: Expected::Int(5),
            },
            Test {
                input: "let y = true;",
                expected_identifier: "y",
                expected_value: Expected::Bool(true),
            },
            Test {
                input: "let foobar = y;",
                expected_identifier: "foobar",
                expected_value: Expected::Ident("y"),
            },
        ];

        for test in tests.iter() {
            let prog = parse_program_with_len(test.input, 1);

            let stmt = prog.statements.first().unwrap();
            test_let_statement(stmt, test.expected_identifier);

            let Statement::Let(stmt) = stmt else {
                panic!("expected let statement, receieved {stmt:?}")
            };

            let val = stmt.value.clone().unwrap();
            test_literal_expr(&val, test.expected_value);
        }
    }

    #[test]
    fn test_return_statements() {
        struct Test {
            input: &'static str,
            expected_value: Expected,
        }
        let tests = [
            Test {
                input: "return 5;",
                expected_value: Expected::Int(5),
            },
            Test {
                input: "return true;",
                expected_value: Expected::Bool(true),
            },
            Test {
                input: "return foobar;",
                expected_value: Expected::Ident("foobar"),
            },
        ];

        for test in tests.iter() {
            let prog = parse_program_with_len(test.input, 1);

            let stmt = prog.statements.first().unwrap();

            let Statement::Return(stmt) = stmt else {
                panic!("expected return statement, receieved {stmt:?}")
            };

            let val = stmt.value.clone().unwrap();
            test_literal_expr(&val, test.expected_value);
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

            if expr.op.as_ref() != prefix_test.op {
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
            Test {
                input: "a + add(b * c) + d",
                expected: "((a + add((b * c))) + d)",
            },
            Test {
                input: "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                expected: "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            },
            Test {
                input: "add(a + b + c * d / f + g)",
                expected: "add((((a + b) + ((c * d) / f)) + g))",
            },
            Test {
                input: "a * [1, 2, 3, 4][b * c] * d",
                expected: "((a * ([1, 2, 3, 4][(b * c)])) * d)",
            },
            Test {
                input: "add(a * b[2], b[1], 2 * [1, 2][1])",
                expected: "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
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

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";
        let prog = parse_program_with_len(input, 1);

        let expr = single_expression(&prog);

        let Expression::If(if_expr) = expr else {
            panic!("expected if expression, received {:?}", expr)
        };

        test_infix_expr(if_expr.cond.as_ref(), "x", "<", "y");

        let Some(consequence) = &if_expr.consequence else {
            panic!("expected if_expr.consequence",)
        };

        let statement = consequence.statements.first().unwrap();

        let Statement::Expression(expr_stmt) = statement else {
            panic!("expected expression statement, received {:?}", statement)
        };

        test_identifier(&expr_stmt.expr, "x");
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";
        let prog = parse_program_with_len(input, 1);

        let expr = single_expression(&prog);

        let Expression::If(if_expr) = expr else {
            panic!("expected if expression, received {:?}", expr)
        };

        test_infix_expr(if_expr.cond.as_ref(), "x", "<", "y");

        let Some(consequence) = &if_expr.consequence else {
            panic!("expected if_expr.consequence",)
        };

        let statement = consequence.statements.first().unwrap();

        let Statement::Expression(expr_stmt) = statement else {
            panic!("expected expression statement, received {:?}", statement)
        };

        test_identifier(&expr_stmt.expr, "x");
    }

    #[test]
    fn test_for_expression() {
        let input = "for x in [1, 2, 3] { x }";
        let prog = parse_program_with_len(input, 1);

        let expr = single_expression(&prog);

        let Expression::For(for_expr) = expr else {
            panic!("expected for expression, received {:?}", expr)
        };

        if for_expr.idents.len() != 1 {
            panic!(
                "expected 1 for_expr.idents, received {}",
                for_expr.idents.len()
            )
        }

        test_literal_expr(
            &Expression::Identifier(for_expr.idents.first().unwrap().clone()),
            "x",
        );

        let Expression::Array(iterable) = for_expr.iterable.as_ref() else {
            panic!("expected array iterable, received {:?}", for_expr.iterable)
        };

        if iterable.elements.len() != 3 {
            panic!(
                "expected 3 iterable.elements, received {}",
                iterable.elements.len()
            )
        }

        let Some(body) = &for_expr.body else {
            panic!("expected for_expr.body")
        };

        let statement = body.statements.first().unwrap();

        let Statement::Expression(expr_stmt) = statement else {
            panic!("expected expression statement, received {:?}", statement)
        };

        test_identifier(&expr_stmt.expr, "x");
    }

    #[test]
    fn test_for_expression_multiple_idents() {
        let input = "for k, v in pairs { k }";
        let prog = parse_program_with_len(input, 1);

        let expr = single_expression(&prog);

        let Expression::For(for_expr) = expr else {
            panic!("expected for expression, received {:?}", expr)
        };

        if for_expr.idents.len() != 2 {
            panic!(
                "expected 2 for_expr.idents, received {}",
                for_expr.idents.len()
            )
        }

        for (i, v) in for_expr.idents.iter().enumerate() {
            test_literal_expr(
                &Expression::Identifier(v.clone()),
                if i == 0 { "k" } else { "v" },
            );
        }

        test_identifier(for_expr.iterable.as_ref(), "pairs");
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y) { x + y; }";
        let prog = parse_program_with_len(input, 1);

        let expr = single_expression(&prog);

        let Expression::FnLiteral(fn_literal_expr) = expr else {
            panic!("expected if expression, received {:?}", expr)
        };

        if fn_literal_expr.params.len() != 2 {
            panic!(
                "expected 2 fn_literal_expr.params, received {}",
                fn_literal_expr.params.len()
            )
        }

        for (i, v) in fn_literal_expr.params.iter().enumerate() {
            test_literal_expr(
                &Expression::Identifier(v.clone()),
                if i == 0 { "x" } else { "y" },
            );
        }

        let fn_body = fn_literal_expr.body.clone().unwrap();

        if fn_body.statements.len() != 1 {
            panic!(
                "fn_literal_expr.body.statements expected to be 1, received {}",
                fn_body.statements.len()
            )
        }

        let fn_body_stmt = fn_body.statements.first().unwrap();
        let Statement::Expression(body_stmt_expr) = fn_body_stmt else {
            panic!(
                "fn_body_stmt exepected to be ExpressionStatement, received {:?}",
                fn_body_stmt
            )
        };

        test_infix_expr(&body_stmt_expr.expr, "x", "+", "y");
    }

    #[test]
    fn test_function_param_parsing() {
        struct Test {
            input: &'static str,
            expected_params: Vec<&'static str>,
        }

        let tests = [
            Test {
                input: "fn() {};",
                expected_params: vec![],
            },
            Test {
                input: "fn(x) {};",
                expected_params: vec!["x"],
            },
            Test {
                input: "fn(x, y,z) {};",
                expected_params: vec!["x", "y", "z"],
            },
        ];

        for test in tests.iter() {
            let prog = parse_program_or_panic(test.input);

            let expr_stmt = prog.statements.first().unwrap();
            let Statement::Expression(expr_stmt) = expr_stmt else {
                panic!("expected expression statement, received {expr_stmt:?}")
            };

            let func = expr_stmt.expr.clone();
            let Expression::FnLiteral(func) = func else {
                panic!("expected function literal expression, received {func:?}")
            };

            if func.params.len() != test.expected_params.len() {
                panic!(
                    "expected {} params, received {}",
                    test.expected_params.len(),
                    func.params.len()
                )
            }

            for (i, ident) in test.expected_params.iter().enumerate() {
                test_literal_expr(
                    &Expression::Identifier(func.params.get(i).unwrap().clone()),
                    *ident,
                );
            }
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5)";

        let prog = parse_program_with_len(input, 1);

        let stmt = prog.statements.first().unwrap();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected expression statement, received {stmt:?}")
        };

        let exp = stmt.expr.clone();
        let Expression::Call(exp) = exp else {
            panic!("expected call expression, received {exp:?}")
        };

        test_identifier(&exp.function, "add");

        if exp.args.len() != 3 {
            panic!("expected exp.args to have 3, recieved {}", exp.args.len())
        }

        test_literal_expr(&exp.args.first().unwrap().clone(), 1);
        test_infix_expr(exp.args.get(1).unwrap(), 2, "*", 3);
        test_infix_expr(exp.args.get(2).unwrap(), 4, "+", 5);
    }

    #[test]
    fn test_string_literal_expression() {
        let input = "\"hello world\"";

        let prog = parse_program_with_len(input, 1);

        let stmt = prog.statements.first().unwrap().clone();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected statement expr, received {stmt:?}")
        };

        let Expression::String(expr) = stmt.expr else {
            panic!("expected statement expr, received {:?}", stmt.expr)
        };

        if expr.value.as_ref() != "hello world" {
            panic!("expected \"hello world\", receieved {}", expr.value)
        }
    }

    #[test]
    fn test_parsing_array_literals() {
        let input = "[1, 2 * 2, 3 + 3]";

        let prog = parse_program_with_len(input, 1);

        let stmt = prog.statements.first().unwrap().clone();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected statement expr, received {stmt:?}")
        };

        let Expression::Array(expr) = stmt.expr else {
            panic!("expected array expr, received {:?}", stmt.expr)
        };

        if expr.elements.len() != 3 {
            panic!(
                "expected 3 elements in array expr, receieved {}",
                expr.elements.len()
            )
        }

        test_integer_literal(expr.elements.first().unwrap(), 1);
        test_infix_expr(expr.elements.get(1).unwrap(), 2, "*", 2);
        test_infix_expr(expr.elements.get(2).unwrap(), 3, "+", 3);
    }

    #[test]
    fn test_parsing_index_expressions() {
        let input = "myArray[1 + 1]";

        let prog = parse_program_or_panic(input);

        let stmt = prog.statements.first().unwrap().clone();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected statement expr, received {stmt:?}")
        };

        let Expression::Index(expr) = stmt.expr else {
            panic!("expected index expr, received {:?}", stmt.expr)
        };

        test_identifier(&expr.left, "myArray");
        test_infix_expr(&expr.index, 1, "+", 1);
    }

    #[test]
    fn test_parsing_hash_literals_string_keys() {
        let input = r#"{"one": 1, "two": 2, "three": 3}"#;

        let prog = parse_program_with_len(input, 1);

        let stmt = prog.statements.first().unwrap().clone();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected statement expr, received {stmt:?}")
        };

        let Expression::Hash(expr) = stmt.expr else {
            panic!("expected hash expr, received {:?}", stmt.expr)
        };

        if expr.pairs.len() != 3 {
            panic!(
                "expected 3 pairs in hash expr, received {}",
                expr.pairs.len()
            )
        }

        let expected = HashMap::from([("one", 1), ("two", 2), ("three", 3)]);

        for (key, value) in &expr.pairs {
            let Expression::String(key) = key else {
                panic!("expected string key, received {key:?}")
            };

            let expected_value = expected
                .get(key.value.as_ref())
                .unwrap_or_else(|| panic!("unexpected key {}", key.value));
            test_integer_literal(value, *expected_value);
        }
    }

    #[test]
    fn test_parsing_empty_hash_literal() {
        let input = "{}";

        let prog = parse_program_with_len(input, 1);

        let stmt = prog.statements.first().unwrap().clone();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected statement expr, received {stmt:?}")
        };

        let Expression::Hash(expr) = stmt.expr else {
            panic!("expected hash expr, received {:?}", stmt.expr)
        };

        if !expr.pairs.is_empty() {
            panic!(
                "expected 0 pairs in hash expr, received {}",
                expr.pairs.len()
            )
        }
    }

    #[test]
    fn test_parsing_hash_literals_with_expressions() {
        let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;

        let prog = parse_program_with_len(input, 1);

        let stmt = prog.statements.first().unwrap().clone();
        let Statement::Expression(stmt) = stmt else {
            panic!("expected statement expr, received {stmt:?}")
        };

        let Expression::Hash(expr) = stmt.expr else {
            panic!("expected hash expr, received {:?}", stmt.expr)
        };

        if expr.pairs.len() != 3 {
            panic!(
                "expected 3 pairs in hash expr, received {}",
                expr.pairs.len()
            )
        }

        let tests: HashMap<&str, (i64, &str, i64)> = HashMap::from([
            ("one", (0, "+", 1)),
            ("two", (10, "-", 8)),
            ("three", (15, "/", 5)),
        ]);

        for (key, value) in &expr.pairs {
            let Expression::String(key) = key else {
                panic!("expected string key, received {key:?}")
            };

            let (left, op, right) = tests
                .get(key.value.as_ref())
                .unwrap_or_else(|| panic!("no test found for key {}", key.value));
            test_infix_expr(value, *left, op, *right);
        }
    }
}
