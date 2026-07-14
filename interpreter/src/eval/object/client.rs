use std::{
    cell::{BorrowError, BorrowMutError, RefCell},
    collections::HashMap,
    fmt::{Display as FmtDisplay, Formatter, Result as FmtResult},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use strum::Display;

use crate::{
    ast::{BlockStatement, IdentifierExpression},
    eval::object::environment::Env,
};

pub trait Objecter {
    fn typ(&self) -> ObjectType;
    fn inspect_value(&self) -> String;
}

// TODO: optimize perf & possibly include more Objects / handle more gracefully.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashKey {
    pub typ: ObjectType,
    pub value: u64,
}

pub trait ObjectHasher {
    fn hash_key(&self) -> Option<HashKey>;
}

impl Object {
    pub const NULL: Object = Object::Null(NullObject {});
    pub const TRUE: Object = Object::Boolean(BooleanObject { value: true });
    pub const FALSE: Object = Object::Boolean(BooleanObject { value: false });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
    Error,
    Function,
    String,
    BuiltIn,
    Array,
    Hash,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(IntegerObject),
    Boolean(BooleanObject),
    Null(NullObject),
    ReturnValue(ReturnValueObject),
    Function(FunctionObject),
    String(StringObject),
    BuiltIn(BuiltInFunctionObject),
    Array(ArrayObject),
    Hash(HashObject),
}

impl Objecter for Object {
    fn typ(&self) -> ObjectType {
        match self {
            Object::Integer(o) => o.typ(),
            Object::Boolean(o) => o.typ(),
            Object::Null(o) => o.typ(),
            Object::ReturnValue(o) => o.typ(),
            Object::Function(o) => o.typ(),
            Object::String(o) => o.typ(),
            Object::BuiltIn(o) => o.typ(),
            Object::Array(o) => o.typ(),
            Object::Hash(o) => o.typ(),
        }
    }

    fn inspect_value(&self) -> String {
        match self {
            Object::Integer(o) => o.inspect_value(),
            Object::Boolean(o) => o.inspect_value(),
            Object::Null(o) => o.inspect_value(),
            Object::ReturnValue(o) => o.inspect_value(),
            Object::Function(o) => o.inspect_value(),
            Object::String(o) => o.inspect_value(),
            Object::BuiltIn(o) => o.inspect_value(),
            Object::Array(o) => o.inspect_value(),
            Object::Hash(o) => o.inspect_value(),
        }
    }
}

