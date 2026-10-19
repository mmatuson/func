use core::fmt;
use std::{fmt::Formatter, rc::Rc};

use crate::scanner::Token;

const MAX_FUNCTION_ARGS: usize = 100;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOperator {
    Bang,
    LogicalNot,
    Minus,
}
impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "negate"),
            UnaryOperator::Bang => write!(f, "not"),
            UnaryOperator::LogicalNot => write!(f, "not"),
        }
    }
}

impl TryFrom<&Token> for UnaryOperator {
    type Error = ParseError;

    fn try_from(t: &Token) -> Result<Self, Self::Error> {
        match t {
            Token::Bang => Ok(UnaryOperator::Bang),
            Token::Minus => Ok(UnaryOperator::Minus),
            Token::LogicalNot => Ok(UnaryOperator::LogicalNot),
            _ => Err(ParseError(format!("Not a unary operator: {:?}", t))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LogicalOperator {
    LogicalAnd,
    LogicalOr,
}
impl LogicalOperator {
    fn precedence(&self) -> Precedence {
        match self {
            LogicalOperator::LogicalOr => Precedence::LogicalOr,
            LogicalOperator::LogicalAnd => Precedence::LogicalAnd,
        }
    }
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOperator::LogicalAnd => write!(f, "and"),
            LogicalOperator::LogicalOr => write!(f, "or"),
        }
    }
}

impl TryFrom<&Token> for LogicalOperator {
    type Error = ParseError;

    fn try_from(t: &Token) -> Result<Self, Self::Error> {
        match t {
            Token::LogicalAnd => Ok(LogicalOperator::LogicalAnd),
            Token::LogicalOr => Ok(LogicalOperator::LogicalOr),
            _ => Err(ParseError(format!("Not a logical operator: {:?}", t))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOperator {
    Plus,
    Star,
    Minus,
    Div,
    Mod,
    GreaterThan,
    LessThan,
    NotEqual,
    EqualTo,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
}

impl BinaryOperator {
    fn precedence(&self) -> Precedence {
        match self {
            BinaryOperator::Plus | BinaryOperator::Minus => Precedence::Term,
            BinaryOperator::Div | BinaryOperator::Mod | BinaryOperator::Star => Precedence::Factor,
            BinaryOperator::EqualTo | BinaryOperator::NotEqual => Precedence::Equality,
            BinaryOperator::GreaterThan
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThanOrEqualTo
            | BinaryOperator::LessThanOrEqualTo => Precedence::Comparison,
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Star => write!(f, "*"),
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::EqualTo => write!(f, "=="),
            BinaryOperator::GreaterThanOrEqualTo => write!(f, ">="),
            BinaryOperator::LessThanOrEqualTo => write!(f, "<="),
        }
    }
}

impl TryFrom<&Token> for BinaryOperator {
    type Error = ParseError;

    fn try_from(t: &Token) -> Result<Self, Self::Error> {
        match t {
            Token::Plus => Ok(BinaryOperator::Plus),
            Token::Minus => Ok(BinaryOperator::Minus),
            Token::Star => Ok(BinaryOperator::Star),
            Token::Div => Ok(BinaryOperator::Div),
            Token::Mod => Ok(BinaryOperator::Mod),
            Token::EqualTo => Ok(BinaryOperator::EqualTo),
            Token::NotEqual => Ok(BinaryOperator::NotEqual),
            Token::GreaterThan => Ok(BinaryOperator::GreaterThan),
            Token::GreaterThanOrEqualTo => Ok(BinaryOperator::GreaterThanOrEqualTo),
            Token::LessThanOrEqualTo => Ok(BinaryOperator::LessThanOrEqualTo),
            Token::LessThan => Ok(BinaryOperator::LessThan),
            _ => Err(ParseError(format!("Not a binary operator: {:?}", t))),
        }
    }
}

/* FunctionCallParen will be used to identify location for runtime errors  */
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FunctionCallParen {
    LeftParen,
    RightParen,
}
// I don't know if we'll really need this
impl TryFrom<&Token> for FunctionCallParen {
    type Error = ParseError;

    fn try_from(t: &Token) -> Result<Self, Self::Error> {
        match t {
            Token::LeftParen => Ok(FunctionCallParen::LeftParen),
            Token::RightParen => Ok(FunctionCallParen::RightParen),
            _ => Err(ParseError(format!(
                "Not a valid function call paren {:?}",
                t
            ))),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Associativity {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Precedence {
    Assignment,
    Call,
    LogicalOr,
    LogicalAnd,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Primary,
}

impl Precedence {
    fn next(self) -> Option<Precedence> {
        match self {
            Precedence::Assignment => Some(Precedence::LogicalOr),
            Precedence::LogicalOr => Some(Precedence::LogicalAnd),
            Precedence::LogicalAnd => Some(Precedence::Equality),
            Precedence::Equality => Some(Precedence::Comparison),
            Precedence::Comparison => Some(Precedence::Term),
            Precedence::Term => Some(Precedence::Factor),
            Precedence::Factor => Some(Precedence::Unary),
            Precedence::Unary => Some(Precedence::Call),
            Precedence::Call => Some(Precedence::Primary),
            Precedence::Primary => None,
        }
    }
    fn associativity(self) -> Associativity {
        match self {
            Precedence::Assignment => Associativity::Right, // ^ will be right as well
            _ => Associativity::Left,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl Literal {
    pub fn type_name(&self) -> &'static str {
        match self {
            Literal::Integer(_) => "Integer",
            Literal::Float(_) => "Float",
            Literal::String(_) => "String",
            Literal::Bool(_) => "Bool",
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(v) => write!(f, "{v}"),
            Literal::Float(v) => write!(f, "{v}"),
            Literal::String(v) => write!(f, "{v}"),
            Literal::Bool(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    LiteralExpression(Literal),
    UnaryExpression(UnaryOperator, Box<Expression>),
    BinaryExpression(Box<Expression>, BinaryOperator, Box<Expression>), //left, op, right
    LogicalExpression(Box<Expression>, LogicalOperator, Box<Expression>), //left, op, right
    VariableExpression(String),
    GroupingExpression(Box<Expression>),
    AssignmentExpression(String, Box<Expression>),
    CallExpression(Box<Expression>, FunctionCallParen, Vec<Expression>), // callee, paren token, args
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::LiteralExpression(Literal::Integer(v)) => write!(f, "{v}"),
            Expression::LiteralExpression(Literal::Float(v)) => write!(f, "{v}"),
            Expression::LiteralExpression(Literal::String(v)) => write!(f, "\"{v}\""),
            Expression::LiteralExpression(Literal::Bool(v)) => write!(f, "{v}"),
            Expression::VariableExpression(value) => write!(f, "{value}"),
            Expression::BinaryExpression(l, o, r) => write!(f, "({} {} {})", o, l, r),
            Expression::LogicalExpression(l, o, r) => write!(f, "({} {} {})", o, l, r),
            Expression::UnaryExpression(o, x) => write!(f, "({} {})", o, x),
            Expression::GroupingExpression(x) => write!(f, "(group {})", x),
            Expression::AssignmentExpression(var, x) => write!(f, "(= {} {})", var, x),
            Expression::CallExpression(callee, _paren, arguments) => {
                write!(f, "(call {} {:?})", callee, arguments)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct IfBranch {
    pub condition: Option<Expression>,
    pub body: Vec<Statement>,
}
impl fmt::Display for IfBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(cond) = &self.condition {
            write!(f, "{}", cond)?;
        }
        for stmt in &self.body {
            write!(f, " {}", stmt)?;
        }
        write!(f, "")
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    DeclarationStatement(String, Expression),
    ExpressionStatement(Expression),
    BlockStatement(Vec<Statement>),
    IfStatement {
        if_branch: IfBranch,
        elif_branches: Vec<IfBranch>,
        else_branch: Option<IfBranch>,
    },
    WhileStatement(Expression, Vec<Statement>),
    FunctionDeclaration(String, Vec<String>, Rc<Vec<Statement>>),
    ReturnStatement(Option<Expression>),
    BreakStatement,
    ContinueStatement,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::BreakStatement => write!(f, "(break)"),
            Statement::ContinueStatement => write!(f, "(continue)"),
            Statement::DeclarationStatement(v, x) => write!(f, "(var= {} {})", v, x),
            Statement::ExpressionStatement(x) => write!(f, "({})", x),
            Statement::BlockStatement(lst) => {
                write!(f, "(block")?;
                for stmt in lst {
                    write!(f, " {}", stmt)?;
                }
                write!(f, ")")
            }
            Statement::IfStatement {
                if_branch,
                elif_branches,
                else_branch,
            } => {
                write!(f, "(if {})", if_branch)?;
                for b in elif_branches {
                    write!(f, "(elif {})", b)?;
                }
                if let Some(b) = else_branch {
                    write!(f, "(else {})", b)?;
                }
                write!(f, "")
            }
            Statement::WhileStatement(cond, statements) => {
                write!(f, "(while {}", cond)?;
                for stmt in statements {
                    write!(f, " {}", stmt)?;
                }
                write!(f, ")")
            }
            Statement::FunctionDeclaration(name, params, body) => {
                write!(f, "(fn {} (", name)?;
                for p in params {
                    write!(f, "{},", p)?;
                }
                write!(f, ") (")?;
                for s in body.iter() {
                    write!(f, "{}", s)?;
                }
                write!(f, ")")
            }
            Statement::ReturnStatement(value) => {
                if let Some(x) = &value {
                    write!(f, "(return {})", x)
                } else {
                    write!(f, "(return)")
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Tree {
    pub statements: Vec<Statement>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            statements: Vec::new(),
        }
    }
}
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(program ")?;
        for statement in &self.statements {
            write!(f, "{}", statement)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError(String);

pub struct Parser<'t> {
    tokens: &'t [Token],
    pos: usize,
    //size: usize,
}
impl<'t> Parser<'t> {
    pub fn new(tokens: &'t [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }
    pub fn parse(&mut self) -> Result<Tree, ParseError> {
        let mut tree: Tree = Tree::new();
        while !matches!(self.peek(), Some(Token::EOF)) {
            tree.statements.push(self.parse_statement()?);
        }
        Ok(tree)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Some(Token::Var) => self.parse_assignment_statement(),
            Some(Token::LeftBrace) => self.parse_block_statement(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::Break) => self.parse_break_statement(),
            Some(Token::Continue) => self.parse_continue_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::Function) => self.parse_function_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_break_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect_token(Token::Break)?;
        self.expect_token(Token::Semicolon)?;
        return Ok(Statement::BreakStatement);
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect_token(Token::Continue)?;
        self.expect_token(Token::Semicolon)?;
        return Ok(Statement::ContinueStatement);
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect_token(Token::Return)?;
        let mut value: Option<Expression> = None;
        if !matches!(self.peek(), Some(Token::Semicolon)) {
            value = Some(self.parse_expression()?);
        }
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::ReturnStatement(value))
    }

    fn parse_assignment_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect_token(Token::Var)?;
        let name = self.expect_ident()?;
        self.expect_token(Token::Equal)?;
        let value = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::DeclarationStatement(name, value))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::ExpressionStatement(value))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, ParseError> {
        let block_stmts = self.parse_block()?;
        return Ok(Statement::BlockStatement(block_stmts));
    }

    fn parse_if_block(&mut self) -> Result<IfBranch, ParseError> {
        // self.expect_token(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        // self.expect_token(Token::RightParen)?;
        let body = self.parse_block()?;
        Ok(IfBranch {
            condition: Some(condition),
            body,
        })
    }

    fn parse_else_block(&mut self) -> Result<IfBranch, ParseError> {
        let body = self.parse_block()?;
        Ok(IfBranch {
            condition: None,
            body,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect_token(Token::If)?;
        let if_branch = self.parse_if_block()?;
        let mut else_branch = None;
        let mut elif_branches: Vec<IfBranch> = Vec::new();
        while matches!(self.peek(), Some(Token::Else)) {
            self.expect_token(Token::Else)?;
            if matches!(self.peek(), Some(Token::If)) {
                self.expect_token(Token::If)?;
                let elif_branch = self.parse_if_block()?;
                elif_branches.push(elif_branch);
            } else {
                else_branch = Some(self.parse_else_block()?);
                break;
            }
        }
        Ok(Statement::IfStatement {
            if_branch,
            elif_branches,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect_token(Token::While)?;
        // self.expect_token(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        //  self.expect_token(Token::RightParen)?;
        let body = self.parse_block()?;

        Ok(Statement::WhileStatement(condition, body))
    }

    fn parse_function_call(&mut self) -> Result<Expression, ParseError> {
        let mut callee = self.parse_precedence_expression(Precedence::Call.next().ok_or(
            ParseError("Missing precedence level after Call".to_string()),
        )?)?;

        while matches!(self.peek(), Some(Token::LeftParen)) {
            let mut args: Vec<Expression> = Vec::with_capacity(MAX_FUNCTION_ARGS);
            self.expect_token(Token::LeftParen)?;
            if !matches!(self.peek(), Some(Token::RightParen)) {
                loop {
                    if args.len() >= MAX_FUNCTION_ARGS {
                        return Err(ParseError(format!(
                            "Maximum amount of function arguments is {}.",
                            MAX_FUNCTION_ARGS
                        )));
                    }
                    let arg_expression = self.parse_expression()?;
                    args.push(arg_expression);
                    if matches!(self.peek(), Some(Token::Comma)) {
                        self.expect_token(Token::Comma)?;
                    } else {
                        break;
                    }
                }
            }
            let closing_paren = self.expect_token(Token::RightParen)?;
            callee = Expression::CallExpression(
                Box::new(callee),
                FunctionCallParen::try_from(closing_paren)?,
                args,
            );
        }
        Ok(callee)
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, ParseError> {
        let mut params: Vec<String> = Vec::with_capacity(MAX_FUNCTION_ARGS);
        self.expect_token(Token::Function)?;
        let name = self.expect_ident()?;
        self.expect_token(Token::LeftParen)?;
        if !matches!(self.peek(), Some(Token::RightParen)) {
            loop {
                if params.len() >= MAX_FUNCTION_ARGS {
                    return Err(ParseError(format!(
                        "Maximum amount of function arguments is {}.",
                        MAX_FUNCTION_ARGS
                    )));
                }
                let param = self.expect_ident()?;
                params.push(param.clone());
                if matches!(self.peek(), Some(Token::Comma)) {
                    self.expect_token(Token::Comma)?;
                } else {
                    break;
                }
            }
        }
        self.expect_token(Token::RightParen)?;
        let body = self.parse_block()?;
        Ok(Statement::FunctionDeclaration(
            name.clone(),
            params,
            Rc::new(body),
        ))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut block_stmts: Vec<Statement> = Vec::new();
        self.expect_token(Token::LeftBrace)?;
        while !matches!(self.peek(), Some(Token::RightBrace)) {
            if matches!(self.peek(), Some(Token::EOF) | None) {
                println!("we hit the end and found no brace");
                return Err(ParseError(
                    "Expected closing brace } at end of block".to_string(),
                ));
            }
            block_stmts.push(self.parse_statement()?);
        }
        self.expect_token(Token::RightBrace)?;
        Ok(block_stmts)
    }

    fn expect_token(&mut self, expected: Token) -> Result<&Token, ParseError> {
        let actual = self.next()?;
        if *actual != expected {
            return Err(ParseError(format!(
                "Expected {:?} but got {:?}",
                expected, actual
            )));
        }
        Ok(actual)
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_precedence_expression(Precedence::Assignment)
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let actual = self.next()?;
        match actual {
            Token::Ident(name) => Ok(name.clone()),
            other => {
                return Err(ParseError(format!(
                    "Expected Identifier but got {:?}",
                    other
                )));
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Result<&Token, ParseError> {
        let token = self
            .tokens
            .get(self.pos)
            .ok_or(ParseError("Out of tokens".to_string()))?;
        self.pos += 1;
        Ok(token)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let actual = self.next()?;
        match actual {
            Token::Ident(name) => Ok(Expression::VariableExpression(name.clone())),
            Token::Number(value) => Ok(Expression::LiteralExpression(Literal::Integer(*value))),
            Token::Float(value) => Ok(Expression::LiteralExpression(Literal::Float(*value))),
            Token::StringLiteral(value) => Ok(Expression::LiteralExpression(Literal::String(
                value.clone(),
            ))),
            Token::Bool(value) => Ok(Expression::LiteralExpression(Literal::Bool(*value))),
            Token::LeftParen => {
                let expression = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(Expression::GroupingExpression(Box::new(expression)))
            }
            other => {
                return Err(ParseError(format!("Unexpected Token got {:?}", other)));
            }
        }
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if let Some(token) = self.peek() {
            if let Ok(op) = UnaryOperator::try_from(token) {
                self.next()?; // consume op
                let right = self.parse_unary()?;
                return Ok(Expression::UnaryExpression(op, Box::new(right)));
            }
        }
        self.parse_precedence_expression(Precedence::Unary.next().ok_or(ParseError(
            "Missing precedence level after Unary".to_string(),
        ))?)
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParseError> {
        let lhs = self.parse_precedence_expression(Precedence::Assignment.next().ok_or(
            ParseError("Missing precedence level after Assignment".to_string()),
        )?)?;

        if matches!(self.peek(), Some(Token::Equal)) {
            self.next()?; // consume

            return match lhs {
                Expression::VariableExpression(var) => {
                    // this allows chained assignment -> a = b = c;
                    // jump to equality to disallow
                    let rhs = self.parse_precedence_expression(Precedence::Assignment)?;
                    Ok(Expression::AssignmentExpression(var, Box::new(rhs)))
                }
                _ => Err(ParseError("Invalid assignment target".to_string())),
            };
        }
        Ok(lhs)
    }

    fn parse_logical(&mut self, level: Precedence) -> Result<Expression, ParseError> {
        let next_level = level
            .next()
            .ok_or(ParseError("Failure to parse lh expression".to_string()))?;

        let mut left = self.parse_precedence_expression(next_level)?;

        loop {
            let Some(nt) = self.peek() else { break };
            let Ok(op) = LogicalOperator::try_from(nt) else {
                break;
            };

            // if the operator is not for our parsing level, skip
            if op.precedence() != level {
                break;
            }

            self.next()?; // consume token we peeked

            let right: Expression = match level.associativity() {
                Associativity::Left => self.parse_precedence_expression(next_level)?,
                Associativity::Right => self.parse_precedence_expression(level)?,
            };

            left = Expression::LogicalExpression(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_binary(&mut self, level: Precedence) -> Result<Expression, ParseError> {
        let next_level = level
            .next()
            .ok_or(ParseError("Failure to parse lh expression".to_string()))?;

        let mut left = self.parse_precedence_expression(next_level)?;

        loop {
            let Some(nt) = self.peek() else { break };
            let Ok(op) = BinaryOperator::try_from(nt) else {
                break;
            };

            // if the operator is not for our parsing level, skip
            if op.precedence() != level {
                break;
            }

            self.next()?; // consume token we peeked

            let right: Expression = match level.associativity() {
                Associativity::Left => self.parse_precedence_expression(next_level)?,
                Associativity::Right => self.parse_precedence_expression(level)?,
            };

            left = Expression::BinaryExpression(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_precedence_expression(&mut self, level: Precedence) -> Result<Expression, ParseError> {
        match level {
            Precedence::Assignment => self.parse_assignment(),
            Precedence::LogicalAnd | Precedence::LogicalOr => self.parse_logical(level),
            Precedence::Unary => self.parse_unary(),
            Precedence::Call => self.parse_function_call(),
            Precedence::Primary => self.parse_primary(),
            _ => self.parse_binary(level),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{parser::Parser, scanner::Token};

//     #[test]
//     fn test_parse_var_decl_expression_statement() {
//         let tokens = vec![
//             Token::Var,
//             Token::Ident("result".to_string()),
//             Token::Equal,
//             Token::Ident("x".to_string()),
//             Token::Plus,
//             Token::Number(3),
//             Token::Star,
//             Token::Number(4),
//             Token::Semicolon,
//             Token::EOF,
//         ];
//         let tree = Parser::new(&tokens).parse().expect("parse should succeed");
//         assert_eq!(tree.to_string(), "(program (= result (+ x (* 3 4))))");
//     }
// }
