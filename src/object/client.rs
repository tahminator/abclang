#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(IntegerObject),
    Boolean(BooleanObject),
    Null(NullObject),
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
        }
    }

    fn inspect_value(&self) -> String {
        match self {
            Object::Integer(o) => o.inspect_value(),
            Object::Boolean(o) => o.inspect_value(),
            Object::Null(o) => o.inspect_value(),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct NullObject {}

impl Objecter for NullObject {
    fn typ(&self) -> ObjectType {
        ObjectType::Null
    }

    fn inspect_value(&self) -> String {
        "null".to_string()
    }
}
