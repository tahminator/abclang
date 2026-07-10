use std::ops::Deref;

use crate::{
    ast::{Expression, Program, Statement},
    eval::error::EvaluateError,
    object::{IntegerObject, NullObject, Object, ObjectType, Objecter},
};

pub fn evaluate(program: &Program) -> Result<Object, EvaluateError> {
    eval_statements(&program.statements)
}

fn eval_statements(stmts: &[Statement]) -> Result<Object, EvaluateError> {
    let mut result = Object::Null(NullObject {});
    for stmt in stmts {
        result = eval_statement(stmt)?;
    }
    Ok(result)
}

fn eval_statement(stmt: &Statement) -> Result<Object, EvaluateError> {
    match stmt {
        Statement::Expression(stmt) => eval_expression(&stmt.expr),
        _ => Ok(Object::NULL),
    }
}

fn eval_expression(expr: &Expression) -> Result<Object, EvaluateError> {
    match expr {
        Expression::IntegerLiteral(expr) => {
            Ok(Object::Integer(IntegerObject { value: expr.value }))
        }
        Expression::Boolean(expr) => {
            if expr.value {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        Expression::Prefix(expr) => {
            let r = eval_expression(&expr.right)?;
            Ok(eval_prefix_expression(expr.op, r))
        }
        _ => Ok(Object::NULL),
    }
}

fn eval_prefix_expression(op: &str, r: Object) -> Object {
    match op {
        "!" => eval_bang_operator_expr(r),
        "-" => eval_minus_prefix_operator_expr(r),
        _ => Object::NULL,
    }
}

fn eval_bang_operator_expr(r: Object) -> Object {
    match r {
        Object::TRUE => Object::FALSE,
        Object::FALSE => Object::TRUE,
        Object::NULL => Object::TRUE,
        _ => Object::FALSE,
    }
}

fn eval_minus_prefix_operator_expr(r: Object) -> Object {
    let Object::Integer(r) = r else {
        return Object::NULL;
    };

    return Object::Integer(IntegerObject { value: -r.value });
}

#[cfg(test)]
mod tests {
    use super::*;

    mod testutils {
        use crate::{eval::evaluate, lexer::Lexer, object::Object, parser::Parser};

        pub fn test_eval(input: &str) -> Object {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer).unwrap();
            let prog = parser.parse_program().unwrap();

            evaluate(&prog).unwrap()
        }

        pub fn test_integer_obj(obj: Object, expected: i64) {
            let Object::Integer(obj) = obj else {
                panic!("expected integer object, received {obj:?}")
            };

            if obj.value != expected {
                panic!(
                    "object has wrong value - received {}, expected {expected}",
                    obj.value
                )
            }
        }

        pub fn test_boolean_obj(obj: Object, expected: bool) {
            let Object::Boolean(obj) = obj else {
                panic!("expected boolean object, received {obj:?}")
            };

            if obj.value != expected {
                panic!(
                    "object has wrong value - received {}, expected {expected}",
                    obj.value
                )
            }
        }
    }

    #[test]
    fn test_eval_integer_expression() {
        struct Test {
            input: &'static str,
            expected: i64,
        }

        let tests = [
            Test {
                input: "5",
                expected: 5,
            },
            Test {
                input: "10",
                expected: 10,
            },
            Test {
                input: "-5",
                expected: -5,
            },
            Test {
                input: "-10",
                expected: -10,
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);
            testutils::test_integer_obj(output, test.expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        struct Test {
            input: &'static str,
            expected: bool,
        }

        let tests = [
            Test {
                input: "true",
                expected: true,
            },
            Test {
                input: "false",
                expected: false,
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);
            testutils::test_boolean_obj(output, test.expected);
        }
    }

    #[test]
    fn test_bang_operator() {
        struct Test {
            input: &'static str,
            expected: bool,
        }

        let tests = [
            Test {
                input: "!true",
                expected: false,
            },
            Test {
                input: "!false",
                expected: true,
            },
            Test {
                input: "!5",
                expected: false,
            },
            Test {
                input: "!!true",
                expected: true,
            },
            Test {
                input: "!!false",
                expected: false,
            },
            Test {
                input: "!!5",
                expected: true,
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);
            testutils::test_boolean_obj(output, test.expected);
        }
    }
}
