use std::fmt::{Display as FmtDisplay, Formatter, Result as FmtResult};

use strum::Display;

use crate::{
    ast::{BlockStatement, IdentifierExpression},
    object::environment::Environment,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
    Error,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object<'a> {
    Integer(IntegerObject),
    Boolean(BooleanObject),
    Null(NullObject),
    ReturnValue(ReturnValueObject<'a>),
    Function(FunctionObject<'a>),
}

impl Object<'static> {
    pub const NULL: Object<'static> = Object::Null(NullObject {});
    pub const TRUE: Object<'static> = Object::Boolean(BooleanObject { value: true });
    pub const FALSE: Object<'static> = Object::Boolean(BooleanObject { value: false });
}

pub trait Objecter {
    fn typ(&self) -> ObjectType;
    fn inspect_value(&self) -> String;
}

impl<'a> Objecter for Object<'a> {
    fn typ(&self) -> ObjectType {
        match self {
            Object::Integer(o) => o.typ(),
            Object::Boolean(o) => o.typ(),
            Object::Null(o) => o.typ(),
            Object::ReturnValue(o) => o.typ(),
            Object::Function(o) => o.typ(),
        }
    }

    fn inspect_value(&self) -> String {
        match self {
            Object::Integer(o) => o.inspect_value(),
            Object::Boolean(o) => o.inspect_value(),
            Object::Null(o) => o.inspect_value(),
            Object::ReturnValue(o) => o.inspect_value(),
            Object::Function(o) => o.inspect_value(),
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
pub struct ReturnValueObject<'a> {
    pub value: Box<Object<'a>>,
}

impl<'a> Objecter for ReturnValueObject<'a> {
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
        format!("ERROR: {msg}")
    }
}

impl FmtDisplay for ErrorObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inspect_value())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionObject<'a> {
    pub params: Vec<IdentifierExpression<'a>>,
    pub body: Option<BlockStatement<'a>>,
    pub env: Environment<'a>,
}

impl<'a> Objecter for FunctionObject<'a> {
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
