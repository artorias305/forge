use std::collections::HashMap;

use crate::{
    lexer::NumberLiteral,
    parser::{BinaryOperator, Expr, Stmt, UnaryOperator},
};

#[derive(Debug, Clone)]
enum Value {
    Nil,
    Number(NumberLiteral),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(NumberLiteral::Int(n)) => write!(f, "{n}"),
            Value::Number(NumberLiteral::Float(n)) => write!(f, "{n}"),
        }
    }
}

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

    pub fn execute(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.execute_statement(stmt);
        }
    }

    fn execute_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value } => {
                let value = self.evaluate(value);
                self.variables.insert(name.clone(), value);
            }

            Stmt::Print(expr) => {
                let value = self.evaluate(expr);
                println!("{value}");
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(number) => Value::Number(*number),
            Expr::Identifier(name) => self
                .variables
                .get(name)
                .cloned()
                .expect("undefined variable"),
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let left = self.evaluate(left);
                let right = self.evaluate(right);

                self.evaluate_binary(left, operator, right)
            }
            Expr::Unary { operator, operand } => {
                let value = self.evaluate(operand);

                self.evaluate_unary(operator, value)
            }
        }
    }

    fn evaluate_unary(&self, operator: &UnaryOperator, value: Value) -> Value {
        match (operator, value) {
            (UnaryOperator::Plus, value) => value,
            (UnaryOperator::Minus, Value::Number(NumberLiteral::Int(value))) => {
                Value::Number(NumberLiteral::Int(-value))
            }
            (UnaryOperator::Minus, Value::Number(NumberLiteral::Float(value))) => {
                Value::Number(NumberLiteral::Float(-value))
            }
            _ => panic!("unsupported unary operation"),
        }
    }

    fn evaluate_binary(&self, left: Value, operator: &BinaryOperator, right: Value) -> Value {
        match (left, operator, right) {
            (
                Value::Number(NumberLiteral::Int(left)),
                BinaryOperator::Plus,
                Value::Number(NumberLiteral::Int(right)),
            ) => Value::Number(NumberLiteral::Int(left + right)),

            (
                Value::Number(NumberLiteral::Int(left)),
                BinaryOperator::Minus,
                Value::Number(NumberLiteral::Int(right)),
            ) => Value::Number(NumberLiteral::Int(left - right)),

            (
                Value::Number(NumberLiteral::Int(left)),
                BinaryOperator::Star,
                Value::Number(NumberLiteral::Int(right)),
            ) => Value::Number(NumberLiteral::Int(left * right)),

            (
                Value::Number(NumberLiteral::Int(left)),
                BinaryOperator::Slash,
                Value::Number(NumberLiteral::Int(right)),
            ) => Value::Number(NumberLiteral::Int(left / right)),

            _ => panic!("unsupported binary operation"),
        }
    }
}
