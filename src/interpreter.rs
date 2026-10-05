use std::{collections::HashMap, fmt};

use crate::{
    lexer::NumberLiteral,
    parser::{BinaryOperator, Expr, Stmt, UnaryOperator},
};

#[derive(Debug, Clone)]
enum Value {
    Number(NumberLiteral),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(NumberLiteral::Int(n)) => write!(f, "{n}"),
            Value::Number(NumberLiteral::Float(n)) => write!(f, "{n}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for RuntimeError {}

pub struct Interpreter {
    variables: HashMap<String, Value>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn execute(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Stmt) -> Result<(), RuntimeError> {
        let (line, result) = match statement {
            Stmt::Let { name, value, line } => {
                let result = self.evaluate(value, *line).map(|value| {
                    self.variables.insert(name.clone(), value);
                });
                (*line, result)
            }
            Stmt::Print { expr, line } => {
                let result = self.evaluate(expr, *line).map(|value| println!("{value}"));
                (*line, result)
            }
        };
        result.map_err(|mut error| {
            error.line = line;
            error
        })
    }

    fn evaluate(&self, expr: &Expr, line: usize) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(number) => Ok(Value::Number(*number)),
            Expr::Identifier(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| self.error(line, format!("undefined variable '{name}'"))),
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let left = self.evaluate(left, line)?;
                let right = self.evaluate(right, line)?;
                self.evaluate_binary(left, operator, right, line)
            }
            Expr::Unary { operator, operand } => {
                let value = self.evaluate(operand, line)?;
                self.evaluate_unary(operator, value, line)
            }
        }
    }

    fn error(&self, line: usize, message: String) -> RuntimeError {
        RuntimeError { line, message }
    }

    fn evaluate_unary(
        &self,
        operator: &UnaryOperator,
        value: Value,
        line: usize,
    ) -> Result<Value, RuntimeError> {
        match (operator, value) {
            (UnaryOperator::Plus, value) => Ok(value),
            (UnaryOperator::Minus, Value::Number(NumberLiteral::Int(value))) => value
                .checked_neg()
                .map(|value| Value::Number(NumberLiteral::Int(value)))
                .ok_or_else(|| self.error(line, "integer overflow".to_string())),
            (UnaryOperator::Minus, Value::Number(NumberLiteral::Float(value))) => {
                Ok(Value::Number(NumberLiteral::Float(-value)))
            }
        }
    }

    fn evaluate_binary(
        &self,
        left: Value,
        operator: &BinaryOperator,
        right: Value,
        line: usize,
    ) -> Result<Value, RuntimeError> {
        let (Value::Number(left), Value::Number(right)) = (left, right);
        match (left, operator, right) {
            (NumberLiteral::Int(left), BinaryOperator::Plus, NumberLiteral::Int(right)) => left
                .checked_add(right)
                .map(|value| Value::Number(NumberLiteral::Int(value)))
                .ok_or_else(|| self.error(line, "integer overflow".to_string())),
            (NumberLiteral::Int(left), BinaryOperator::Minus, NumberLiteral::Int(right)) => left
                .checked_sub(right)
                .map(|value| Value::Number(NumberLiteral::Int(value)))
                .ok_or_else(|| self.error(line, "integer overflow".to_string())),
            (NumberLiteral::Int(left), BinaryOperator::Star, NumberLiteral::Int(right)) => left
                .checked_mul(right)
                .map(|value| Value::Number(NumberLiteral::Int(value)))
                .ok_or_else(|| self.error(line, "integer overflow".to_string())),
            (NumberLiteral::Int(left), BinaryOperator::Slash, NumberLiteral::Int(right)) => {
                if right == 0 {
                    Err(self.error(line, "division by zero".to_string()))
                } else {
                    left.checked_div(right)
                        .map(|value| Value::Number(NumberLiteral::Int(value)))
                        .ok_or_else(|| self.error(line, "integer overflow".to_string()))
                }
            }
            _ => Err(self.error(line, "unsupported binary operation".to_string())),
        }
    }
}
