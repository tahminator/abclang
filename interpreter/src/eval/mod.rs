pub mod builtins;
pub mod object;

use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{
        BlockStatement, Expression, ForExpression, HashExpression, IdentifierExpression,
        IfExpression, Program, Statement,
    },
    eval::{
        builtins::BUILTINS,
        object::{
            ArrayObject, CharObject, ClassObject, ErrorObject, FloatObject, FunctionObject,
            HashObject, IntegerObject, NullObject, Object, ObjectHasher, ObjectType, Objecter,
            ReturnValueObject, StringObject,
            environment::{Env, Environment},
        },
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

            let name = stmt.name.value.clone();

            let mut env = env.try_borrow_mut().map_err(|e| ErrorObject {
                msg: format!("internal eval error: could not borrow env due to: {e:#?}"),
            })?;

            match env.get_in_scope(&name) {
                Some(_) => Err(ErrorObject {
                    msg: format!(
                        "{name} already exists, you may reassign it's value instead by removing the `let` keyword"
                    ),
                }),
                None => {
                    env.set(name.to_string(), val);
                    Ok(Object::NULL)
                }
            }
        }
        Statement::Class(stmt) => {
            let name = stmt.name.value.clone();

            let class = Object::Class(ClassObject {
                name: name.clone(),
                body: stmt.body.clone(),
                env: env.clone(),
            });

            let mut env = env.try_borrow_mut().map_err(|e| ErrorObject {
                msg: format!("internal eval error: could not borrow env due to: {e:#?}"),
            })?;

            match env.get_in_scope(&name) {
                Some(_) => Err(ErrorObject {
                    msg: format!("{name} already exists, choose a different class name"),
                }),
                None => {
                    env.set(name.to_string(), class);
                    Ok(Object::NULL)
                }
            }
        }
        Statement::Assign(stmt) => {
            let val = eval_expression(&stmt.value, env)?;

            match &stmt.target {
                Expression::Identifier(ident) => {
                    if env.borrow_mut().assign(&ident.value, val) {
                        Ok(Object::NULL)
                    } else {
                        Err(ErrorObject {
                            msg: format!("identifier not found: {}", ident.value),
                        })
                    }
                }
                Expression::Index(idx) => {
                    let left = eval_expression(&idx.left, env)?;
                    let index = eval_expression(&idx.index, env)?;

                    eval_index_assign(&left, &index, val)?;
                    Ok(Object::NULL)
                }
                other => Err(ErrorObject {
                    msg: format!("invalid assignment target: {other}"),
                }),
            }
        }
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

