use phf::phf_map;

use crate::eval::object::{
    ArrayObject, BuiltInFunctionObject, ErrorObject, FloatObject, IntegerObject, Object, Objecter,
    StringObject, environment::Env,
};

pub static BUILTINS: phf::Map<&'static str, BuiltInFunctionObject> = phf_map! {
    "len" => BuiltInFunctionObject {
        function: len,
        function_name: "len",
    },
    "max" => BuiltInFunctionObject {
        function: max,
        function_name: "max",
    },
    "min" => BuiltInFunctionObject {
        function: min,
        function_name: "min",
    },
    "first" => BuiltInFunctionObject {
        function: first,
        function_name: "first",
    },
    "last" => BuiltInFunctionObject {
        function: last,
        function_name: "last",
    },
    "rest" => BuiltInFunctionObject {
        function: rest,
        function_name: "rest",
    },
    "push" => BuiltInFunctionObject {
        function: push,
        function_name: "push",
    },
    "print" => BuiltInFunctionObject {
        function: print,
        function_name: "print",
    },
    "println" => BuiltInFunctionObject {
        function: println,
        function_name: "println",
    },
    "range" => BuiltInFunctionObject {
        function: range,
        function_name: "range",
    },
    "str" => BuiltInFunctionObject {
        function: str,
        function_name: "str",
    },
    "string" => BuiltInFunctionObject {
        function: string,
        function_name: "string",
    },
    "int" => BuiltInFunctionObject {
        function: int,
        function_name: "int",
    },
    "float" => BuiltInFunctionObject {
        function: float,
        function_name: "float",
    },
    "type" => BuiltInFunctionObject {
        function: _type,
        function_name: "type",
    },
};

fn len(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::String(s)] => Ok(Object::Integer(IntegerObject {
            value: s.value.len() as i64,
        })),
        [Object::Array(arr)] => Ok(Object::Integer(IntegerObject {
            value: arr.elements.try_borrow()?.len() as i64,
        })),
        [arg] => Err(ErrorObject {
            msg: format!(
                "argument to `len` not supported, expected String or Array, got {}",
                arg.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `len`. got={}, want=1",
                args.len()
            ),
        }),
    }
}

fn max(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Integer(l), Object::Integer(r)] => Ok(Object::Integer(IntegerObject {
            value: std::cmp::max(l.value, r.value),
        })),
        [Object::Float(l), Object::Float(r)] => Ok(Object::Float(FloatObject {
            value: l.value.max(r.value),
        })),
        [Object::Integer(l), Object::Float(r)] => Ok(Object::Float(FloatObject {
            value: r.value.max(l.value as f64),
        })),
        [Object::Float(l), Object::Integer(r)] => Ok(Object::Float(FloatObject {
            value: l.value.max(r.value as f64),
        })),
        [l, r] => Err(ErrorObject {
            msg: format!(
                "arguments to `max` not supported, expected (Integer || Float) and (Integer || Float), got {} and {}",
                l.typ(),
                r.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `max`. got={}, want=2",
                args.len()
            ),
        }),
    }
}

fn min(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Integer(l), Object::Integer(r)] => Ok(Object::Integer(IntegerObject {
            value: std::cmp::min(l.value, r.value),
        })),
        [Object::Float(l), Object::Float(r)] => Ok(Object::Float(FloatObject {
            value: l.value.min(r.value),
        })),
        [Object::Integer(l), Object::Float(r)] => Ok(Object::Float(FloatObject {
            value: r.value.min(l.value as f64),
        })),
        [Object::Float(l), Object::Integer(r)] => Ok(Object::Float(FloatObject {
            value: l.value.min(r.value as f64),
        })),
        [l, r] => Err(ErrorObject {
            msg: format!(
                "arguments to `min` not supported, expected (Integer || Float) and (Integer || Float), got {} and {}",
                l.typ(),
                r.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `min`. got={}, want=2",
                args.len()
            ),
        }),
    }
}

fn first(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Array(arr)] => Ok(arr
            .elements
            .try_borrow()?
            .first()
            .cloned()
            .unwrap_or(Object::NULL)),
        [o] => Err(ErrorObject {
            msg: format!(
                "arguments to `first` not supported, expected array, got {}",
                o.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `first`. got={}, want=1",
                args.len()
            ),
        }),
    }
}

fn last(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Array(arr)] => Ok(arr
            .elements
            .try_borrow()?
            .last()
            .cloned()
            .unwrap_or(Object::NULL)),
        [o] => Err(ErrorObject {
            msg: format!(
                "arguments to `last` not supported, expected array, got {}",
                o.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `last`. got={}, want=1",
                args.len()
            ),
        }),
    }
}

