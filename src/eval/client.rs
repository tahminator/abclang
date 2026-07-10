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
        Expression::Infix(expr) => {
            let l = eval_expression(&expr.left)?;
            let r = eval_expression(&expr.right)?;
            Ok(eval_infix_expression(expr.op, l, r))
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

fn eval_infix_expression(op: &str, l: Object, r: Object) -> Object {
    match (l, r) {
        (Object::Integer(ol), Object::Integer(or)) => eval_integer_infix_expression(op, ol, or),
        (ol, or) if op == "==" => {
            if ol == or {
                Object::TRUE
            } else {
                Object::FALSE
            }
        }
        (ol, or) if op == "!=" => {
            if ol != or {
                Object::TRUE
            } else {
                Object::FALSE
            }
        }
        _ => Object::NULL,
    }
}

fn eval_integer_infix_expression(op: &str, l: IntegerObject, r: IntegerObject) -> Object {
    let lval = l.value;
    let rval = r.value;

    match op {
        "+" => Object::Integer(IntegerObject { value: lval + rval }),
        "-" => Object::Integer(IntegerObject { value: lval - rval }),
        "*" => Object::Integer(IntegerObject { value: lval * rval }),
        "/" => Object::Integer(IntegerObject { value: lval / rval }),
        "<" => {
            if lval < rval {
                Object::TRUE
            } else {
                Object::FALSE
            }
        }
        ">" => {
            if lval > rval {
                Object::TRUE
            } else {
                Object::FALSE
            }
        }
        "==" => {
            if lval == rval {
                Object::TRUE
            } else {
                Object::FALSE
            }
        }
        "!=" => {
            if lval != rval {
                Object::TRUE
            } else {
                Object::FALSE
            }
        }
        _ => Object::NULL,
    }
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
            Test {
                input: "5 + 5 + 5 + 5 - 10",
                expected: 10,
            },
            Test {
                input: "2 * 2 * 2 * 2 * 2",
                expected: 32,
            },
            Test {
                input: "-50 + 100 + -50",
                expected: 0,
            },
            Test {
                input: "5 * 2 + 10",
                expected: 20,
            },
            Test {
                input: "5 + 2 * 10",
                expected: 25,
            },
            Test {
                input: "20 + 2 * -10",
                expected: 0,
            },
            Test {
                input: "50 / 2 * 2 + 10",
                expected: 60,
            },
            Test {
                input: "2 * (5 + 10)",
                expected: 30,
            },
            Test {
                input: "3 * 3 * 3 + 10",
                expected: 37,
            },
            Test {
                input: "3 * (3 * 3) + 10",
                expected: 37,
            },
            Test {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
                expected: 50,
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
            Test {
                input: "1 < 2",
                expected: true,
            },
            Test {
                input: "1 > 2",
                expected: false,
            },
            Test {
                input: "1 < 1",
                expected: false,
            },
            Test {
                input: "1 > 1",
                expected: false,
            },
            Test {
                input: "1 == 1",
                expected: true,
            },
            Test {
                input: "1 != 1",
                expected: false,
            },
            Test {
                input: "1 == 2",
                expected: false,
            },
            Test {
                input: "1 != 2",
                expected: true,
            },
            Test {
                input: "true == true",
                expected: true,
            },
            Test {
                input: "false == false",
                expected: true,
            },
            Test {
                input: "true == false",
                expected: false,
            },
            Test {
                input: "true != false",
                expected: true,
            },
            Test {
                input: "false != true",
                expected: true,
            },
            Test {
                input: "(1 < 2) == true",
                expected: true,
            },
            Test {
                input: "(1 < 2) == false",
                expected: false,
            },
            Test {
                input: "(1 > 2) == true",
                expected: false,
            },
            Test {
                input: "(1 > 2) == false",
                expected: true,
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
