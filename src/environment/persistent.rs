/**
 * This approach to environment uses persistent data structures for environments
 * so a closure always sees the env when they were created.
 *
 * Not very efficient as a new Env is created each time a var is declared
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
    current: SharedMutableEnvironment,
    checkpoints: Vec<SharedMutableEnvironment>,
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
            global: global.clone(),
            current: global,
            checkpoints: vec![],
        }
    }

    pub fn push(&mut self) {
        // store the scope we entered
        self.checkpoints.push(self.current());
        // track the new current scope
        self.current = Rc::new(RefCell::new(Environment::new(Some(self.current()))));
    }

    pub fn push_with_locals(&mut self, parent_env: SharedMutableEnvironment, locals: Locals) {
        self.checkpoints.push(self.current());

        self.current = Rc::new(RefCell::new(Environment::with_values(
            Some(parent_env),
            locals,
        )));
    }

    pub fn pop(&mut self) -> bool {
        match self.checkpoints.pop() {
            None => false,
            Some(restore_point) => {
                self.current = restore_point;
                true
            }
        }
    }

    pub fn current(&self) -> SharedMutableEnvironment {
        self.current.clone()
    }

    pub fn declare_global(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        self.global.borrow_mut().declare(variable, value)
    }

    pub fn declare(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        // every new variable declaration creates a new env but not a checkpoint
        self.current = Rc::new(RefCell::new(Environment::new(Some(self.current()))));

        self.current().borrow_mut().declare(variable, value)
    }

    pub fn assign(&mut self, variable: String, value: Value) -> Result<(), RuntimeError> {
        self.current().borrow_mut().assign(variable, value)
    }

    pub fn get(&self, variable: &str) -> Result<Value, RuntimeError> {
        self.current().borrow().get(variable)
    }
}
