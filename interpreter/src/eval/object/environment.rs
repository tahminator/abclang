use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::eval::object::Object;

pub type Env = Rc<RefCell<Environment>>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub outer: Option<Env>,
}

impl Environment {
    pub fn new() -> Env {
        Rc::new(RefCell::new(Self::default()))
    }

    pub fn new_enclosed(outer: Env) -> Env {
        Rc::new(RefCell::new(Self {
            store: HashMap::default(),
            outer: Some(outer),
        }))
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(v) => Some(v.clone()),
            None => self.outer.as_ref().and_then(|o| o.borrow().get(name)),
        }
    }

    pub fn set(&mut self, name: String, v: Object) {
        self.store.insert(name, v);
    }
}
