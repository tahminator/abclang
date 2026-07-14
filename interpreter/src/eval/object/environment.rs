use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::eval::object::Object;

pub type Env = Rc<RefCell<Environment>>;

/// shared print buffer. each env decides what to do after eval (REPL, web)
pub type Output = Rc<RefCell<String>>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub outer: Option<Env>,
    pub output: Output,
}

impl Environment {
    pub fn new() -> Env {
        Rc::new(RefCell::new(Self::default()))
    }

    pub fn new_enclosed(outer: Env) -> Env {
        let output = outer.borrow().output.clone();
        Rc::new(RefCell::new(Self {
            store: HashMap::default(),
            outer: Some(outer),
            output,
        }))
    }

    pub fn write_output(&self, text: &str) {
        self.output.borrow_mut().push_str(text);
    }

    pub fn take_output(&self) -> String {
        std::mem::take(&mut self.output.borrow_mut())
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