#[allow(unreachable_patterns)]
fn eval_expression(expr: &Expression, env: &Env) -> Result<Object, ErrorObject> {
    match expr {
        Expression::If(expr) => eval_if_expression(expr, env),
        Expression::For(expr) => eval_for_expression(expr, env),
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

            apply_function(func, args, env)
        }
        Expression::IntegerLiteral(expr) => {
            Ok(Object::Integer(IntegerObject { value: expr.value }))
        }
        Expression::FloatLiteral(expr) => Ok(Object::Float(FloatObject {
            value: expr.value.into(),
        })),
        Expression::Boolean(expr) => {
            if expr.value {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        Expression::NullLiteral(_) => Ok(Object::NULL),
        Expression::Hash(expr) => eval_hash_literal(expr, env),
        Expression::Prefix(expr) => {
            let r = eval_expression(&expr.right, env)?;

            eval_prefix_expression(expr.op.as_ref(), r)
        }
        Expression::Array(expr) => {
            let elements = eval_expressions(&expr.elements, env)?;

            Ok(Object::Array(ArrayObject::new(elements)))
        }
        Expression::Identifier(expr) => eval_identifier(expr, env),
        Expression::Index(expr) => {
            let left = eval_expression(&expr.left, env)?;
            let index = eval_expression(&expr.index, env)?;

            eval_index_expression(&left, &index)
        }
        Expression::Infix(infix) if infix.op.as_ref() == "&&" || infix.op.as_ref() == "||" => {
            let left = eval_expression(&infix.left, env)?;
            match infix.op.as_ref() {
                "&&" if !is_truthy(&left) => Ok(left),
                "&&" => eval_expression(&infix.right, env),
                "||" if is_truthy(&left) => Ok(left),
                "||" => eval_expression(&infix.right, env),
                _ => unreachable!(),
            }
        }
        Expression::Infix(expr) => {
            let l = eval_expression(&expr.left, env)?;
            let r = eval_expression(&expr.right, env)?;
            eval_infix_expression(expr.op.as_ref(), l, r)
        }
        Expression::String(expr) => Ok(Object::String(StringObject {
            value: expr.value.clone(),
        })),
        Expression::Char(expr) => Ok(Object::Char(CharObject {
            value: expr.value.clone(),
        })),
        _ => Ok(Object::NULL),
    }
}

fn apply_function(func: Object, args: Vec<Object>, env: &Env) -> Result<Object, ErrorObject> {
    match func {
        Object::Function(func) => {
            let body = func.body.clone().ok_or_else(|| ErrorObject {
                msg: "function body is empty when it should not be".to_string(),
            })?;

            let extended_env = extend_function_env(func, args)?;
            let output = eval_block_statement(&body, &extended_env)?;

            Ok(unwrap_return_value(output))
        }
        Object::BuiltIn(func) => (func.function)(&args, env),
        _ => Err(ErrorObject {
            msg: format!("not a function: {func:?}"),
        }),
    }
}

fn eval_hash_literal(expr: &HashExpression, env: &Env) -> Result<Object, ErrorObject> {
    let mut pairs = HashMap::new();

    for (k, v) in expr.pairs.iter() {
        let key = eval_expression(k, env)?;

        let value = eval_expression(v, env)?;

        let hashed = key.hash_key().ok_or_else(|| ErrorObject {
            msg: format!("{} is unusable as a hash key", key.typ()),
        })?;

        pairs.insert(hashed, (key, value));
    }

    Ok(Object::Hash(HashObject::new(pairs)))
}

fn eval_index_expression(left: &Object, index: &Object) -> Result<Object, ErrorObject> {
    match (left, index) {
        (Object::Array(left), Object::Integer(index)) => eval_array_index_expression(left, index),
        (Object::String(left), Object::Integer(index)) => eval_string_index_expression(left, index),
        (Object::Hash(left), index) => eval_hash_index_expression(left, index),
        _ => Err(ErrorObject {
            msg: format!("index operator not supported: {}", left.typ()),
        }),
    }
}

fn eval_index_assign(left: &Object, index: &Object, value: Object) -> Result<(), ErrorObject> {
    match (left, index) {
        (Object::Array(arr), Object::Integer(idx)) => {
            let mut elements = arr.elements.try_borrow_mut()?;
            let len = elements.len();

            let i = idx.value;
            let slot = elements.get_mut(i as usize).ok_or_else(|| ErrorObject {
                msg: format!(
                    "index {i} out of bounds for Array of length {len}, use `push` to grow"
                ),
            })?;

            *slot = value;

            Ok(())
        }
        (Object::Array(_), index) => Err(ErrorObject {
            msg: format!("array index must be an Integer, got {}", index.typ()),
        }),
        (Object::Hash(hash), key) => {
            let hashed = key.hash_key().ok_or_else(|| ErrorObject {
                msg: format!("{} is unusable as a hash key", key.typ()),
            })?;

            hash.pairs
                .try_borrow_mut()?
                .insert(hashed, (key.clone(), value));

            Ok(())
        }
        _ => Err(ErrorObject {
            msg: format!("index assignment not supported: {}", left.typ()),
        }),
    }
}

fn eval_hash_index_expression(left: &HashObject, index: &Object) -> Result<Object, ErrorObject> {
    let Some(key) = index.hash_key() else {
        return Err(ErrorObject {
            msg: format!("{} is unusable as a hash key", index.typ()),
        });
    };

    Ok(match left.pairs.try_borrow()?.get(&key) {
        Some((_, v)) => v.clone(),
        None => Object::NULL,
    })
}

fn eval_string_index_expression(
    string_obj: &StringObject,
    index_obj: &IntegerObject,
) -> Result<Object, ErrorObject> {
    Ok(Object::Char(CharObject {
        value: Rc::from(
            string_obj
                .clone()
                .value
                .chars()
                .nth(index_obj.value as usize)
                .ok_or_else(|| ErrorObject {
                    msg: format!(
                        "{} is not a valid index on a string with length of {}",
                        index_obj.value,
                        string_obj.value.len()
                    ),
                })?,
        ),
    }))
}

fn eval_array_index_expression(
    array_obj: &ArrayObject,
    index_obj: &IntegerObject,
) -> Result<Object, ErrorObject> {
    Ok(array_obj
        .elements
        .try_borrow()?
        .get(index_obj.value as usize)
        .cloned()
        .unwrap_or(Object::NULL))
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

fn eval_expressions(exprs: &[Expression], env: &Env) -> Result<Vec<Object>, ErrorObject> {
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
    match r {
        Object::Integer(r) => Ok(Object::Integer(IntegerObject { value: -r.value })),
        Object::Float(r) => Ok(Object::Float(FloatObject { value: -r.value })),
        _ => Err(ErrorObject {
            msg: format!("unknown operator: -{}", r.typ()),
        }),
    }
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

fn eval_for_expression(expr: &ForExpression, env: &Env) -> Result<Object, ErrorObject> {
    let iterable_obj = eval_expression(&expr.iterable, env)?;

    let Some(body) = &expr.body else {
        return Ok(Object::NULL);
    };

    let rows: Vec<Vec<Object>> = match &iterable_obj {
        Object::Array(arr) => {
            if expr.idents.len() != 1 {
                return Err(ErrorObject {
                    msg: format!(
                        "for loop over Array expects 1 variable, got {}",
                        expr.idents.len()
                    ),
                });
            }

            arr.elements
                .try_borrow()?
                .iter()
                .map(|el| vec![el.clone()])
                .collect()
        }
        Object::String(s) => {
            if expr.idents.len() != 1 {
                return Err(ErrorObject {
                    msg: format!(
                        "for loop over String expected 1 variable, got {}",
                        expr.idents.len()
                    ),
                });
            }

            s.value
                .chars()
                .map(|c| vec![Object::Char(CharObject { value: Rc::from(c) })])
                .collect::<Vec<_>>()
        }
        Object::Hash(hash) => {
            let with_value = match expr.idents.len() {
                1 => false,
                2 => true,
                n => {
                    return Err(ErrorObject {
                        msg: format!("for loop over Hash expects 1 or 2 variables, got {n}"),
                    });
                }
            };

            hash.pairs
                .try_borrow()?
                .values()
                .map(|(key, value)| {
                    if with_value {
                        vec![key.clone(), value.clone()]
                    } else {
                        vec![key.clone()]
                    }
                })
                .collect()
        }
        other => {
            return Err(ErrorObject {
                msg: format!("{} is not iterable", other.typ()),
            });
        }
    };

    for row in rows {
        let loop_env = Environment::new_enclosed(env.clone());

        for (ident, value) in expr.idents.iter().zip(row) {
            loop_env.borrow_mut().set(ident.value.to_string(), value);
        }

        let result = eval_block_statement(body, &loop_env)?;

        if matches!(result, Object::ReturnValue(_)) {
            return Ok(result);
        }
    }

    Ok(Object::NULL)
}

fn is_truthy(obj: &Object) -> bool {
    match *(obj) {
        Object::NULL => false,
        Object::TRUE => true,
        Object::FALSE => false,
        _ => true,
    }
}

fn eval_infix_expression(op: &str, l: Object, r: Object) -> Result<Object, ErrorObject> {
    match (l, r) {
        (Object::Integer(ol), Object::Integer(or)) => eval_integer_infix_expression(op, ol, or),
        (Object::Float(ol), Object::Float(or)) => eval_float_infix_expression(op, ol, or),
        (Object::Integer(ol), Object::Float(or)) => eval_float_infix_expression(
            op,
            FloatObject {
                value: ol.value as f64,
            },
            or,
        ),
        (Object::Float(ol), Object::Integer(or)) => eval_float_infix_expression(
            op,
            ol,
            FloatObject {
                value: or.value as f64,
            },
        ),
        (Object::String(ol), Object::String(or)) => eval_string_infix_expression(op, ol, or),
        (ol, Object::NULL) if op == "==" => Ok(if ol == Object::NULL {
            Object::TRUE
        } else {
            Object::FALSE
        }),
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
        (ol, Object::NULL) => Ok(if ol == Object::NULL {
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
        "==" => Ok(if l.value == r.value {
            Object::TRUE
        } else {
            Object::FALSE
        }),
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
        "<=" => {
            if lval <= rval {
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
        ">=" => {
            if lval >= rval {
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

fn eval_float_infix_expression(
    op: &str,
    l: FloatObject,
    r: FloatObject,
) -> Result<Object, ErrorObject> {
    let lval = l.value;
    let rval = r.value;

    match op {
        "+" => Ok(Object::Float(FloatObject { value: lval + rval })),
        "-" => Ok(Object::Float(FloatObject { value: lval - rval })),
        "*" => Ok(Object::Float(FloatObject { value: lval * rval })),
        "/" => Ok(Object::Float(FloatObject { value: lval / rval })),
        "<" => {
            if lval < rval {
                Ok(Object::TRUE)
            } else {
                Ok(Object::FALSE)
            }
        }
        "<=" => {
            if lval <= rval {
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
        ">=" => {
            if lval >= rval {
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
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn run(input: &str) -> String {
        let env = Environment::new();
        let lexer = Lexer::new(input);

        let mut parser = match Parser::new(lexer) {
            Ok(parser) => parser,
            Err(err) => return format!("lexer/parser error: {err}"),
        };

        match parser.parse_program() {
            Ok(program) => {
                let result = evaluate(&program, &env);
                let mut out = env.borrow().take_output();

                match result {
                    Ok(Object::Null(_)) => {}
                    Ok(obj) => out.push_str(&obj.inspect_value()),
                    Err(err) => out.push_str(&err.inspect_value()),
                }

                out
            }
            Err(errors) => {
                let errs = errors
                    .iter()
                    .map(|err| format!("\t{err}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("parser has {} error(s):\n{}", errors.len(), errs)
            }
        }
    }

    struct Case {
        name: &'static str,
        input: &'static str,
        output: &'static str,
    }

    const CASES: &[Case] = &[
        // integers
        Case {
            name: "int_literal_positive",
            input: "5",
            output: "5",
        },
        Case {
            name: "int_literal_positive_10",
            input: "10",
            output: "10",
        },
        Case {
            name: "int_prefix_negative",
            input: "-5",
            output: "-5",
        },
        Case {
            name: "int_prefix_negative_10",
            input: "-10",
            output: "-10",
        },
        Case {
            name: "int_add_sub_chain",
            input: "5 + 5 + 5 + 5 - 10",
            output: "10",
        },
        Case {
            name: "int_mul_chain",
            input: "2 * 2 * 2 * 2 * 2",
            output: "32",
        },
        Case {
            name: "int_negatives_sum",
            input: "-50 + 100 + -50",
            output: "0",
        },
        Case {
            name: "int_mul_before_add",
            input: "5 * 2 + 10",
            output: "20",
        },
        Case {
            name: "int_add_before_mul",
            input: "5 + 2 * 10",
            output: "25",
        },
        Case {
            name: "int_mixed_precedence",
            input: "20 + 2 * -10",
            output: "0",
        },
        Case {
            name: "int_div_mul_add",
            input: "50 / 2 * 2 + 10",
            output: "60",
        },
        Case {
            name: "int_paren_mul",
            input: "2 * (5 + 10)",
            output: "30",
        },
        Case {
            name: "int_triple_mul_add",
            input: "3 * 3 * 3 + 10",
            output: "37",
        },
        Case {
            name: "int_paren_triple_mul_add",
            input: "3 * (3 * 3) + 10",
            output: "37",
        },
        Case {
            name: "int_complex_expr",
            input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
            output: "50",
        },
        // booleans
        Case {
            name: "bool_true",
            input: "true",
            output: "true",
        },
        Case {
            name: "bool_false",
            input: "false",
            output: "false",
        },
        Case {
            name: "bool_lt_true",
            input: "1 < 2",
            output: "true",
        },
        Case {
            name: "bool_gt_false",
            input: "1 > 2",
            output: "false",
        },
        Case {
            name: "bool_lt_eq_false",
            input: "1 < 1",
            output: "false",
        },
        Case {
            name: "bool_gt_eq_false",
            input: "1 > 1",
            output: "false",
        },
        Case {
            name: "bool_eq_true",
            input: "1 == 1",
            output: "true",
        },
        Case {
            name: "bool_neq_false",
            input: "1 != 1",
            output: "false",
        },
        Case {
            name: "bool_eq_false",
            input: "1 == 2",
            output: "false",
        },
        Case {
            name: "bool_neq_true",
            input: "1 != 2",
            output: "true",
        },
        Case {
            name: "bool_true_eq_true",
            input: "true == true",
            output: "true",
        },
        Case {
            name: "bool_false_eq_false",
            input: "false == false",
            output: "true",
        },
        Case {
            name: "bool_true_eq_false",
            input: "true == false",
            output: "false",
        },
        Case {
            name: "bool_true_neq_false",
            input: "true != false",
            output: "true",
        },
        Case {
            name: "bool_false_neq_true",
            input: "false != true",
            output: "true",
        },
        Case {
            name: "bool_group_eq_true",
            input: "(1 < 2) == true",
            output: "true",
        },
        Case {
            name: "bool_group_eq_false",
            input: "(1 < 2) == false",
            output: "false",
        },
        Case {
            name: "bool_group_gt_eq_true",
            input: "(1 > 2) == true",
            output: "false",
        },
        Case {
            name: "bool_group_gt_eq_false",
            input: "(1 > 2) == false",
            output: "true",
        },
        // lte_gte
        Case {
            name: "int_lte_less",
            input: "1 <= 2",
            output: "true",
        },
        Case {
            name: "int_lte_equal",
            input: "2 <= 2",
            output: "true",
        },
        Case {
            name: "int_lte_greater",
            input: "3 <= 2",
            output: "false",
        },
        Case {
            name: "int_gte_less",
            input: "1 >= 2",
            output: "false",
        },
        Case {
            name: "int_gte_equal",
            input: "2 >= 2",
            output: "true",
        },
        Case {
            name: "int_gte_greater",
            input: "3 >= 2",
            output: "true",
        },
        Case {
            name: "float_lte_equal",
            input: "1.5 <= 1.5",
            output: "true",
        },
        Case {
            name: "float_lte_false",
            input: "1.5 <= 1.4",
            output: "false",
        },
        Case {
            name: "float_gte_equal",
            input: "1.5 >= 1.5",
            output: "true",
        },
        Case {
            name: "float_gte_false",
            input: "1.4 >= 1.5",
            output: "false",
        },
        Case {
            name: "mixed_lte_int_to_float",
            input: "1 <= 1.5",
            output: "true",
        },
        Case {
            name: "mixed_gte_float_to_int",
            input: "1.5 >= 1",
            output: "true",
        },
        // bang
        Case {
            name: "bang_true",
            input: "!true",
            output: "false",
        },
        Case {
            name: "bang_false",
            input: "!false",
            output: "true",
        },
        Case {
            name: "bang_int",
            input: "!5",
            output: "false",
        },
        Case {
            name: "bang_bang_true",
            input: "!!true",
            output: "true",
        },
        Case {
            name: "bang_bang_false",
            input: "!!false",
            output: "false",
        },
        Case {
            name: "bang_bang_int",
            input: "!!5",
            output: "true",
        },
        // chained_booleans
        Case {
            name: "and_both_true",
            input: "true && true",
            output: "true",
        },
        Case {
            name: "and_left_true_evaluates_right",
            input: "true && false",
            output: "false",
        },
        Case {
            name: "and_left_false_short_circuits",
            input: "false && foo",
            output: "false",
        },
        Case {
            name: "and_chain",
            input: "true && true && false",
            output: "false",
        },
        Case {
            name: "or_left_true_short_circuits",
            input: "true || foo",
            output: "true",
        },
        Case {
            name: "or_left_false_evaluates_right",
            input: "false || true",
            output: "true",
        },
        Case {
            name: "or_chain",
            input: "false || false || true",
            output: "true",
        },
        Case {
            name: "and_with_comparisons",
            input: "1 < 2 && 3 < 4",
            output: "true",
        },
        Case {
            name: "or_with_comparisons",
            input: "0 == 1 || 2 == 2",
            output: "true",
        },
        // if_else
        Case {
            name: "if_true_returns_consequence",
            input: "if (true) { 10 }",
            output: "10",
        },
        Case {
            name: "if_false_returns_null",
            input: "if (false) { 10 }",
            output: "",
        },
        Case {
            name: "if_truthy_int",
            input: "if (1) { 10 }",
            output: "10",
        },
        Case {
            name: "if_lt_true",
            input: "if (1 < 2) { 10 }",
            output: "10",
        },
        Case {
            name: "if_gt_false_null",
            input: "if (1 > 2) { 10 }",
            output: "",
        },
        Case {
            name: "if_else_falls_through",
            input: "if (1 > 2) { 10 } else { 20 }",
            output: "20",
        },
        Case {
            name: "if_else_true_branch",
            input: "if (1 < 2) { 10 } else { 20 }",
            output: "10",
        },
        // return
        Case {
            name: "return_simple",
            input: "return 10;",
            output: "10",
        },
        Case {
            name: "return_ignores_trailing",
            input: "return 10; 9;",
            output: "10",
        },
        Case {
            name: "return_expr",
            input: "return 2 * 5; 9;",
            output: "10",
        },
        Case {
            name: "return_after_stmt",
            input: "9; return 2 * 5; 9;",
            output: "10",
        },
        Case {
            name: "return_nested_if",
            input: "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
            output: "10",
        },
        // errors
        Case {
            name: "err_int_plus_bool",
            input: "5 + true;",
            output: "ERROR: type mismatch: Integer + Boolean",
        },
        Case {
            name: "err_int_plus_bool_trailing",
            input: "5 + true; 5;",
            output: "ERROR: type mismatch: Integer + Boolean",
        },
        Case {
            name: "err_negate_bool",
            input: "-true",
            output: "ERROR: unknown operator: -Boolean",
        },
        Case {
            name: "err_bool_plus_bool",
            input: "true + false;",
            output: "ERROR: unknown operator: Boolean + Boolean",
        },
        Case {
            name: "err_bool_plus_bool_leading",
            input: "5; true + false; 5",
            output: "ERROR: unknown operator: Boolean + Boolean",
        },
        Case {
            name: "err_bool_plus_bool_in_if",
            input: "if (10 > 1) { true + false; }",
            output: "ERROR: unknown operator: Boolean + Boolean",
        },
        Case {
            name: "err_bool_plus_bool_in_return",
            input: "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
            output: "ERROR: unknown operator: Boolean + Boolean",
        },
        Case {
            name: "err_identifier_not_found",
            input: "foobar",
            output: "ERROR: identifier not found: foobar",
        },
        Case {
            name: "err_assign_undeclared",
            input: "x = 5",
            output: "ERROR: identifier not found: x",
        },
        Case {
            name: "err_string_minus_string",
            input: "\"hello\" - \"world\"",
            output: "ERROR: unknown operator: String - String",
        },
        Case {
            name: "err_function_as_hash_key",
            input: r#"{"name": "Monkey"}[fn(x) { x }];"#,
            output: "ERROR: Function is unusable as a hash key",
        },
        Case {
            name: "err_push_wrong_arity",
            input: "push([1, 2], 3, 4)",
            output: "ERROR: wrong number of arguments to `push`. got=3, want=2",
        },
        Case {
            name: "err_push_wrong_type",
            input: "push(5, 1)",
            output: "ERROR: argument to `push` not supported, expected Array, got Integer",
        },
        Case {
            name: "err_index_assign_out_of_bounds",
            input: "let a = [1, 2, 3]; a[5] = 9",
            output: "ERROR: index 5 out of bounds for Array of length 3, use `push` to grow",
        },
        Case {
            name: "err_index_assign_non_integer",
            input: r#"let a = [1, 2, 3]; a["x"] = 9"#,
            output: "ERROR: array index must be an Integer, got String",
        },
        Case {
            name: "err_index_assign_unsupported_target",
            input: "let n = 5; n[0] = 1",
            output: "ERROR: index assignment not supported: Integer",
        },
        Case {
            name: "err_hash_assign_function_key",
            input: r#"let h = {}; h[fn(x) { x }] = 1"#,
            output: "ERROR: Function is unusable as a hash key",
        },
        // let
        Case {
            name: "let_simple",
            input: "let a = 5; a;",
            output: "5",
        },
        Case {
            name: "let_with_expr",
            input: "let a = 5 * 5; a;",
            output: "25",
        },
        Case {
            name: "let_from_another",
            input: "let a = 5; let b = a; b;",
            output: "5",
        },
        Case {
            name: "let_chained",
            input: "let a = 5; let b = a; let c = a + b + 5; c;",
            output: "15",
        },
        Case {
            name: "err_let_redeclaration",
            input: "let a = 5; let a = 10; a",
            output: "ERROR: a already exists, you may reassign it's value instead by removing the `let` keyword",
        },
        Case {
            name: "let_redeclare_in_nested_scope_is_fine",
            input: "let a = 5; let f = fn() { let a = 10; a }; f() + a",
            output: "15",
        },
        // reassignment
        Case {
            name: "reassign_literal",
            input: "let a = 5; a = 10; a",
            output: "10",
        },
        Case {
            name: "reassign_from_var",
            input: "let a = 1; let b = 2; a = b; a",
            output: "2",
        },
        Case {
            name: "reassign_self_increment",
            input: "let a = 0; a = a + 1; a = a + 1; a",
            output: "2",
        },
        Case {
            name: "reassign_in_for_loop",
            input: "let total = 0; for x in [1, 2, 3] { total = total + x; } total",
            output: "6",
        },
        Case {
            name: "reassign_captured_by_closure",
            input: "let a = 0; let f = fn() { a = 9; }; f(); a",
            output: "9",
        },
        // index_assignment
        Case {
            name: "index_assign_array_element",
            input: "let a = [1, 2, 3]; a[0] = 9; a[0]",
            output: "9",
        },
        Case {
            name: "index_assign_array_sum",
            input: "let a = [1, 2, 3]; a[2] = 30; a[0] + a[1] + a[2]",
            output: "33",
        },
        Case {
            name: "index_assign_array_via_var",
            input: "let a = [1, 2, 3]; let i = 1; a[i] = a[i] + 5; a[1]",
            output: "7",
        },
        Case {
            name: "index_assign_hash_existing_key",
            input: r#"let h = {"a": 1}; h["a"] = 5; h["a"]"#,
            output: "5",
        },
        Case {
            name: "index_assign_hash_new_key",
            input: r#"let h = {"a": 1}; h["b"] = 2; h["a"] + h["b"]"#,
            output: "3",
        },
        Case {
            name: "index_assign_hash_empty",
            input: "let h = {}; h[1] = 10; h[1]",
            output: "10",
        },
        Case {
            name: "index_assign_hash_dot",
            input: r#"let h = {"x": 1}; h.x = 42; h.x"#,
            output: "42",
        },
        Case {
            name: "index_assign_nested_array_of_hashes",
            input: r#"let people = [{"name": "a"}, {"name": "b"}]; people[1]["name"] = "z"; len(people[1]["name"])"#,
            output: "1",
        },
        Case {
            name: "index_assign_shares_underlying_array",
            input: "let a = [1, 2, 3]; let b = a; a[0] = 99; b[0]",
            output: "99",
        },
        // functions
        Case {
            name: "function_literal_inspect",
            input: "fn(x) { x + 2; }",
            output: "fn(x) {\n(x + 2)\n}",
        },
        Case {
            name: "function_identity",
            input: "let identity = fn(x) { x; }; identity(5);",
            output: "5",
        },
        Case {
            name: "function_identity_with_return",
            input: "let identity = fn(x) { return x; }; identity(5);",
            output: "5",
        },
        Case {
            name: "function_double",
            input: "let double = fn(x) { x * 2; }; double(5);",
            output: "10",
        },
        Case {
            name: "function_add",
            input: "let add = fn(x, y) { x + y; }; add(5, 5);",
            output: "10",
        },
        Case {
            name: "function_add_nested_call",
            input: "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
            output: "20",
        },
        Case {
            name: "function_immediately_invoked",
            input: "fn(x) { x; }(5)",
            output: "5",
        },
        // closures
        Case {
            name: "closure_new_adder",
            input: "let newAdder = fn(x) { fn(y) { x + y }; }; let addTwo = newAdder(2); addTwo(2);",
            output: "4",
        },
        // strings
        Case {
            name: "string_literal",
            input: "\"hello world\"",
            output: "hello world",
        },
        Case {
            name: "string_concat",
            input: "\"hello\" + \" \" + \"world\"",
            output: "hello world",
        },
        Case {
            name: "string_index_first_char",
            input: r#" let s = "xyz"; print(s[0]); "#,
            output: "x",
        },
        Case {
            name: "string_index_second_char",
            input: r#" let s = "xyz"; print(s[1]); "#,
            output: "y",
        },
        Case {
            name: "string_index_offset",
            input: r#" print("xyzyzyzywdq"[9]); "#,
            output: "d",
        },
        Case {
            name: "string_index_char_eq_char_literal",
            input: r#" let s = "xyz"; print(s[0] == 'x'); "#,
            output: "true",
        },
        Case {
            name: "string_index_char_neq_string_literal",
            input: r#" let s = "xyz"; print(s[0] == "x"); "#,
            output: "false",
        },
        Case {
            name: "string_index_char_coerced_to_str_eq",
            input: r#" let s = "xyz"; print(str(s[0]) == "x"); "#,
            output: "true",
        },
        Case {
            name: "string_looping",
            input: r#" for c in "xyz" { print(c) }; "#,
            output: "xyz",
        },
        // chars
        Case {
            name: "char_literal_print",
            input: "print('a')",
            output: "a",
        },
        Case {
            name: "char_type_via_string_loop",
            input: r#"for c in "xy" { println(type(c)) }"#,
            output: "Char\nChar\n",
        },
        Case {
            name: "char_type_via_string_index",
            input: r#"print(type("xy"[0]))"#,
            output: "Char",
        },
        Case {
            name: "char_eq_char_true",
            input: "'a' == 'a'",
            output: "true",
        },
        Case {
            name: "char_eq_char_false",
            input: "'a' == 'b'",
            output: "false",
        },
        Case {
            name: "char_neq_char_true",
            input: "'a' != 'b'",
            output: "true",
        },
        Case {
            name: "char_neq_string_literal",
            input: r#"'a' == "a""#,
            output: "false",
        },
        Case {
            name: "string_literal_neq_char",
            input: r#""a" == 'a'"#,
            output: "false",
        },
        Case {
            name: "char_coerced_str_eq_string",
            input: r#"str('a') == "a""#,
            output: "true",
        },
        // builtins
        Case {
            name: "len_empty_string",
            input: r#"len("")"#,
            output: "0",
        },
        Case {
            name: "len_string",
            input: r#"len("four")"#,
            output: "4",
        },
        Case {
            name: "len_longer_string",
            input: r#"len("hello world")"#,
            output: "11",
        },
        Case {
            name: "len_wrong_type",
            input: "len(1)",
            output: "ERROR: argument to `len` not supported, expected String or Array, got Integer",
        },
        Case {
            name: "len_wrong_arity",
            input: r#"len("one", "two")"#,
            output: "ERROR: wrong number of arguments to `len`. got=2, want=1",
        },
        Case {
            name: "min_basic",
            input: "min(1, 2)",
            output: "1",
        },
        Case {
            name: "max_basic",
            input: "max(1, 2)",
            output: "2",
        },
        Case {
            name: "min_reversed",
            input: "min(1, 103)",
            output: "1",
        },
        Case {
            name: "max_reversed",
            input: "max(103, 1)",
            output: "103",
        },
        Case {
            name: "min_floats",
            input: "min(1.5, 2.5)",
            output: "1.5",
        },
        Case {
            name: "max_floats",
            input: "max(1.5, 2.5)",
            output: "2.5",
        },
        Case {
            name: "min_int_float",
            input: "min(1, 2.5)",
            output: "1",
        },
        Case {
            name: "max_int_float",
            input: "max(1, 2.5)",
            output: "2.5",
        },
        Case {
            name: "min_float_int",
            input: "min(2.5, 1)",
            output: "1",
        },
        Case {
            name: "max_float_int",
            input: "max(2.5, 1)",
            output: "2.5",
        },
        Case {
            name: "err_min_wrong_type",
            input: "min(true, 1)",
            output: "ERROR: arguments to `min` not supported, expected (Integer || Float) and (Integer || Float), got Boolean and Integer",
        },
        Case {
            name: "err_max_wrong_type",
            input: "max(true, 1)",
            output: "ERROR: arguments to `max` not supported, expected (Integer || Float) and (Integer || Float), got Boolean and Integer",
        },
        Case {
            name: "err_min_wrong_arity",
            input: "min(1)",
            output: "ERROR: wrong number of arguments to `min`. got=1, want=2",
        },
        Case {
            name: "err_max_wrong_arity",
            input: "max(1)",
            output: "ERROR: wrong number of arguments to `max`. got=1, want=2",
        },
        Case {
            name: "err_first_wrong_type",
            input: "first(5)",
            output: "ERROR: arguments to `first` not supported, expected array, got Integer",
        },
        Case {
            name: "err_first_wrong_arity",
            input: "first([1],[2])",
            output: "ERROR: wrong number of arguments to `first`. got=2, want=1",
        },
        Case {
            name: "err_last_wrong_type",
            input: "last(5)",
            output: "ERROR: arguments to `last` not supported, expected array, got Integer",
        },
        Case {
            name: "err_last_wrong_arity",
            input: "last([1],[2])",
            output: "ERROR: wrong number of arguments to `last`. got=2, want=1",
        },
        Case {
            name: "err_rest_wrong_type",
            input: "rest(5)",
            output: "ERROR: arguments to `rest` not supported, expected array, got Integer",
        },
        Case {
            name: "err_rest_wrong_arity",
            input: "rest([1],[2])",
            output: "ERROR: wrong number of arguments to `rest`. got=2, want=1",
        },
        Case {
            name: "err_push_no_args",
            input: "push()",
            output: "ERROR: wrong number of arguments to `push`. got=0, want=2",
        },
        Case {
            name: "builtin_function_inspect",
            input: "print(len)",
            output: "builtin function",
        },
        // range_builtin
        Case {
            name: "range_single_arg",
            input: "range(5)",
            output: "[0, 1, 2, 3, 4]",
        },
        Case {
            name: "range_two_args",
            input: "range(2,5)",
            output: "[2, 3, 4]",
        },
        Case {
            name: "err_range_wrong_type_single",
            input: r#"range("a")"#,
            output: "ERROR: argument to `range` not supported, expected Integer, got String",
        },
        Case {
            name: "err_range_wrong_type_double",
            input: r#"range("a","b")"#,
            output: "ERROR: arguments to `range` not supported, expected Integer and Integer, got String and String",
        },
        Case {
            name: "err_range_no_args",
            input: "range()",
            output: "ERROR: wrong number of arguments to `range`. got=0, want=1 or 2",
        },
        Case {
            name: "err_range_too_many_args",
            input: "range(1,2,3)",
            output: "ERROR: wrong number of arguments to `range`. got=3, want=1 or 2",
        },
        // type_builtin
        Case {
            name: "type_integer",
            input: "type(5)",
            output: "Integer",
        },
        Case {
            name: "type_float",
            input: "type(1.5)",
            output: "Float",
        },
        Case {
            name: "type_boolean",
            input: "type(true)",
            output: "Boolean",
        },
        Case {
            name: "type_string",
            input: r#"type("hi")"#,
            output: "String",
        },
        Case {
            name: "type_char",
            input: "type('c')",
            output: "Char",
        },
        Case {
            name: "type_null",
            input: "type(null)",
            output: "Null",
        },
        Case {
            name: "type_array",
            input: "type([1, 2])",
            output: "Array",
        },
        Case {
            name: "type_hash",
            input: r#"type({"a": 1})"#,
            output: "Hash",
        },
        Case {
            name: "type_function",
            input: "type(fn(x) { x })",
            output: "Function",
        },
        Case {
            name: "err_type_wrong_arity",
            input: "type(1, 2)",
            output: "ERROR: expected 1 argument to type(), received 2",
        },
        // str_builtin
        Case {
            name: "str_from_int",
            input: "print(str(5))",
            output: "5",
        },
        Case {
            name: "str_from_float",
            input: "print(str(1.5))",
            output: "1.5",
        },
        Case {
            name: "str_from_bool",
            input: "print(str(true))",
            output: "true",
        },
        Case {
            name: "str_from_char",
            input: "print(str('c'))",
            output: "c",
        },
        Case {
            name: "str_from_string_eq",
            input: r#"print(str("hi") == "hi")"#,
            output: "true",
        },
        Case {
            name: "err_str_wrong_arity",
            input: "str(1, 2)",
            output: "ERROR: expected 1 argument to str(), received 2",
        },
        // int_builtin
        Case {
            name: "int_from_int",
            input: "int(5)",
            output: "5",
        },
        Case {
            name: "int_from_float_truncates",
            input: "int(3.9)",
            output: "3",
        },
        Case {
            name: "int_from_negative_float_truncates",
            input: "int(-3.9)",
            output: "-3",
        },
        Case {
            name: "int_from_true",
            input: "int(true)",
            output: "1",
        },
        Case {
            name: "int_from_false",
            input: "int(false)",
            output: "0",
        },
        Case {
            name: "int_from_null",
            input: "int(null)",
            output: "0",
        },
        Case {
            name: "err_int_from_string",
            input: r#"int("5")"#,
            output: "ERROR: String cannot be coerced to an int",
        },
        Case {
            name: "err_int_wrong_arity",
            input: "int(1, 2)",
            output: "ERROR: expected 1 argument to int(), received 2",
        },
        // float_builtin
        Case {
            name: "float_from_int",
            input: "float(5)",
            output: "5",
        },
        Case {
            name: "float_from_float",
            input: "float(1.5)",
            output: "1.5",
        },
        Case {
            name: "float_from_true",
            input: "float(true)",
            output: "1",
        },
        Case {
            name: "float_from_false",
            input: "float(false)",
            output: "0",
        },
        Case {
            name: "err_float_from_string",
            input: r#"float("5")"#,
            output: "ERROR: String cannot be coerced to an int",
        },
        Case {
            name: "err_float_wrong_arity",
            input: "float(1, 2)",
            output: "ERROR: expected 1 argument to int(), received 2",
        },
        // arrays
        Case {
            name: "array_literal_with_exprs",
            input: "[1, 2 * 2, 3 + 3]",
            output: "[1, 4, 6]",
        },
        Case {
            name: "array_index_zero",
            input: "[1, 2, 3][0]",
            output: "1",
        },
        Case {
            name: "array_index_one",
            input: "[1, 2, 3][1]",
            output: "2",
        },
        Case {
            name: "array_index_two",
            input: "[1, 2, 3][2]",
            output: "3",
        },
        Case {
            name: "array_index_via_var",
            input: "let i = 0; [1][i];",
            output: "1",
        },
        Case {
            name: "array_index_expr",
            input: "[1, 2, 3][1 + 1];",
            output: "3",
        },
        Case {
            name: "array_index_named_var",
            input: "let myArray = [1, 2, 3]; myArray[2];",
            output: "3",
        },
        Case {
            name: "array_index_sum",
            input: "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
            output: "6",
        },
        Case {
            name: "array_index_by_element",
            input: "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
            output: "2",
        },
        Case {
            name: "array_index_out_of_bounds",
            input: "[1, 2, 3][3]",
            output: "",
        },
        Case {
            name: "array_index_negative",
            input: "[1, 2, 3][-1]",
            output: "",
        },
        Case {
            name: "len_array_with_mixed_elements",
            input: r#"len([1, 2 * 2, 3 + 3, "hello"])"#,
            output: "4",
        },
        Case {
            name: "array_first",
            input: "first([1, 2, 3])",
            output: "1",
        },
        Case {
            name: "array_first_empty",
            input: "first([])",
            output: "",
        },
        Case {
            name: "array_last",
            input: "last([1, 2, 3])",
            output: "3",
        },
        Case {
            name: "array_last_empty",
            input: "last([])",
            output: "",
        },
        Case {
            name: "array_rest",
            input: "rest([1, 2, 3])",
            output: "[2, 3]",
        },
        Case {
            name: "array_rest_single",
            input: "rest([1])",
            output: "[]",
        },
        Case {
            name: "array_rest_chained",
            input: "rest(rest(rest(rest([1, 2, 3, 4, 5]))))",
            output: "[5]",
        },
        Case {
            name: "array_push",
            input: "push([1, 2], 3)",
            output: "[1, 2, 3]",
        },
        Case {
            name: "array_push_empty",
            input: "push([], 1)",
            output: "[1]",
        },
        // hash
        Case {
            name: "hash_index_hit",
            input: r#"{"foo": 5}["foo"]"#,
            output: "5",
        },
        Case {
            name: "hash_index_miss",
            input: r#"{"foo": 5}["bar"]"#,
            output: "",
        },
        Case {
            name: "hash_index_via_var",
            input: r#"let key = "foo"; {"foo": 5}[key]"#,
            output: "5",
        },
        Case {
            name: "hash_index_empty_hash",
            input: r#"{}["foo"]"#,
            output: "",
        },
        Case {
            name: "hash_index_int_key",
            input: "{5: 5}[5]",
            output: "5",
        },
        Case {
            name: "hash_index_true_key",
            input: "{true: 5}[true]",
            output: "5",
        },
        Case {
            name: "hash_index_false_key",
            input: "{false: 5}[false]",
            output: "5",
        },
        Case {
            name: "hash_index_char_key",
            input: "{'a': 1}['a']",
            output: "1",
        },
        Case {
            name: "hash_index_float_key",
            input: "{1.5: 1}[1.5]",
            output: "1",
        },
        Case {
            name: "err_hash_null_key",
            input: "{null: 1}",
            output: "ERROR: Null is unusable as a hash key",
        },
        Case {
            name: "err_hash_array_key",
            input: "{[1]: 1}",
            output: "ERROR: Array is unusable as a hash key",
        },
        Case {
            name: "hash_dot_access_hit",
            input: r#"{"foo": 5}.foo"#,
            output: "5",
        },
        Case {
            name: "hash_dot_access_miss",
            input: r#"{"foo": 5}.bar"#,
            output: "",
        },
        Case {
            name: "hash_dot_access_via_var",
            input: r#"let h = {"foo": 5}; h.foo"#,
            output: "5",
        },
        Case {
            name: "hash_dot_access_nested",
            input: r#"let h = {"a": {"b": 42}}; h.a.b"#,
            output: "42",
        },
        // print
        Case {
            name: "print_basic",
            input: r#"print("hello")"#,
            output: "hello",
        },
        Case {
            name: "println_basic",
            input: r#"println("hello")"#,
            output: "hello\n",
        },
        Case {
            name: "print_multiple_calls",
            input: r#"print("a"); print("b"); print("c")"#,
            output: "abc",
        },
        Case {
            name: "println_multiple_calls",
            input: r#"println("a"); println("b")"#,
            output: "a\nb\n",
        },
        Case {
            name: "print_multiple_args",
            input: r#"print("x", 42, true)"#,
            output: "x 42 true",
        },
        Case {
            name: "println_multiple_args",
            input: r#"println("x", 42, true)"#,
            output: "x 42 true\n",
        },
        Case {
            name: "print_println_mixed",
            input: r#"print("a"); println("b"); print("c")"#,
            output: "ab\nc",
        },
        Case {
            name: "println_inside_function",
            input: r#"let greet = fn(name) { println("hi " + name) }; greet("bob")"#,
            output: "hi bob\n",
        },
        Case {
            name: "print_no_args",
            input: "print()",
            output: "",
        },
        // for_loops
        Case {
            name: "for_array_prints_each",
            input: "for x in [1, 2, 3] { print(x) }",
            output: "123",
        },
        Case {
            name: "for_array_of_strings",
            input: r#"for x in ["a", "b"] { println(x) }"#,
            output: "a\nb\n",
        },
        Case {
            name: "for_empty_array",
            input: "for x in [] { print(x) }",
            output: "",
        },
        Case {
            name: "for_named_array",
            input: "let nums = [10, 20]; for n in nums { print(n) }",
            output: "1020",
        },
        Case {
            name: "for_return_propagates",
            input: "let f = fn() { for x in [1, 2, 3] { if (x == 2) { return x } } }; f()",
            output: "2",
        },
        Case {
            name: "for_variable_is_scoped",
            input: "for x in [1, 2, 3] { x }; x",
            output: "ERROR: identifier not found: x",
        },
        Case {
            name: "for_over_non_iterable",
            input: "for x in 5 { x }",
            output: "ERROR: Integer is not iterable",
        },
        Case {
            name: "for_array_wrong_variable_count",
            input: "for a, b in [1, 2] { a }",
            output: "ERROR: for loop over Array expects 1 variable, got 2",
        },
        Case {
            name: "for_hash_wrong_variable_count",
            input: "for a, b, c in {1: 2} { a }",
            output: "ERROR: for loop over Hash expects 1 or 2 variables, got 3",
        },
        Case {
            name: "for_hash_key_and_value",
            input: r#"for k, v in {"a": 1} { print(k); print(v) }"#,
            output: "a1",
        },
        Case {
            name: "for_hash_key_only",
            input: r#"for k in {"only": 9} { print(k) }"#,
            output: "only",
        },
        Case {
            name: "for_empty_hash",
            input: "for k, v in {} { print(k) }",
            output: "",
        },
        Case {
            name: "for_hash_return_propagates",
            input: r#"
let find = fn(m, target) {
    for k, v in m {
        if (k == target) {
            return v
        }
    }
};

find({1: 10, 2: 20, 3: 30}, 2)"#,
            output: "20",
        },
        // floats
        Case {
            name: "float_literal",
            input: "3.5",
            output: "3.5",
        },
        Case {
            name: "float_negative_literal",
            input: "-2.25",
            output: "-2.25",
        },
        Case {
            name: "float_add",
            input: "1.5 + 2.25",
            output: "3.75",
        },
        Case {
            name: "float_add_via_vars",
            input: "let a = 1.5; let b = 2.75; a + b",
            output: "4.25",
        },
        Case {
            name: "float_array",
            input: "[1.5, 2.0 * 2.0, 3.0 + 0.25]",
            output: "[1.5, 4, 3.25]",
        },
        // null
        Case {
            name: "null_literal",
            input: "null",
            output: "",
        },
        Case {
            name: "null_via_var",
            input: "let x = null; x",
            output: "",
        },
        Case {
            name: "null_returned_from_function",
            input: "let f = fn() { null }; f()",
            output: "",
        },
        Case {
            name: "null_missing_hash_key",
            input: r#"{"a": 1}["b"]"#,
            output: "",
        },
        Case {
            name: "null_explicit_hash_value",
            input: r#"{"a": null}["a"]"#,
            output: "",
        },
        Case {
            name: "null_missing_dot_access",
            input: r#"let m = {"a": 1}; m.b"#,
            output: "",
        },
        Case {
            name: "null_array_out_of_bounds",
            input: "[1, 2, 3][5]",
            output: "",
        },
        Case {
            name: "null_inside_array",
            input: "[null][0]",
            output: "",
        },
        Case {
            name: "null_eq_null",
            input: "null == null",
            output: "true",
        },
        Case {
            name: "null_neq_null",
            input: "null != null",
            output: "false",
        },
        Case {
            name: "null_eq_int",
            input: "null == 5",
            output: "false",
        },
        Case {
            name: "int_eq_null",
            input: "5 == null",
            output: "false",
        },
        Case {
            name: "null_neq_int",
            input: "null != 5",
            output: "true",
        },
        Case {
            name: "int_neq_null",
            input: "5 != null",
            output: "true",
        },
        Case {
            name: "null_eq_string",
            input: r#"null == "a""#,
            output: "false",
        },
        Case {
            name: "null_eq_bool",
            input: "null == true",
            output: "false",
        },
        Case {
            name: "null_neq_bool",
            input: "null != false",
            output: "true",
        },
        Case {
            name: "null_var_eq_null",
            input: "let x = null; x == null",
            output: "true",
        },
        Case {
            name: "null_eq_null_var",
            input: "let x = null; null == x",
            output: "true",
        },
        Case {
            name: "null_var_neq_null",
            input: "let x = null; x != null",
            output: "false",
        },
        Case {
            name: "int_var_eq_null",
            input: "let y = 5; y == null",
            output: "false",
        },
        Case {
            name: "int_var_neq_null",
            input: "let y = 5; y != null",
            output: "true",
        },
        Case {
            name: "null_var_eq_null_var",
            input: "let a = null; let b = null; a == b",
            output: "true",
        },
        Case {
            name: "null_missing_hash_key_eq_null",
            input: r#"{"a": 1}["b"] == null"#,
            output: "true",
        },
        Case {
            name: "null_explicit_hash_value_eq_null",
            input: r#"{"a": null}["a"] == null"#,
            output: "true",
        },
        Case {
            name: "present_hash_value_neq_null",
            input: r#"{"a": 1}["a"] == null"#,
            output: "false",
        },
        Case {
            name: "null_array_out_of_bounds_eq_null",
            input: "[1, 2, 3][5] == null",
            output: "true",
        },
        Case {
            name: "null_is_falsy_in_if",
            input: r#"if (null) { print("t") } else { print("f") }"#,
            output: "f",
        },
        Case {
            name: "null_eq_check_in_if",
            input: r#"let x = null; if (x == null) { print("missing") }"#,
            output: "missing",
        },
        // decode_ways_regression
        Case {
            name: "decode_ways_11106",
            input: r#"
let s = "11106";
let decodeWays = fn(s) {
  let cache = { len(s): 1 }

  let dp = fn(i) {
    if (cache[i] != null) {
      return cache[i];
    }

    if (s[i] == '0') {
      return 0;
    }

    let res = dp(i + 1);
    if (i + 1 < len(s)) {
      if (s[i] == '1') {
        res = res + dp(i + 2);
      } else {
        if (s[i] == '2') {
          for n in "0123456" {
            if (s[i + 1] == n) {
              res = res + dp(i + 2);
            }
          }
        }
      }
    }
    cache[i] = res;
    return res;
  }

  return dp(0);
}

decodeWays(s)
"#,
            output: "2",
        },
        Case {
            name: "decode_ways_12",
            input: r#"
let s = "12";
let decodeWays = fn(s) {
  let cache = { len(s): 1 }

  let dp = fn(i) {
    if (cache[i] != null) {
      return cache[i];
    }

    if (s[i] == '0') {
      return 0;
    }

    let res = dp(i + 1);
    if (i + 1 < len(s)) {
      if (s[i] == '1') {
        res = res + dp(i + 2);
      } else {
        if (s[i] == '2') {
          for n in "0123456" {
            if (s[i + 1] == n) {
              res = res + dp(i + 2);
            }
          }
        }
      }
    }
    cache[i] = res;
    return res;
  }

  return dp(0);
}

decodeWays(s)
"#,
            output: "2",
        },
        Case {
            name: "decode_ways_226",
            input: r#"
let s = "226";
let decodeWays = fn(s) {
  let cache = { len(s): 1 }

  let dp = fn(i) {
    if (cache[i] != null) {
      return cache[i];
    }

    if (s[i] == '0') {
      return 0;
    }

    let res = dp(i + 1);
    if (i + 1 < len(s)) {
      if (s[i] == '1') {
        res = res + dp(i + 2);
      } else {
        if (s[i] == '2') {
          for n in "0123456" {
            if (s[i + 1] == n) {
              res = res + dp(i + 2);
            }
          }
        }
      }
    }
    cache[i] = res;
    return res;
  }

  return dp(0);
}

decodeWays(s)
"#,
            output: "3",
        },
        Case {
            name: "decode_ways_06",
            input: r#"
let s = "06";
let decodeWays = fn(s) {
  let cache = { len(s): 1 }

  let dp = fn(i) {
    if (cache[i] != null) {
      return cache[i];
    }

    if (s[i] == '0') {
      return 0;
    }

    let res = dp(i + 1);
    if (i + 1 < len(s)) {
      if (s[i] == '1') {
        res = res + dp(i + 2);
      } else {
        if (s[i] == '2') {
          for n in "0123456" {
            if (s[i + 1] == n) {
              res = res + dp(i + 2);
            }
          }
        }
      }
    }
    cache[i] = res;
    return res;
  }

  return dp(0);
}

decodeWays(s)
"#,
            output: "0",
        },
        // parser_errors
        Case {
            name: "err_parse_let_missing_ident",
            input: "let 5 = 1;",
            output: "parser has 3 error(s):\n\texpected next token to be IDENT, got INT instead\n\tinvalid assignment target: 5, expected an identifier or index expression\n\tno prefix parse function for = found",
        },
        Case {
            name: "err_parse_let_missing_assign",
            input: "let x 5;",
            output: "parser has 1 error(s):\n\texpected next token to be =, got INT instead",
        },
        Case {
            name: "err_parse_fn_missing_paren",
            input: "fn x { x }",
            output: "parser has 3 error(s):\n\texpected next token to be (, got IDENT instead\n\texpected next token to be :, got } instead\n\tno prefix parse function for } found",
        },
        Case {
            name: "err_parse_if_missing_paren",
            input: "if true { 1 }",
            output: "parser has 3 error(s):\n\texpected next token to be (, got true instead\n\texpected next token to be :, got } instead\n\tno prefix parse function for } found",
        },
        Case {
            name: "err_parse_hash_missing_colon",
            input: r#"{"a" 1}"#,
            output: "parser has 2 error(s):\n\texpected next token to be :, got INT instead\n\tno prefix parse function for } found",
        },
        Case {
            name: "err_parse_for_missing_in",
            input: "for x [1,2] { x }",
            output: "parser has 3 error(s):\n\texpected next token to be in, got [ instead\n\texpected next token to be :, got } instead\n\tno prefix parse function for } found",
        },
        Case {
            name: "err_parse_array_missing_bracket",
            input: "[1, 2",
            output: "parser has 1 error(s):\n\texpected next token to be ], got EOF instead",
        },
        Case {
            name: "err_parse_no_prefix_rparen",
            input: ")",
            output: "parser has 1 error(s):\n\tno prefix parse function for ) found",
        },
        Case {
            name: "err_parse_no_prefix_semicolon",
            input: ";",
            output: "parser has 1 error(s):\n\tno prefix parse function for ; found",
        },
        Case {
            name: "err_parse_invalid_assignment_target",
            input: "5 = 10",
            output: "parser has 2 error(s):\n\tinvalid assignment target: 5, expected an identifier or index expression\n\tno prefix parse function for = found",
        },
        Case {
            name: "err_lexer_char_too_long",
            input: "'ab'",
            output: "parser has 1 error(s):\n\tlexer error encountered: failed to parse character due to: char does not have a closing '",
        },
        Case {
            name: "err_lexer_float_missing_digits",
            input: "5.",
            output: "parser has 1 error(s):\n\tlexer error encountered: failed to find digits after period on a float",
        },
        Case {
            name: "err_parse_multiple_statements_accumulate",
            input: "let 5 = 1; let 6 = 2;",
            output: "parser has 6 error(s):\n\texpected next token to be IDENT, got INT instead\n\tinvalid assignment target: 5, expected an identifier or index expression\n\tno prefix parse function for = found\n\texpected next token to be IDENT, got INT instead\n\tinvalid assignment target: 6, expected an identifier or index expression\n\tno prefix parse function for = found",
        },
    ];

    #[test]
    fn eval_table() {
        for case in CASES {
            let got = run(case.input);
            assert_eq!(
                got, case.output,
                "case `{}` failed for input {:?}",
                case.name, case.input
            );
        }
    }
}
