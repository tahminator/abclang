use std::collections::HashMap;

use crate::object::Object;

pub struct Environment {
    pub store: HashMap<String, Object>,
}
impl Default for Environment {
    fn default() -> Self {
        Self {
            store: HashMap::default(),
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            store: HashMap::default(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, v: Object) -> Option<Object> {
        self.store.insert(name, v)
    }
}
