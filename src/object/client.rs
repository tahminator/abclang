use std::fmt::{Display as FmtDisplay, Formatter, Result as FmtResult};

use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(IntegerObject),
    Boolean(BooleanObject),
    Null(NullObject),
    ReturnValue(ReturnValueObject),
}

impl Object {
    pub const NULL: Object = Object::Null(NullObject {});
    pub const TRUE: Object = Object::Boolean(BooleanObject { value: true });
    pub const FALSE: Object = Object::Boolean(BooleanObject { value: false });
}

pub trait Objecter {
    fn typ(&self) -> ObjectType;
    fn inspect_value(&self) -> String;
}

impl Objecter for Object {
    fn typ(&self) -> ObjectType {
        match self {
            Object::Integer(o) => o.typ(),
            Object::Boolean(o) => o.typ(),
            Object::Null(o) => o.typ(),
            Object::ReturnValue(o) => o.typ(),
        }
    }

    fn inspect_value(&self) -> String {
        match self {
            Object::Integer(o) => o.inspect_value(),
            Object::Boolean(o) => o.inspect_value(),
            Object::Null(o) => o.inspect_value(),
            Object::ReturnValue(o) => o.inspect_value(),
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

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorObject {
    pub msg: String,
}

impl Objecter for ErrorObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Error
    }

    fn inspect_value(&self) -> String {
        let msg = &self.msg;
        return format!("ERROR: {msg}");
    }
}

impl FmtDisplay for ErrorObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inspect_value())
    }
}
