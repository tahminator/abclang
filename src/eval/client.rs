use std::rc::Rc;

use crate::{
    ast::{BlockStatement, Expression, IdentifierExpression, IfExpression, Program, Statement},
    eval::builtins::BUILTINS,
    object::{
        ErrorObject, FunctionObject, IntegerObject, NullObject, Object, ObjectType, Objecter,
        ReturnValueObject, StringObject,
        environment::{Env, Environment},
    },
};

pub fn evaluate(program: &Program, env: &Env) -> Result<Object, ErrorObject> {
    eval_program(&program.statements, env)
}

fn eval_program(stmts: &[Statement], env: &Env) -> Result<Object, ErrorObject> {
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

fn eval_statement(stmt: &Statement, env: &Env) -> Result<Object, ErrorObject> {
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

            env.borrow_mut().set(stmt.name.value.to_string(), val);

            Ok(Object::NULL)
        }
        _ => Ok(Object::NULL),
    }
}

fn eval_block_statement(block: &BlockStatement, env: &Env) -> Result<Object, ErrorObject> {
    let mut result = Object::Null(NullObject {});
    for stmt in block.statements.iter() {
        result = eval_statement(stmt, env)?;

        if matches!(result.typ(), ObjectType::ReturnValue | ObjectType::Error) {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_expression(expr: &Expression, env: &Env) -> Result<Object, ErrorObject> {
    match expr {
        Expression::If(expr) => eval_if_expression(expr, env),
        Expression::FnLiteral(expr) => {
            let params = expr.params.clone();
            let body = expr.body.clone();
            Ok(Object::Function(FunctionObject {
                params,
                body,
                env: env.clone(),
            }))
        }
        Expression::Call(expr) => {
            let func = eval_expression(&expr.function, env)?;

            let args = eval_expressions(&expr.args, env)?;

            apply_function(func, args)
        }
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

            eval_prefix_expression(expr.op.as_ref(), r)
        }
        Expression::Identifier(expr) => eval_identifier(expr, env),
        Expression::Infix(expr) => {
            let l = eval_expression(&expr.left, env)?;
            let r = eval_expression(&expr.right, env)?;
            eval_infix_expression(expr.op.as_ref(), l, r)
        }
        Expression::String(expr) => Ok(Object::String(StringObject {
            value: expr.value.clone(),
        })),
        _ => Ok(Object::NULL),
    }
}

fn apply_function(func: Object, args: Vec<Object>) -> Result<Object, ErrorObject> {
    match func {
        Object::Function(func) => {
            let body = func.body.clone().ok_or_else(|| ErrorObject {
                msg: "function body is empty when it should not be".to_string(),
            })?;

            let mut extended_env = extend_function_env(func, args)?;
            let output = eval_block_statement(&body, &mut extended_env)?;

            Ok(unwrap_return_value(output))
        }
        Object::BuiltIn(func) => (func.function)(&args),
        _ => Err(ErrorObject {
            msg: format!("not a function: {func:?}"),
        }),
    }
}

fn extend_function_env(func: FunctionObject, args: Vec<Object>) -> Result<Env, ErrorObject> {
    let env = Environment::new_enclosed(func.env);

    for (i, p) in func.params.iter().enumerate() {
        env.borrow_mut().set(p.value.to_string(), args.get(i).ok_or_else(|| ErrorObject {
            msg: "when extending function environment, attempting to find an original arg, but cannot find it.".to_string()
        })?.clone());
    }

    Ok(env)
}

fn unwrap_return_value(o: Object) -> Object {
    if let Object::ReturnValue(o) = o {
        *o.value
    } else {
        o
    }
}

fn eval_expressions(exprs: &Vec<Expression>, env: &Env) -> Result<Vec<Object>, ErrorObject> {
    let mut results = vec![];

    for e in exprs.iter() {
        let evald = eval_expression(e, env)?;

        results.push(evald);
    }

    Ok(results)
}

fn eval_identifier(expr: &IdentifierExpression, env: &Env) -> Result<Object, ErrorObject> {
    match env.borrow().get(expr.value.as_ref()) {
        Some(v) => Ok(v.clone()),
        None => match BUILTINS.get(expr.value.as_ref()) {
            Some(v) => Ok(Object::BuiltIn(v.clone())),
            None => Err(ErrorObject {
                msg: format!("identifier not found: {}", expr.value),
            }),
        },
    }
}

fn eval_prefix_expression(op: &str, r: Object) -> Result<Object, ErrorObject> {
    match op {
        "!" => Ok(eval_bang_operator_expr(r)),
        "-" => eval_minus_prefix_operator_expr(r),
        _ => Err(ErrorObject {
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

fn eval_minus_prefix_operator_expr(r: Object) -> Result<Object, ErrorObject> {
    let Object::Integer(r) = r else {
        return Err(ErrorObject {
            msg: format!("unknown operator: -{}", r.typ()),
        });
    };

    Ok(Object::Integer(IntegerObject { value: -r.value }))
}

fn eval_if_expression(expr: &IfExpression, env: &Env) -> Result<Object, ErrorObject> {
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

fn eval_infix_expression(op: &str, l: Object, r: Object) -> Result<Object, ErrorObject> {
    match (l, r) {
        (Object::Integer(ol), Object::Integer(or)) => eval_integer_infix_expression(op, ol, or),
        (Object::String(ol), Object::String(or)) => eval_string_infix_expression(op, ol, or),
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

fn eval_string_infix_expression(
    op: &str,
    l: StringObject,
    r: StringObject,
) -> Result<Object, ErrorObject> {
    match op {
        "+" => Ok(Object::String(StringObject {
            value: format!("{}{}", l.value, r.value).into(),
        })),
        _ => Err(ErrorObject {
            msg: format!("unknown operator: {} {op} {}", l.typ(), r.typ()),
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
            let env = Environment::new();

            evaluate(&prog, &env)
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
            Test {
                input: "\"hello\" - \"world\"",
                expected_message: "unknown operator: String - String",
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

    #[test]
    fn test_function_object() {
        let input = "fn(x) { x + 2; }";
        let output = testutils::test_eval(input).unwrap();

        let Object::Function(output) = output else {
            panic!("expected object function, received {output:?}")
        };

        if output.params.len() != 1 {
            panic!(
                "expected function params to be 1, received {}",
                output.params.len()
            )
        }

        let first_param = output.params.first().unwrap();
        if first_param.to_string() != "x" {
            panic!("expected \"x\", received {}", first_param)
        }

        if output.body.as_ref().unwrap().to_string() != "(x + 2)" {
            panic!(
                "expected \"(x + 2)\", recieved {}",
                output.body.as_ref().unwrap()
            )
        }
    }

    #[test]
    fn test_function_application() {
        struct Test {
            input: &'static str,
            expected: i64,
        }

        let tests = [
            Test {
                input: "let identity = fn(x) { x; }; identity(5);",
                expected: 5,
            },
            Test {
                input: "let identity = fn(x) { return x; }; identity(5);",
                expected: 5,
            },
            Test {
                input: "let double = fn(x) { x * 2; }; double(5);",
                expected: 10,
            },
            Test {
                input: "let add = fn(x, y) { x + y; }; add(5, 5);",
                expected: 10,
            },
            Test {
                input: "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                expected: 20,
            },
            Test {
                input: "fn(x) { x; }(5)",
                expected: 5,
            },
        ];

        for test in tests.iter() {
            testutils::test_integer_obj(testutils::test_eval(test.input).unwrap(), test.expected);
        }
    }

    #[test]
    fn test_closures() {
        let input = "
            let newAdder = fn(x) {
                fn(y) { x + y };
            };

            let addTwo = newAdder(2);
            addTwo(2);
        ";

        testutils::test_integer_obj(testutils::test_eval(input).unwrap(), 4);
    }

    #[test]
    fn test_string_literal() {
        let input = "\"hello world\"";

        let output = testutils::test_eval(input).unwrap();
        let Object::String(output) = output else {
            panic!("expected string object, recieved {output:?}")
        };

        if output.value.as_ref() != "hello world" {
            panic!("expected \"hello world\", received \"{}\"", output.value)
        }
    }

    #[test]
    fn test_string_concat() {
        let input = "\"hello\" + \" \" + \"world\"";

        let output = testutils::test_eval(input).unwrap();
        let Object::String(output) = output else {
            panic!("expected string object, recieved {output:?}")
        };

        if output.value.as_ref() != "hello world" {
            panic!("expected \"hello world\", received \"{}\"", output.value)
        }
    }

    #[test]
    fn test_builtin_functions() {
        enum Expected {
            String(String),
            Integer(i64),
        }
        struct Test {
            input: &'static str,
            expected: Expected,
        }

        let tests = [
            Test {
                input: "len(\"\")",
                expected: Expected::Integer(0),
            },
            Test {
                input: "len(\"four\")",
                expected: Expected::Integer(4),
            },
            Test {
                input: "len(\"hello world\")",
                expected: Expected::Integer(11),
            },
            Test {
                input: "len(1)",
                expected: Expected::String("argument to `len` not supported, got Integer".into()),
            },
            Test {
                input: "len(\"one\", \"two\")",
                expected: Expected::String("wrong number of arguments. got=2, want=1".into()),
            },
        ];

        for test in tests.iter() {
            match &test.expected {
                Expected::String(expected) => {
                    if let Err(e) = testutils::test_eval(test.input)
                        && e.msg != *expected
                    {
                        panic!("expected {expected}, received {}", e.msg)
                    }
                }
                Expected::Integer(i) => {
                    let output = testutils::test_eval(test.input).unwrap();
                    testutils::test_integer_obj(output, *i)
                }
            }
        }
    }
}
