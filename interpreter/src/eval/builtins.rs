use phf::phf_map;

use crate::eval::object::{
    ArrayObject, BuiltInFunctionObject, ErrorObject, IntegerObject, Object, Objecter,
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
    }
};

fn len(args: &[Object]) -> Result<Object, ErrorObject> {
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

fn max(args: &[Object]) -> Result<Object, ErrorObject> {
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

fn min(args: &[Object]) -> Result<Object, ErrorObject> {
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

fn first(args: &[Object]) -> Result<Object, ErrorObject> {
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

fn last(args: &[Object]) -> Result<Object, ErrorObject> {
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

fn rest(args: &[Object]) -> Result<Object, ErrorObject> {
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

fn append(args: &[Object]) -> Result<Object, ErrorObject> {
    match args {
        [Object::Array(arr), itm] => {
            let mut clone = arr.elements.to_vec();
            clone.push(itm.clone());
            Ok(Object::Array(ArrayObject { elements: clone }))
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

fn print(args: &[Object]) -> Result<Object, ErrorObject> {
    for arg in args.iter() {
        println!("{}", arg.inspect_value())
    }

    Ok(Object::NULL)
}
