/**
 * This is the original implementation of the EnvStack with a closure bug
 * closures wouldn't see the env at the point they were created, but the entire env, which leads to correctness bugs
 */
use crate::{errors::RuntimeError, interpreter::Value};
use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    mem::discriminant,
    rc::Rc,
};

pub type SharedMutableEnvironment = Rc<RefCell<Environment>>;
pub type Locals = HashMap<String, Value>;

#[derive(Debug)]
pub struct Environment {
    values: Locals,
    parent: Option<SharedMutableEnvironment>,
}

#[derive(Debug)]
pub struct EnvironmentStack {
    global: SharedMutableEnvironment,
    stack: Vec<SharedMutableEnvironment>,
}

impl Environment {
    pub fn new(parent: Option<SharedMutableEnvironment>) -> Self {
        Self {
            values: HashMap::new(),
            parent,
        }
    }

    pub fn with_values(parent: Option<SharedMutableEnvironment>, values: Locals) -> Self {
        Self { values, parent }
    }

    pub fn declare(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        match self.values.entry(variable.clone()) {
            Entry::Occupied(_) => Err(RuntimeError(format!(
                "Variable {variable} already declared."
            ))),

            Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    pub fn assign(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        match self.values.entry(variable.clone()) {
            Entry::Occupied(mut entry) => {
                let current = entry.get();
                // TODO we'll need the ability to cast values if we prevent assignments that change types
                if discriminant(current) != discriminant(&value) {
                    return Err(RuntimeError(format!(
                        "cannot change type of {variable} from {} to {}",
                        current.type_name(),
                        value.type_name()
                    )));
                }
                entry.insert(value);
                Ok(())
            }
            Entry::Vacant(_) => match &self.parent {
                None => Err(RuntimeError(format!("Undefined variable {variable}."))),
                Some(p) => p.borrow_mut().assign(variable, value),
            },
        }
    }

    pub fn get(&self, variable: &str) -> Result<Value, RuntimeError> {
        match self.values.get(variable) {
            Some(result) => Ok(result.clone()),
            None => match &self.parent {
                None => Err(RuntimeError(format!("Undefined variable {variable}."))),
                Some(p) => p.borrow().get(variable),
            },
        }
    }
}

impl EnvironmentStack {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment::new(None)));
        Self {
            global,
            stack: vec![],
        }
    }

    pub fn push(&mut self) {
        let new_env = Rc::new(RefCell::new(Environment::new(Some(self.current()))));
        self.stack.push(new_env);
    }

    pub fn push_with_locals(&mut self, parent_env: SharedMutableEnvironment, locals: Locals) {
        let new_env = Rc::new(RefCell::new(Environment::with_values(
            Some(parent_env),
            locals,
        )));
        self.stack.push(new_env);
    }

    pub fn pop(&mut self) -> bool {
        match self.stack.pop() {
            None => false,
            Some(_) => true,
        }
    }

    pub fn current(&self) -> SharedMutableEnvironment {
        Rc::clone(self.stack.last().unwrap_or(&self.global))
    }

    pub fn declare_global(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        self.global.borrow_mut().declare(variable, value)
    }

    pub fn declare(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        self.current().borrow_mut().declare(variable, value)
    }

    pub fn assign(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        self.current().borrow_mut().assign(variable, value)
    }

    pub fn get(&self, variable: &str) -> Result<Value, RuntimeError> {
        self.current().borrow().get(variable)
    }
}
