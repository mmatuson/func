use std::{collections::HashMap, rc::Rc};

use crate::{
    errors::ResolveError,
    parser::{Expression, IfBranch, Statement},
};

type Scope = HashMap<String, bool>;

type ResolvedLocals = HashMap<usize, i32>; // id, depth
#[derive(Debug)]
struct Resolver {
    scopes: Vec<Scope>,
    locals: ResolvedLocals,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            scopes: vec![HashMap::new()],
        }
    }

    pub fn run(&mut self, stmts: &[Statement]) -> Result<ResolvedLocals, ResolveError> {
        for stmt in stmts {
            self.resolve_statement(stmt)?;
        }
        Ok(self.locals.clone())
    }

    // STATEMENT RESOLVERS

    fn resolve_statement(&mut self, stmt: &Statement) -> Result<(), ResolveError> {
        match stmt {
            Statement::BreakStatement => Ok(()),
            Statement::ContinueStatement => Ok(()),
            Statement::ExpressionStatement(expr) => self.resolve_expression_statement(expr),
            Statement::BlockStatement(block) => self.resolve_block(block),
            Statement::DeclarationStatement(var, value) => self.resolve_declaration(var, value),
            Statement::FunctionDeclaration(name, params, body) => {
                self.resolve_function_declaration(name, params, body)
            }
            Statement::IfStatement {
                if_branch,
                elif_branches,
                else_branch,
            } => self.resolve_branches(if_branch, elif_branches, else_branch),
            Statement::ReturnStatement(value) => self.resolve_return(value),
            Statement::WhileStatement(condition, body) => self.resolve_while(condition, body),
        }
    }

    fn resolve_block(&mut self, stmts: &[Statement]) -> Result<(), ResolveError> {
        self.enter_scope();
        for stmt in stmts {
            self.resolve_statement(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }

    fn resolve_declaration(
        &mut self,
        var: &String,
        value: &Expression,
    ) -> Result<(), ResolveError> {
        self.declare(var);
        self.resolve_expression(value)?;
        self.define(var);
        Ok(())
    }

    fn resolve_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<(), ResolveError> {
        self.resolve_expression(condition)?;
        self.resolve_block(body)?;
        Ok(())
    }

    fn resolve_expression_statement(
        &mut self,
        expression: &Expression,
    ) -> Result<(), ResolveError> {
        self.resolve_expression(expression)
    }

    fn resolve_return(&mut self, value: &Option<Expression>) -> Result<(), ResolveError> {
        match value {
            Some(expr) => self.resolve_expression(expr),
            None => Ok(()),
        }
    }

    fn resolve_branches(
        &mut self,
        if_branch: &IfBranch,
        elif_branches: &[IfBranch],
        else_branch: &Option<IfBranch>,
    ) -> Result<(), ResolveError> {
        if let Some(cond) = &if_branch.condition {
            self.resolve_expression(cond)?;
        }
        self.resolve_block(&if_branch.body)?;

        // else if blocks
        for branch in elif_branches {
            if let Some(cond) = &branch.condition {
                self.resolve_expression(cond)?;
            }
            self.resolve_block(&branch.body)?;
        }

        // else block
        if let Some(branch) = else_branch {
            self.resolve_block(&branch.body)?;
        }

        Ok(())
    }

    fn resolve_function_declaration(
        &mut self,
        name: &String,
        params: &[String],
        body: &Rc<Vec<Statement>>,
    ) -> Result<(), ResolveError> {
        self.declare(name)?;
        self.define(name)?;

        self.enter_scope();

        for p in params {
            self.declare(p)?;
            self.define(p)?;
        }

        // we're not using resolve_block here since we want params + body in the same scope
        // wrap in closure so we can ensure exit_scope is called
        let result: Result<(), ResolveError> = (|| {
            for stmt in body.as_ref() {
                self.resolve_statement(stmt)?;
            }
            Ok(())
        })();

        self.exit_scope();

        result
    }

    // EXPRESSION RESOLVERS
    fn resolve_local(&mut self, name: String, expr: &Expression) -> Result<(), ResolveError> {
        // TBD
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<(), ResolveError> {
        match expression {
            Expression::AssignmentExpression(name, expr) => {
                self.resolve_expression(expr)?;
                self.resolve_local(name.clone(), expr)?;
                Ok(())
            }
            Expression::BinaryExpression(lhs, _, rhs) => {
                self.resolve_expression(lhs)?;
                self.resolve_expression(rhs)?;
                Ok(())
            }
            Expression::CallExpression(callee, _, args) => {
                self.resolve_expression(callee)?;
                for arg in args {
                    self.resolve_expression(arg)?;
                }
                Ok(())
            }
            Expression::GroupingExpression(expr) => {
                self.resolve_expression(expr)?;
                Ok(())
            }
            Expression::LiteralExpression(_) => Ok(()),
            Expression::LogicalExpression(lhs, _, rhs) => {
                self.resolve_expression(lhs)?;
                self.resolve_expression(rhs)?;
                Ok(())
            }
            Expression::UnaryExpression(_, rhs) => {
                self.resolve_expression(rhs)?;
                Ok(())
            }
            Expression::VariableExpression(name) => {
                // book has resolve_local here but I dont have an expr
                if let Some(scope) = self.scopes.last_mut() {
                    return scope.contains_key(name).ok_or(ResolveError(
                        "Cannot read local variable in it's own initializer.".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }
    // SCOPE HELPERS

    // we dont support declare without initialize, fyi
    fn declare(&mut self, var: &String) -> Result<(), ResolveError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(var) {
                return Err(ResolveError(format!(
                    "variable {} already declared in this scope",
                    var
                )));
            }
            scope.insert(var.clone(), false);
        } // else we assume the variable is global if there are no scopes

        Ok(())
    }

    fn define(&mut self, var: &String) -> Result<(), ResolveError> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(var.clone(), true);
        } // else we assume the variable is global if there are no scopes

        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn exit_scope(&mut self) -> bool {
        match self.scopes.pop() {
            Some(_) => true,
            None => false,
        }
    }
}
