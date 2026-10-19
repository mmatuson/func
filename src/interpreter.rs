use std::{fmt, rc::Rc};

use crate::{
    environment::{self, EnvironmentStack, Locals},
    native,
    parser::{
        self, BinaryOperator,
        Expression::{
            self, AssignmentExpression, BinaryExpression, CallExpression, GroupingExpression,
            LiteralExpression, LogicalExpression, UnaryExpression, VariableExpression,
        },
        Literal, LogicalOperator, Statement, UnaryOperator,
    },
};

use crate::errors::RuntimeError;
#[derive(Debug)]
pub struct FunctionDefinition {
    params: Vec<String>,
    body: Rc<Vec<Statement>>,
}

#[derive(Debug)]
pub struct UserFunction {
    name: String,
    definition: Rc<FunctionDefinition>,
    closure: environment::SharedMutableEnvironment,
}

#[derive(Debug)]
pub enum Arity {
    Nary(usize),
    Indefinite, //variadic
}
#[derive(Debug)]
pub struct NativeFunction {
    arity: Arity,
    name: String,
    function: fn(&[Value]) -> Result<Value, RuntimeError>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    UserFunction(Rc<UserFunction>),
    NativeFunction(Rc<NativeFunction>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "Nil",
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
            Value::NativeFunction(_) => "NativeFunction",
            Value::UserFunction(_) => "UserFunction",
        }
    }

    pub fn is_truthy(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Nil => Ok(false),
            Value::Bool(v) => Ok(*v),
            // I'm not sure if I want to treat Ints and Strings as bool with logical not
            Value::Integer(v) => Ok(*v != 0),
            Value::Float(v) => Ok(*v != 0.0),
            Value::String(v) => Ok(!v.is_empty()),
            Value::UserFunction(_) => Ok(true),
            Value::NativeFunction(_) => Ok(true),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Integer(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::UserFunction(_) => write!(f, "<function>"),
        }
    }
}

impl From<Literal> for Value {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Integer(v) => Value::Integer(v),
            Literal::Float(v) => Value::Float(v),
            Literal::String(v) => Value::String(v),
            Literal::Bool(v) => Value::Bool(v),
        }
    }
}

impl From<&Literal> for Value {
    fn from(l: &Literal) -> Self {
        match l {
            Literal::Integer(v) => Value::Integer(*v),
            Literal::Float(v) => Value::Float(*v),
            Literal::String(v) => Value::String(v.clone()),
            Literal::Bool(v) => Value::Bool(*v),
        }
    }
}

struct StatementResult {
    flow: ControlFlow,
    value: Value,
}
enum ControlFlow {
    Next,
    Continue,
    Break,
    Return(Value),
}

const NORMAL_CONTROL_FLOW: StatementResult = StatementResult {
    flow: ControlFlow::Next,
    value: Value::Nil,
};

