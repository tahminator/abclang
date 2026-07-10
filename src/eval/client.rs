use std::ops::Deref;

use crate::{
    ast::{BlockStatement, Expression, IfExpression, Program, Statement},
    eval::error::EvaluateError,
    object::{
        ErrorObject, IntegerObject, NullObject, Object, ObjectType, Objecter, ReturnValueObject,
    },
};

pub fn evaluate(program: &Program) -> Result<Object, EvaluateError> {
    eval_program(&program.statements)
}

fn eval_program(stmts: &[Statement]) -> Result<Object, EvaluateError> {
    let mut result = Object::Null(NullObject {});
    for stmt in stmts {
        result = eval_statement(stmt)?;

        let cur_result = std::mem::replace(&mut result, Object::NULL);

        match cur_result {
            Object::ReturnValue(o) => {
                return Ok(*o.value);
            }
            Object::Error(o) => {
                return Ok(Object::Error(o));
            }
            _ => {
                result = cur_result;
            }
        }
    }
    Ok(result)
}

fn is_error_obj(o: &Object) -> bool {
    matches!(o, Object::Error(_))
}

fn eval_statement(stmt: &Statement) -> Result<Object, EvaluateError> {
    match stmt {
        Statement::Expression(stmt) => eval_expression(&stmt.expr),
        Statement::Block(stmt) => eval_block_statement(stmt),
        Statement::Return(stmt) => {
            let expr = eval_expression(
                stmt.value
                    .as_ref()
                    .ok_or(EvaluateError::ExpectedReturnButNoValueAttached)?,
            )?;
            if is_error_obj(&expr) {
                return Ok(expr);
            }

            let value = Box::new(expr);
            Ok(Object::ReturnValue(ReturnValueObject { value }))
        }
        _ => Ok(Object::NULL),
    }
}