fn rest(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Array(arr)] => {
            let vec = arr.elements.try_borrow()?;
            Ok(Object::Array(ArrayObject::new(
                vec.get(1..).unwrap_or(&[]).to_vec(),
            )))
        }
        [o] => Err(ErrorObject {
            msg: format!(
                "arguments to `rest` not supported, expected array, got {}",
                o.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `rest`. got={}, want=1",
                args.len()
            ),
        }),
    }
}

fn push(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Array(arr), itm] => {
            arr.elements.try_borrow_mut()?.push(itm.clone());
            Ok(Object::Array(arr.clone()))
        }
        [Object::Array(_), ..] => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `push`. got={}, want=2",
                args.len()
            ),
        }),
        [o, ..] => Err(ErrorObject {
            msg: format!(
                "argument to `push` not supported, expected Array, got {}",
                o.typ()
            ),
        }),
        [] => Err(ErrorObject {
            msg: "wrong number of arguments to `push`. got=0, want=2".to_string(),
        }),
    }
}

fn range(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    let (start, end) = match args {
        [Object::Integer(end)] => (0, end.value),
        [Object::Integer(start), Object::Integer(end)] => (start.value, end.value),
        [end] => {
            return Err(ErrorObject {
                msg: format!(
                    "argument to `range` not supported, expected Integer, got {}",
                    end.typ()
                ),
            });
        }
        [start, end] => {
            return Err(ErrorObject {
                msg: format!(
                    "arguments to `range` not supported, expected Integer and Integer, got {} and {}",
                    start.typ(),
                    end.typ()
                ),
            });
        }
        _ => {
            return Err(ErrorObject {
                msg: format!(
                    "wrong number of arguments to `range`. got={}, want=1 or 2",
                    args.len()
                ),
            });
        }
    };

    let elements = (start..end)
        .map(|value| Object::Integer(IntegerObject { value }))
        .collect::<Vec<_>>();

    Ok(Object::Array(ArrayObject::new(elements)))
}

fn print(args: &[Object], env: &Env) -> Result<Object, ErrorObject> {
    let text = args
        .iter()
        .map(|arg| arg.inspect_value())
        .collect::<Vec<_>>()
        .join(" ");
    env.borrow().write_output(&text);

    Ok(Object::NULL)
}

fn println(args: &[Object], env: &Env) -> Result<Object, ErrorObject> {
    let text = args
        .iter()
        .map(|arg| arg.inspect_value())
        .collect::<Vec<_>>()
        .join(" ");
    env.borrow().write_output(&format!("{text}\n"));

    Ok(Object::NULL)
}

fn str(args: &[Object], env: &Env) -> Result<Object, ErrorObject> {
    env.borrow()
        .write_output("[WARNING] str() is deprecated, please use string() instead");
    string(args, env)
}

fn string(args: &[Object], _: &Env) -> Result<Object, ErrorObject> {
    match args {
        [o] => Ok(Object::String(StringObject {
            value: o.inspect_value().into(),
        })),
        _ => Err(ErrorObject {
            msg: format!("expected 1 argument to string(), received {}", args.len()),
        }),
    }
}

fn int(args: &[Object], _: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Float(o)] => Ok(Object::Integer(IntegerObject {
            value: unsafe { o.value.to_int_unchecked() },
        })),
        [Object::Integer(o)] => Ok(Object::Integer(o.clone())),
        [Object::Boolean(o)] => Ok(Object::Integer(IntegerObject {
            value: if o.value { 1 } else { 0 },
        })),
        [Object::NULL] => Ok(Object::Integer(IntegerObject { value: 0 })),
        [o] => Err(ErrorObject {
            msg: format!("{} cannot be coerced to an int", o.typ()),
        }),
        _ => Err(ErrorObject {
            msg: format!("expected 1 argument to int(), received {}", args.len()),
        }),
    }
}

fn float(args: &[Object], _: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Float(o)] => Ok(Object::Float(o.clone())),
        [Object::Integer(o)] => Ok(Object::Float(FloatObject {
            value: o.value as f64,
        })),
        [Object::Boolean(o)] => Ok(Object::Float(FloatObject {
            value: if o.value { 1.0f64 } else { 0.0f64 },
        })),
        [Object::NULL] => Ok(Object::Integer(IntegerObject { value: 0 })),
        [o] => Err(ErrorObject {
            msg: format!("{} cannot be coerced to an int", o.typ()),
        }),
        _ => Err(ErrorObject {
            msg: format!("expected 1 argument to int(), received {}", args.len()),
        }),
    }
}

fn _type(args: &[Object], _: &Env) -> Result<Object, ErrorObject> {
    match args {
        [o] => Ok(Object::String(StringObject {
            value: o.typ().to_string().into(),
        })),
        _ => Err(ErrorObject {
            msg: format!("expected 1 argument to type(), received {}", args.len()),
        }),
    }
}
