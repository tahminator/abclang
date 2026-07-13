use phf::phf_map;

use crate::object::{BuiltInFunctionObject, ErrorObject, IntegerObject, Object, Objecter};

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
    }
};

fn len(args: &[Object]) -> Result<Object, ErrorObject> {
    match args {
        [Object::String(s)] => Ok(Object::Integer(IntegerObject {
            value: s.value.len() as i64,
        })),
        [arg] => Err(ErrorObject {
            msg: format!("argument to `len` not supported, got {}", arg.typ()),
        }),
        _ => Err(ErrorObject {
            msg: format!("wrong number of arguments. got={}, want=1", args.len()),
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
                "arguments to `max` not supported, got {} and {}",
                l.typ(),
                r.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!("wrong number of arguments. got={}, want=2", args.len()),
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
                "arguments to `min` not supported, got {} and {}",
                l.typ(),
                r.typ()
            ),
        }),
        _ => Err(ErrorObject {
            msg: format!("wrong number of arguments. got={}, want=2", args.len()),
        }),
    }
}