impl ObjectHasher for Object {
    fn hash_key(&self) -> Option<HashKey> {
        match self {
            Object::Integer(o) => o.hash_key(),
            Object::Boolean(o) => o.hash_key(),
            Object::Null(o) => o.hash_key(),
            Object::ReturnValue(o) => o.hash_key(),
            Object::Function(o) => o.hash_key(),
            Object::String(o) => o.hash_key(),
            Object::BuiltIn(o) => o.hash_key(),
            Object::Array(o) => o.hash_key(),
            Object::Hash(o) => o.hash_key(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerObject {
    pub value: i64,
}

impl Objecter for IntegerObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Integer
    }

    fn inspect_value(&self) -> String {
        let v = self.value;
        format!("{v}")
    }
}

impl ObjectHasher for IntegerObject {
    fn hash_key(&self) -> Option<HashKey> {
        Some(HashKey {
            typ: self.typ(),
            value: self.value as u64,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanObject {
    pub value: bool,
}

impl Objecter for BooleanObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Boolean
    }

    fn inspect_value(&self) -> String {
        let v = self.value;
        format!("{v}")
    }
}

impl ObjectHasher for BooleanObject {
    fn hash_key(&self) -> Option<HashKey> {
        Some(HashKey {
            typ: self.typ(),
            value: if self.value { 1 } else { 0 },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullObject {}

impl Objecter for NullObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Null
    }

    fn inspect_value(&self) -> String {
        "null".to_string()
    }
}

impl ObjectHasher for NullObject {
    fn hash_key(&self) -> Option<HashKey> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnValueObject {
    pub value: Box<Object>,
}

impl Objecter for ReturnValueObject {
    fn typ(&self) -> ObjectType {
        ObjectType::ReturnValue
    }

    fn inspect_value(&self) -> String {
        self.value.inspect_value()
    }
}

impl ObjectHasher for ReturnValueObject {
    fn hash_key(&self) -> Option<HashKey> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorObject {
    pub msg: String,
}

impl From<BorrowError> for ErrorObject {
    fn from(e: BorrowError) -> Self {
        ErrorObject {
            msg: format!(
                "an evaluation error occured that is likely a bug that is not caused by you. Details below:\n\n{e:#?}"
            ),
        }
    }
}

impl From<BorrowMutError> for ErrorObject {
    fn from(e: BorrowMutError) -> Self {
        ErrorObject {
            msg: format!(
                "an evaluation error occured that is likely a bug that is not caused by you. Details below:\n\n{e:#?}"
            ),
        }
    }
}

impl Objecter for ErrorObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Error
    }

    fn inspect_value(&self) -> String {
        let msg = &self.msg;
        format!("ERROR: {msg}")
    }
}

impl FmtDisplay for ErrorObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inspect_value())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionObject {
    pub params: Vec<IdentifierExpression>,
    pub body: Option<BlockStatement>,
    pub env: Env,
}

impl ObjectHasher for FunctionObject {
    fn hash_key(&self) -> Option<HashKey> {
        None
    }
}

impl Objecter for FunctionObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Function
    }

    fn inspect_value(&self) -> String {
        format!(
            "fn({}) {{\n{}\n}}",
            self.params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.body
                .as_ref()
                .map(|b| b.to_string())
                .unwrap_or_else(|| "None".to_string())
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringObject {
    pub value: Rc<str>,
}

impl Objecter for StringObject {
    fn typ(&self) -> ObjectType {
        ObjectType::String
    }

    fn inspect_value(&self) -> String {
        self.value.to_string()
    }
}

impl ObjectHasher for StringObject {
    fn hash_key(&self) -> Option<HashKey> {
        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        let hash = hasher.finish();

        Some(HashKey {
            typ: self.typ(),
            value: hash,
        })
    }
}

type BuiltInFunction = fn(args: &[Object], env: &Env) -> Result<Object, ErrorObject>;

#[derive(Debug, Clone)]
pub struct BuiltInFunctionObject {
    pub function: BuiltInFunction,
    pub function_name: &'static str,
}

impl PartialEq for BuiltInFunctionObject {
    fn eq(&self, other: &Self) -> bool {
        self.function_name == other.function_name
    }
}

impl Objecter for BuiltInFunctionObject {
    fn typ(&self) -> ObjectType {
        ObjectType::BuiltIn
    }

    fn inspect_value(&self) -> String {
        "builtin function".into()
    }
}

impl ObjectHasher for BuiltInFunctionObject {
    fn hash_key(&self) -> Option<HashKey> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayObject {
    pub elements: Rc<RefCell<Vec<Object>>>,
}

impl ArrayObject {
    pub fn new(vec: Vec<Object>) -> Self {
        ArrayObject {
            elements: Rc::new(RefCell::new(vec)),
        }
    }
}

impl Objecter for ArrayObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Array
    }

    fn inspect_value(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .try_borrow()
                .map(|r| r
                    .iter()
                    .map(|el| el.inspect_value())
                    .collect::<Vec<_>>()
                    .join(", "))
                .unwrap_or_else(|e| format!("[<array: borrow error>]\n{e:?}"))
        )
    }
}

impl ObjectHasher for ArrayObject {
    fn hash_key(&self) -> Option<HashKey> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HashObject {
    pub pairs: Rc<RefCell<HashMap<HashKey, (Object, Object)>>>,
}

impl HashObject {
    pub fn new(pairs: HashMap<HashKey, (Object, Object)>) -> Self {
        Self {
            pairs: Rc::new(RefCell::new(pairs)),
        }
    }
}

impl Objecter for HashObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Hash
    }

    fn inspect_value(&self) -> String {
        format!(
            "{{{}}}",
            self.pairs
                .try_borrow()
                .map(|r| r
                    .iter()
                    .map(|(_, (k, v))| format!("{}: {}", k.inspect_value(), v.inspect_value()))
                    .collect::<Vec<_>>()
                    .join(", "))
                .unwrap_or_else(|e| format!("{{<map: borrow error>}}\n {e}"))
        )
    }
}

impl ObjectHasher for HashObject {
    fn hash_key(&self) -> Option<HashKey> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_hash_key() {
        let hello1 = StringObject {
            value: "Hello World".into(),
        };
        let hello2 = StringObject {
            value: "Hello World".into(),
        };
        let diff1 = StringObject {
            value: "My name is john".into(),
        };
        let diff2 = StringObject {
            value: "My name is john".into(),
        };

        if hello1.hash_key() != hello2.hash_key() {
            panic!(
                "strings with same context have different hash keys. {} v {}",
                hello1.hash_key().unwrap().value,
                hello2.hash_key().unwrap().value
            )
        }

        if diff1.hash_key() != diff2.hash_key() {
            panic!(
                "strings with same context have different hash keys. {} v {}",
                diff1.hash_key().unwrap().value,
                diff2.hash_key().unwrap().value
            )
        }

        if hello1.hash_key() == diff1.hash_key() {
            panic!(
                "strings with different context have same hash keys. {} v {}",
                hello1.hash_key().unwrap().value,
                diff1.hash_key().unwrap().value
            )
        }
    }
}
