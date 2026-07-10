use std::{collections::HashMap, rc::Rc};

use crate::object::Object;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment<'a> {
    pub store: HashMap<String, Object<'a>>,
    pub outer: Option<Rc<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_enclosed(outer: Rc<Environment<'a>>) -> Self {
        let mut env = Self::new();
        env.outer = Some(outer);

        env
    }

    pub fn default() -> Self {
        Self {
            store: HashMap::default(),
            outer: None,
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object<'a>> {
        match self.store.get(name) {
            Some(v) => Some(v),
            None => self.outer.as_ref()?.get(name),
        }
    }

    pub fn set(&mut self, name: String, v: Object<'a>) -> Option<Object<'a>> {
        self.store.insert(name, v)
    }
}
