use std::ops::Deref;

use crate::{
    ast::{BlockStatement, Expression, IdentifierExpression, IfExpression, Program, Statement},
    object::{
        ErrorObject, IntegerObject, NullObject, Object, ObjectType, Objecter, ReturnValueObject,
        environment::Environment,
    },
};

pub fn evaluate<'a>(
    program: &Program<'a>,
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    eval_program(&program.statements, env)
}

fn eval_program<'a>(
    stmts: &[Statement<'a>],
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    let mut result = Object::Null(NullObject {});
    for stmt in stmts {
        result = eval_statement(stmt, env)?;

        let cur_result = std::mem::replace(&mut result, Object::NULL);

        match cur_result {
            Object::ReturnValue(o) => {
                return Ok(*o.value);
            }
            _ => {
                result = cur_result;
            }
        }
    }
    Ok(result)
}

fn eval_statement<'a>(
    stmt: &Statement<'a>,
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    match stmt {
        Statement::Expression(stmt) => eval_expression(&stmt.expr, env),
        Statement::Block(stmt) => eval_block_statement(stmt, env),
        Statement::Return(stmt) => {
            let expr = eval_expression(
                stmt.value.as_ref().ok_or(ErrorObject {
                    msg: "expected return but no value attached".to_string(),
                })?,
                env,
            )?;

            let value = Box::new(expr);
            Ok(Object::ReturnValue(ReturnValueObject { value }))
        }
        Statement::Let(stmt) => {
            let val = eval_expression(
                stmt.value.as_ref().ok_or(ErrorObject {
                    msg: "expected return but no value attached".to_string(),
                })?,
                env,
            )?;

            env.set(stmt.name.value.to_string(), val);

            Ok(Object::NULL)
        }
        _ => Ok(Object::NULL),
    }
}

fn eval_block_statement<'a>(
    block: &BlockStatement<'a>,
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    let mut result = Object::Null(NullObject {});
    for stmt in block.statements.iter() {
        result = eval_statement(stmt, env)?;

        if matches!(result.typ(), ObjectType::ReturnValue | ObjectType::Error) {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_expression<'a>(
    expr: &Expression<'a>,
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    match expr {
        Expression::If(expr) => eval_if_expression(expr, env),
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
            let r = eval_expression(&expr.right, env)?;

            eval_prefix_expression(expr.op, r)
        }
        Expression::Identifier(expr) => eval_identifier(expr, env),
        Expression::Infix(expr) => {
            let l = eval_expression(&expr.left, env)?;
            let r = eval_expression(&expr.right, env)?;
            eval_infix_expression(expr.op, l, r)
        }
        _ => Ok(Object::NULL),
    }
}

fn eval_identifier<'a>(
    expr: &IdentifierExpression<'a>,
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    match env.get(expr.value) {
        Some(v) => Ok(v.clone()),
        None => Err(ErrorObject {
            msg: format!("identifier not found: {}", expr.value),
        }),
    }
}

fn eval_prefix_expression<'a>(op: &'a str, r: Object<'a>) -> Result<Object<'a>, ErrorObject> {
    match op {
        "!" => Ok(eval_bang_operator_expr(r)),
        "-" => eval_minus_prefix_operator_expr(r),
        _ => Err(ErrorObject {
            msg: format!("unknown operator: {op}{}", r.typ()),
        }),
    }
}

fn eval_bang_operator_expr<'a>(r: Object<'a>) -> Object<'a> {
    match r {
        Object::TRUE => Object::FALSE,
        Object::FALSE => Object::TRUE,
        Object::NULL => Object::TRUE,
        _ => Object::FALSE,
    }
}

fn eval_minus_prefix_operator_expr<'a>(r: Object<'a>) -> Result<Object<'a>, ErrorObject> {
    let Object::Integer(r) = r else {
        return Err(ErrorObject {
            msg: format!("unknown operator: -{}", r.typ()),
        });
    };

    Ok(Object::Integer(IntegerObject { value: -r.value }))
}

