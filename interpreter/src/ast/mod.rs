pub mod expression;
pub mod statement;

pub use expression::*;
pub use statement::*;

use std::{fmt, rc::Rc};

pub trait Node {
    fn token_literal(&self) -> Rc<str>;
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Node for Program {
    fn token_literal(&self) -> Rc<str> {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            "".into()
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{stmt}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Expression, IdentifierExpression, LetStatement},
        lexer::token::{Token, TokenType},
    };

    use super::*;

    #[test]
    fn test_to_string() {
        let prog = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    typ: TokenType::Let,
                    literal: "let".into(),
                }
                .into(),
                name: IdentifierExpression {
                    token: Token {
                        typ: TokenType::Ident,
                        literal: "myVar".into(),
                    }
                    .into(),
                    value: "myVar".into(),
                },
                value: Some(Expression::Identifier(IdentifierExpression {
                    token: Token {
                        typ: TokenType::Ident,
                        literal: "anotherVar".into(),
                    }
                    .into(),
                    value: "anotherVar".into(),
                })),
            })],
        };

        let s = prog.to_string();
        if s != "let myVar = anotherVar;" {
            panic!("program.to_string() wrong, got {}", s)
        }
    }

    // display_smoke
    macro_rules! display_smoke_test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                use crate::{lexer::Lexer, parser::Parser};

                let lexer = Lexer::new($input);
                let mut parser = Parser::new(lexer).unwrap();
                let program = parser.parse_program().unwrap();

                let got = program.to_string();
                assert_eq!(got, $expected, "input {:?}", $input);
            }
        };
    }

    display_smoke_test!(display_let_statement, "let x = 5;", "let x = 5;");
    display_smoke_test!(display_return_statement, "return 5;", "return 5");
    display_smoke_test!(display_expression_statement, "foobar;", "foobar");
    display_smoke_test!(display_assign_statement, "x = 5;", "x = 5;");
    display_smoke_test!(display_prefix_minus, "-5;", "(-5)");
    display_smoke_test!(display_prefix_bang, "!true;", "(!true)");
    display_smoke_test!(display_infix_expression, "5 + 5;", "(5 + 5)");
    display_smoke_test!(display_boolean_true, "true;", "true");
    display_smoke_test!(display_boolean_false, "false;", "false");
    display_smoke_test!(display_null_literal, "null;", "null");
    display_smoke_test!(display_float_literal, "3.5;", "3.5");
    display_smoke_test!(display_if_no_else, "if (x < y) { x }", "if (x < y) x");
    display_smoke_test!(
        display_if_else,
        "if (x < y) { x } else { y }",
        "if (x < y) x else y"
    );
    display_smoke_test!(display_for_expression, "for x in y { x }", "for x in y x");
    display_smoke_test!(
        display_fn_literal,
        "fn(x, y) { x + y; }",
        "fn(x, y) (x + y)"
    );
    display_smoke_test!(
        display_call_expression,
        "add(1, 2 * 3);",
        "add(1, (2 * 3))"
    );
    display_smoke_test!(display_string_expression, r#""hello";"#, "hello");
    display_smoke_test!(display_char_expression, "'a';", "a");
    display_smoke_test!(display_array_expression, "[1, 2 * 3];", "[1, (2 * 3)]");
    display_smoke_test!(
        display_index_expression,
        "myArray[1 + 1];",
        "(myArray[(1 + 1)])"
    );
    display_smoke_test!(display_hash_empty, "{};", "{}");
    display_smoke_test!(display_hash_single_pair, r#"{"a": 1};"#, "{a:1}");
    display_smoke_test!(display_dot_expression, "h.x;", "(h[x])");
    display_smoke_test!(
        display_block_multi_statement,
        "if (x) { 1; 2; }",
        "if x 1\n2"
    );
}
