use std::collections::HashMap;

use crate::object::Object;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment<'a> {
    pub store: HashMap<String, Object<'a>>,
}
impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Self {
            store: HashMap::default(),
        }
    }
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            store: HashMap::default(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object<'a>> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, v: Object<'a>) -> Option<Object<'a>> {
        self.store.insert(name, v)
    }
}