fn eval_if_expression<'a>(
    expr: &IfExpression<'a>,
    env: &mut Environment<'a>,
) -> Result<Object<'a>, ErrorObject> {
    let cond = eval_expression(&expr.cond, env)?;

    match cond {
        _ if is_truthy(&cond) => {
            let Some(stmt) = &expr.consequence else {
                return Ok(Object::NULL);
            };

            eval_block_statement(stmt, env)
        }
        _ if expr.alternative.is_some() => {
            let Some(stmt) = &expr.alternative else {
                return Ok(Object::NULL);
            };

            eval_block_statement(stmt, env)
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

fn eval_infix_expression<'a>(
    op: &'a str,
    l: Object<'a>,
    r: Object<'a>,
) -> Result<Object<'a>, ErrorObject> {
    match (l, r) {
        (Object::Integer(ol), Object::Integer(or)) => eval_integer_infix_expression(op, ol, or),
        (ol, or) if op == "==" => Ok(if ol == or {
            Object::TRUE
        } else {
            Object::FALSE
        }),
        (ol, or) if op == "!=" => Ok(if ol != or {
            Object::TRUE
        } else {
            Object::FALSE
        }),
        (ol, or) if ol.typ() != or.typ() => Err(ErrorObject {
            msg: format!("type mismatch: {} {op} {}", ol.typ(), or.typ()),
        }),
        (ol, or) => Err(ErrorObject {
            msg: format!("unknown operator: {} {op} {}", ol.typ(), or.typ()),
        }),
    }
}

fn eval_integer_infix_expression(
    op: &str,
    l: IntegerObject,
    r: IntegerObject,
) -> Result<Object, ErrorObject> {
    let lval = l.value;
    let rval = r.value;

    match op {
        "+" => Ok(Object::Integer(IntegerObject { value: lval + rval })),
        "-" => Ok(Object::Integer(IntegerObject { value: lval - rval })),
        "*" => Ok(Object::Integer(IntegerObject { value: lval * rval })),
        "/" => Ok(Object::Integer(IntegerObject { value: lval / rval })),
        "<" => {
            if lval < rval {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        ">" => {
            if lval > rval {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        "==" => {
            if lval == rval {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        "!=" => {
            if lval != rval {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        _ => Err(ErrorObject {
            msg: format!("unknown operator: {} {op} {}", l.typ(), r.typ()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::client::tests::testutils::{test_eval, test_integer_obj};

    use super::*;

    mod testutils {
        use crate::{
            eval::evaluate,
            lexer::Lexer,
            object::{ErrorObject, Object, environment::Environment},
            parser::Parser,
        };

        pub fn test_eval(input: &str) -> Result<Object, ErrorObject> {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer).unwrap();
            let prog = parser.parse_program().unwrap();
            let mut env = Environment::default();

            evaluate(&prog, &mut env)
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
            let output = testutils::test_eval(test.input).unwrap();
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
            let output = testutils::test_eval(test.input).unwrap();
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
            let output = testutils::test_eval(test.input).unwrap();
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
            let output = testutils::test_eval(test.input).unwrap();
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
            let output = testutils::test_eval(test.input).unwrap();
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
            Test {
                input: "foobar",
                expected_message: "identifier not found: foobar",
            },
        ];

        for test in tests.iter() {
            let output = testutils::test_eval(test.input);

            let Err(output) = output else {
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

    #[test]
    fn test_let_statements() {
        struct Test {
            input: &'static str,
            expected: i64,
        }

        let tests = [
            Test {
                input: "let a = 5; a;",
                expected: 5,
            },
            Test {
                input: "let a = 5 * 5; a;",
                expected: 25,
            },
            Test {
                input: "let a = 5; let b = a; b;",
                expected: 5,
            },
            Test {
                input: "let a = 5; let b = a; let c = a + b + 5; c;",
                expected: 15,
            },
        ];

        for test in tests.iter() {
            test_integer_obj(test_eval(test.input).unwrap(), test.expected);
        }
    }
}