fn add_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs + rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs + rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 + rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs + rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
        (Value::String(lhs), Value::Integer(rhs)) => Ok(Value::String(lhs + &rhs.to_string())),
        (lhs, rhs) => Err(RuntimeError(format!(
            "+ is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn sub_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs - rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs - rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 - rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs - rhs as f64)),
        (lhs, rhs) => Err(RuntimeError(format!(
            "- is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn mul_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs * rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs * rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 * rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs * rhs as f64)),
        (Value::String(lhs), Value::Integer(rhs)) => {
            if rhs < 1 {
                return Err(RuntimeError(format!(
                    "Repeat count must be >=1. got {}",
                    rhs
                )));
            }
            Ok(Value::String(lhs.repeat(rhs as usize)))
        }
        (lhs, rhs) => Err(RuntimeError(format!(
            "* is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn mod_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (_, Value::Integer(rhs)) if rhs == 0 => {
            Err(RuntimeError("Attempt to mod by zero".to_string()))
        }
        (_, Value::Float(rhs)) if rhs == 0.0 => {
            Err(RuntimeError("Attempt to mod by zero".to_string()))
        }
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs % rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs % rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 % rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs % rhs as f64)),
        (lhs, rhs) => Err(RuntimeError(format!(
            "% is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn div_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (_, Value::Integer(rhs)) if rhs == 0 => {
            Err(RuntimeError("Attempt to divide by zero".to_string()))
        }
        (_, Value::Float(rhs)) if rhs == 0.0 => {
            Err(RuntimeError("Attempt to divide by zero".to_string()))
        }
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs / rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs / rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 / rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs / rhs as f64)),
        (lhs, rhs) => Err(RuntimeError(format!(
            "/ is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn negate_value(rhs: Value) -> Result<Value, RuntimeError> {
    match rhs {
        Value::Float(v) => Ok(Value::Float(-v)),
        Value::Integer(v) => {
            let n = v.checked_neg().ok_or_else(|| {
                RuntimeError("Integer overflow during unary negation".to_string())
            })?;
            Ok(Value::Integer(n))
        }
        rhs => Err(RuntimeError(format!(
            "negate (-) is not supported for operand {}",
            rhs
        ))),
    }
}

fn logical_not_value(rhs: Value) -> Result<Value, RuntimeError> {
    let b = rhs.is_truthy()?;
    Ok(Value::Bool(!b))
}

fn compare_equal_to_value(lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs == rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs == rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs as f64 == rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs == rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs.eq(&rhs))),
        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs == rhs)),
        (lhs, rhs) => Err(RuntimeError(format!(
            "== is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn compare_not_equal_to_value(lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs != rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs != rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs as f64 != rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs != rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs.ne(&rhs))),
        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs != rhs)),
        (lhs, rhs) => Err(RuntimeError(format!(
            "!= is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn compare_gt_value(lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs > rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs > rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Bool((lhs as f64) > rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs > rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs.gt(&rhs))),
        (lhs, rhs) => Err(RuntimeError(format!(
            "> is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn compare_gte_value(lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs >= rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs >= rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Bool((lhs as f64) >= rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs >= rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs.ge(&rhs))),
        (lhs, rhs) => Err(RuntimeError(format!(
            ">= is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn compare_lt_value(lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs < rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs < rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Bool((lhs as f64) < rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs < rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs.lt(&rhs))),
        (lhs, rhs) => Err(RuntimeError(format!(
            "< is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

fn compare_lte_value(lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
    match (lhs, rhs) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs <= rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs <= rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Bool((lhs as f64) <= rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Bool(lhs <= rhs as f64)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs.le(&rhs))),
        (lhs, rhs) => Err(RuntimeError(format!(
            "<= is not supported for operands {} and {}",
            lhs, rhs
        ))),
    }
}

impl BinaryOperator {
    fn eval(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match self {
            BinaryOperator::Plus => add_value(left, right),
            BinaryOperator::Minus => sub_value(left, right),
            BinaryOperator::Star => mul_value(left, right),
            BinaryOperator::Div => div_value(left, right),
            BinaryOperator::Mod => mod_value(left, right),
            BinaryOperator::EqualTo => compare_equal_to_value(left, right),
            BinaryOperator::NotEqual => compare_not_equal_to_value(left, right),
            BinaryOperator::GreaterThan => compare_gt_value(left, right),
            BinaryOperator::GreaterThanOrEqualTo => compare_gte_value(left, right),
            BinaryOperator::LessThan => compare_lt_value(left, right),
            BinaryOperator::LessThanOrEqualTo => compare_lte_value(left, right),
        }
    }
}

impl UnaryOperator {
    fn eval(&self, right: Value) -> Result<Value, RuntimeError> {
        match self {
            UnaryOperator::Bang => logical_not_value(right),
            UnaryOperator::LogicalNot => logical_not_value(right),
            UnaryOperator::Minus => negate_value(right),
        }
    }
}

impl LogicalOperator {
    fn eval<FR>(&self, left: Value, mut eval_right: FR) -> Result<Value, RuntimeError>
    where
        FR: FnMut() -> Result<Value, RuntimeError>,
    {
        match self {
            LogicalOperator::LogicalAnd => {
                if !(left.is_truthy()?) {
                    return Ok(left);
                }
            }
            LogicalOperator::LogicalOr => {
                if left.is_truthy()? {
                    return Ok(left);
                }
            }
        }
        let rhs = eval_right()?;
        Ok(rhs)
    }
}

pub struct Interpreter {
    env: EnvironmentStack,
    debug: bool,
}

impl Interpreter {
    pub fn new(debug: bool) -> Self {
        let mut vm = Self {
            env: EnvironmentStack::new(),
            debug,
        };

        vm.register_builtins().expect("failed to register builtins");
        vm
    }

    fn register_builtins(&mut self) -> Result<(), RuntimeError> {
        self.env.declare_global(
            "sleep".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction {
                arity: Arity::Nary(1),
                name: "sleep".to_string(),
                function: native::sleep,
            })),
        )?;

        self.env.declare_global(
            "print".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction {
                arity: Arity::Indefinite,
                name: "print".to_string(),
                function: native::print,
            })),
        )?;

        Ok(())
    }

    fn execute_statements(
        &mut self,
        statements: &[Statement],
    ) -> Result<StatementResult, RuntimeError> {
        let mut last = NORMAL_CONTROL_FLOW;

        for s in statements {
            match self.execute(s) {
                Ok(result) => match result.flow {
                    ControlFlow::Break | ControlFlow::Continue | ControlFlow::Return(_) => {
                        return Ok(result);
                    }
                    ControlFlow::Next => last = result,
                },
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Ok(last)
    }

    fn execute_block_scope(
        &mut self,
        statements: &[Statement],
    ) -> Result<StatementResult, RuntimeError> {
        self.env.push();
        let result = self.execute_statements(statements);
        self.env.pop();

        result
    }

    fn execute_user_function(
        &mut self,
        f: Rc<UserFunction>,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        if args.len() != f.definition.params.len() {
            return Err(RuntimeError(format!(
                "function {} expects {} args. You provided {}",
                f.name,
                f.definition.params.len(),
                args.len()
            )));
        }
        let locals: Locals = f
            .definition
            .params
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect();

        self.env.push_with_locals(Rc::clone(&f.closure), locals);
        let exec_value = self.execute_statements(&f.definition.body);
        self.env.pop();

        let result = exec_value?;
        match result.flow {
            ControlFlow::Continue | ControlFlow::Break => Err(RuntimeError(
                "Cannot use break or continue outside of a loop".to_string(),
            )),
            ControlFlow::Return(v) => Ok(v),
            ControlFlow::Next => Ok(result.value), // Value::Nil to force a return <expr>
        }
    }

    fn execute_native_function(
        &mut self,
        f: Rc<NativeFunction>,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        if let Arity::Nary(arity) = f.arity {
            if args.len() != arity {
                return Err(RuntimeError(format!(
                    "function {} expects {} args. You provided {}",
                    f.name,
                    arity,
                    args.len()
                )));
            }
        }

        (f.function)(args)
    }

    fn execute(&mut self, statement: &Statement) -> Result<StatementResult, RuntimeError> {
        match statement {
            Statement::BreakStatement => Ok({
                StatementResult {
                    flow: ControlFlow::Break,
                    value: Value::Nil,
                }
            }),
            Statement::ContinueStatement => Ok({
                StatementResult {
                    flow: ControlFlow::Continue,
                    value: Value::Nil,
                }
            }),
            Statement::ExpressionStatement(x) => Ok(StatementResult {
                flow: ControlFlow::Next,
                value: self.evaluate(x)?,
            }),

            Statement::BlockStatement(stmts) => self.execute_block_scope(stmts),

            Statement::DeclarationStatement(variable, value) => {
                let result_value = self.evaluate(value)?;
                self.env.declare(variable.clone(), result_value)?;
                Ok(NORMAL_CONTROL_FLOW)
            }

            Statement::WhileStatement(cond, stmts) => {
                loop {
                    if !(self.evaluate(cond)?.is_truthy()?) {
                        break;
                    }
                    let result = self.execute_block_scope(stmts)?;
                    match result.flow {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Next => {}
                        ControlFlow::Return(_) => return Ok(result),
                    }
                }

                Ok(NORMAL_CONTROL_FLOW)
            }

            Statement::FunctionDeclaration(name, params, body) => {
                let f = UserFunction {
                    name: name.clone(),
                    definition: Rc::new(FunctionDefinition {
                        params: params.clone(),
                        body: Rc::clone(body),
                    }),
                    closure: self.env.current(),
                };
                self.env
                    .declare(name.clone(), Value::UserFunction(Rc::new(f)))?;
                Ok(NORMAL_CONTROL_FLOW)
            }

            Statement::ReturnStatement(value) => match value {
                None => Ok(StatementResult {
                    flow: ControlFlow::Return(Value::Nil),
                    value: Value::Nil,
                }),
                Some(value) => Ok(StatementResult {
                    flow: ControlFlow::Return(self.evaluate(value)?),
                    value: Value::Nil,
                }),
            },
            Statement::IfStatement {
                if_branch,
                elif_branches,
                else_branch,
            } => {
                // if block
                if let Some(cond) = &if_branch.condition {
                    if self.evaluate(cond)?.is_truthy()? {
                        return self.execute_block_scope(&if_branch.body);
                    }
                }
                // else if blocks
                for branch in elif_branches {
                    if let Some(cond) = &branch.condition {
                        if self.evaluate(cond)?.is_truthy()? {
                            return self.execute_block_scope(&branch.body);
                        }
                    }
                }
                // else block
                if let Some(branch) = else_branch {
                    return self.execute_block_scope(&branch.body);
                }

                Ok(NORMAL_CONTROL_FLOW)
            }
        }
    }

    pub fn run(&mut self, tree: parser::Tree) -> Result<Value, RuntimeError> {
        if self.debug {
            println!("eval => {tree}");
        }

        let mut last = NORMAL_CONTROL_FLOW;
        for statement in tree.statements {
            let result = self.execute(&statement)?;
            match result.flow {
                ControlFlow::Break => {
                    return Err(RuntimeError(
                        "break cannot be used outside of a loop.".to_string(),
                    ));
                }
                ControlFlow::Continue => {
                    return Err(RuntimeError(
                        "continue cannot be used outside of a loop.".to_string(),
                    ));
                }
                ControlFlow::Return(_) => {
                    return Err(RuntimeError(
                        "return cannot be used outside of a function.".to_string(),
                    ));
                }
                ControlFlow::Next => last = result,
            }
        }
        Ok(last.value)
    }

    fn evaluate(&mut self, expression: &Expression) -> Result<Value, RuntimeError> {
        match expression {
            VariableExpression(variable) => Ok(self.env.get(variable)?),
            LiteralExpression(lit) => Ok(Value::from(lit)),
            UnaryExpression(op, r) => op.eval(self.evaluate(r)?),
            BinaryExpression(l, op, r) => op.eval(self.evaluate(l)?, self.evaluate(r)?),
            LogicalExpression(lhs, op, rhs) => op.eval(self.evaluate(lhs)?, || self.evaluate(rhs)),
            GroupingExpression(x) => self.evaluate(x),
            AssignmentExpression(variable, value) => {
                let value = self.evaluate(value)?;
                self.env.assign(variable.clone(), value.clone())?;
                Ok(value)
            }
            CallExpression(callee, _, arguments) => {
                let params: Vec<Value> = arguments
                    .iter()
                    .map(|a| self.evaluate(a))
                    .collect::<Result<_, _>>()?;

                match self.evaluate(callee)? {
                    Value::UserFunction(f) => self.execute_user_function(f, &params),
                    Value::NativeFunction(f) => self.execute_native_function(f, &params),
                    other => Err(RuntimeError(format!(
                        "{} is not callable",
                        other.type_name()
                    ))),
                }
            }
        }
    }
}
