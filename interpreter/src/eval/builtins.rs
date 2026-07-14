use phf::phf_map;

use crate::eval::object::{
    ArrayObject, BuiltInFunctionObject, ErrorObject, HashObject, IntegerObject, Object,
    ObjectHasher, Objecter, environment::Env,
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
    "append" => BuiltInFunctionObject {
        function: append,
        function_name: "append",
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
    }
};

fn len(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::String(s)] => Ok(Object::Integer(IntegerObject {
            value: s.value.len() as i64,
        })),
        [Object::Array(arr)] => Ok(Object::Integer(IntegerObject {
            value: arr.elements.len() as i64,
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
        [l, r] => Err(ErrorObject {
            msg: format!(
                "arguments to `max` not supported, expected Integer and Integer, got {} and {}",
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
        [l, r] => Err(ErrorObject {
            msg: format!(
                "arguments to `min` not supported, expected Integer and Integer, got {} and {}",
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
        [Object::Array(arr)] => Ok(arr.elements.first().cloned().unwrap_or(Object::NULL)),
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
        [Object::Array(arr)] => Ok(arr.elements.last().cloned().unwrap_or(Object::NULL)),
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
        [Object::Array(arr)] => Ok(Object::Array(ArrayObject {
            elements: arr.elements[1..arr.elements.len()].to_vec(),
        })),
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

fn append(args: &[Object], _env: &Env) -> Result<Object, ErrorObject> {
    match args {
        [Object::Array(arr), itm] => {
            let mut clone = arr.elements.to_vec();
            clone.push(itm.clone());
            Ok(Object::Array(ArrayObject { elements: clone }))
        }
        [Object::Hash(hash), key, value] => {
            let hashed = key.hash_key().ok_or_else(|| ErrorObject {
                msg: format!("{} is unusable as a hash key", key.typ()),
            })?;

            let mut pairs = hash.pairs.clone();
            pairs.insert(hashed, (key.clone(), value.clone()));

            Ok(Object::Hash(HashObject { pairs }))
        }
        [Object::Array(_), ..] => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `append` for Array. got={}, want=2",
                args.len()
            ),
        }),
        [Object::Hash(_), ..] => Err(ErrorObject {
            msg: format!(
                "wrong number of arguments to `append` for Hash. got={}, want=3",
                args.len()
            ),
        }),
        [o, ..] => Err(ErrorObject {
            msg: format!(
                "argument to `append` not supported, expected Array or Hash, got {}",
                o.typ()
            ),
        }),
        [] => Err(ErrorObject {
            msg: "wrong number of arguments to `append`. got=0, want=2 for array or 3 for map"
                .to_string(),
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
        .collect();

    Ok(Object::Array(ArrayObject { elements }))
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