fn eval_block_statement(block: &BlockStatement) -> Result<Object, EvaluateError> {
    let mut result = Object::Null(NullObject {});
    for stmt in block.statements.iter() {
        result = eval_statement(stmt)?;

        if matches!(result.typ(), ObjectType::ReturnValue | ObjectType::Error) {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_expression(expr: &Expression) -> Result<Object, EvaluateError> {
    match expr {
        Expression::If(expr) => eval_if_expression(expr),
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

            if is_error_obj(&r) {
                return Ok(r);
            }

            Ok(eval_prefix_expression(expr.op, r))
        }
        Expression::Infix(expr) => {
            let l = eval_expression(&expr.left)?;
            if is_error_obj(&l) {
                return Ok(l);
            }

            let r = eval_expression(&expr.right)?;
            if is_error_obj(&r) {
                return Ok(r);
            }
            Ok(eval_infix_expression(expr.op, l, r))
        }
        _ => Ok(Object::NULL),
    }
}

fn eval_prefix_expression(op: &str, r: Object) -> Object {
    match op {
        "!" => eval_bang_operator_expr(r),
        "-" => eval_minus_prefix_operator_expr(r),
        _ => Object::Error(ErrorObject {
            msg: format!("unknown operator: {op}{}", r.typ()),
        }),
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
        return Object::Error(ErrorObject {
            msg: format!("unknown operator: -{}", r.typ()),
        });
    };

    Object::Integer(IntegerObject { value: -r.value })
}

fn eval_if_expression(expr: &IfExpression<'_>) -> Result<Object, EvaluateError> {
    let cond = eval_expression(&expr.cond)?;

    match cond {
        _ if is_error_obj(&cond) => Ok(cond),
        _ if is_truthy(&cond) => {
            let Some(stmt) = &expr.consequence else {
                return Ok(Object::NULL);
            };

            eval_block_statement(stmt)
        }
        _ if expr.alternative.is_some() => {
            let Some(stmt) = &expr.alternative else {
                return Ok(Object::NULL);
            };

            eval_block_statement(stmt)
        }
        _ => Ok(Object::NULL),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match *obj {
        Object::NULL => false,
        Object::TRUE => true,
        Object::FALSE => false,
        _ => true,
    }
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
        (ol, or) if ol.typ() != or.typ() => Object::Error(ErrorObject {
            msg: format!("type mismatch: {} {op} {}", ol.typ(), or.typ()),
        }),
        (ol, or) => Object::Error(ErrorObject {
            msg: format!("unknown operator: {} {op} {}", ol.typ(), or.typ()),
        }),
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
        _ => Object::Error(ErrorObject {
            msg: format!("unknown operator: {} {op} {}", l.typ(), r.typ()),
        }),
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

        pub fn test_null_obj(obj: Object) {
            if !matches!(obj, Object::Null(_)) {
                panic!("expected null object, received {obj:?}")
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

    #[test]
    fn test_if_else_expressions() {
        enum Expected {
            Integer(i64),
            Null,
        }

        struct Test {
            input: &'static str,
            expected: Expected,
        }

        let tests = [
            Test {
                input: "if (true) { 10 }",
                expected: Expected::Integer(10),
            },
            Test {
                input: "if (false) { 10 }",
                expected: Expected::Null,
            },
            Test {
                input: "if (1) { 10 }",
                expected: Expected::Integer(10),
            },
            Test {
                input: "if (1 < 2) { 10 }",
                expected: Expected::Integer(10),
            },
            Test {
                input: "if (1 > 2) { 10 }",
                expected: Expected::Null,
            },
            Test {
                input: "if (1 > 2) { 10 } else { 20 }",
                expected: Expected::Integer(20),
            },
            Test {
                input: "if (1 < 2) { 10 } else { 20 }",
                expected: Expected::Integer(10),
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);
            match test.expected {
                Expected::Integer(expected) => testutils::test_integer_obj(output, expected),
                Expected::Null => testutils::test_null_obj(output),
            }
        }
    }

    #[test]
    fn test_return_statements() {
        struct Test {
            input: &'static str,
            expected: i64,
        }

        let tests = [
            Test {
                input: "return 10;",
                expected: 10,
            },
            Test {
                input: "return 10; 9;",
                expected: 10,
            },
            Test {
                input: "return 2 * 5; 9;",
                expected: 10,
            },
            Test {
                input: "9; return 2 * 5; 9;",
                expected: 10,
            },
            Test {
                input: "
                    if (10 > 1) {
                        if (10 > 1) {
                            return 10;
                        }
                        return 1;
                    }
",
                expected: 10,
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);
            testutils::test_integer_obj(output, test.expected);
        }
    }

    #[test]
    fn test_error_handling() {
        struct Test {
            input: &'static str,
            expected_message: &'static str,
        }

        let tests = [
            Test {
                input: "5 + true;",
                expected_message: "type mismatch: Integer + Boolean",
            },
            Test {
                input: "5 + true; 5;",
                expected_message: "type mismatch: Integer + Boolean",
            },
            Test {
                input: "-true",
                expected_message: "unknown operator: -Boolean",
            },
            Test {
                input: "true + false;",
                expected_message: "unknown operator: Boolean + Boolean",
            },
            Test {
                input: "5; true + false; 5",
                expected_message: "unknown operator: Boolean + Boolean",
            },
            Test {
                input: "if (10 > 1) { true + false; }",
                expected_message: "unknown operator: Boolean + Boolean",
            },
            Test {
                input: "if (10 > 1) {
                            if (10 > 1) {
                                return true + false;
                            }
                                return 1;
                        }
                ",
                expected_message: "unknown operator: Boolean + Boolean",
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);

            let Object::Error(output) = output else {
                panic!("expected error object, receieved {output:?}")
            };

            if output.msg != test.expected_message {
                panic!(
                    "expected {}, received {}",
                    test.expected_message, output.msg
                )
            }
        }
    }
}
