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
            ArrayObject, ErrorObject, FloatObject, FunctionObject, HashObject, IntegerObject,
            NullObject, Object, ObjectHasher, ObjectType, Objecter, ReturnValueObject,
            StringObject,
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

            env.borrow_mut().set(stmt.name.value.to_string(), val);

            Ok(Object::NULL)
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
    Ok(Object::String(StringObject {
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
                })?
                .to_string()
                .as_str(),
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
                .map(|c| {
                    // TODO: Update eval to support CharObject
                    vec![Object::String(StringObject {
                        value: Rc::from(c.to_string().as_str()),
                    })]
                })
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
