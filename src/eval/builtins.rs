use phf::phf_map;

use crate::object::{BuiltInFunctionObject, ErrorObject, IntegerObject, Object, Objecter};

pub static BUILTINS: phf::Map<&'static str, BuiltInFunctionObject> = phf_map! {
    "len" => BuiltInFunctionObject {
        function: len,
        function_name: "len",
    },
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
