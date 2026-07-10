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
    fn typ() -> ObjectType;
    fn inspect(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct IntegerObject {
    pub value: i64,
}

impl Objecter for IntegerObject {
    fn typ() -> ObjectType {
        ObjectType::Integer
    }

    fn inspect(&self) -> String {
        let v = self.value;
        format!("{v}")
    }
}

#[derive(Debug, Clone)]
pub struct BooleanObject {
    pub value: bool,
}

impl Objecter for BooleanObject {
    fn typ() -> ObjectType {
        ObjectType::Boolean
    }

    fn inspect(&self) -> String {
        let v = self.value;
        format!("{v}")
    }
}

#[derive(Debug, Clone)]
pub struct NullObject {}

impl Objecter for NullObject {
    fn typ() -> ObjectType {
        ObjectType::Null
    }

    fn inspect(&self) -> String {
        "null".to_string()
    }
}
