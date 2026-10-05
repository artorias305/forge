use std::{collections::HashMap, fmt};

use crate::{
    lexer::NumberLiteral,
    parser::{BinaryOperator, Expr, Stmt, UnaryOperator},
};

#[derive(Debug, Clone)]
enum Value {
    Number(NumberLiteral),
    String(String),
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(NumberLiteral::Int(n)) => write!(f, "{n}"),
            Value::Number(NumberLiteral::Float(n)) => write!(f, "{n}"),
            Value::String(value) => write!(f, "{value}"),
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
            Expr::String(value) => Ok(Value::String(value.clone())),
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
            (UnaryOperator::Minus, Value::String(_)) => Err(self.error(
                line,
                format!("cannot use '{}' with string", operator.symbol()),
            )),
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
        let left_type = left.type_name();
        let right_type = right.type_name();

        if let (Value::String(left), Value::String(right), BinaryOperator::Plus) =
            (&left, &right, operator)
        {
            return Ok(Value::String(format!("{left}{right}")));
        }

        if let (Value::Number(left), Value::Number(right)) = (&left, &right) {
            return self.evaluate_numbers(*left, operator, *right, line);
        }

        Err(self.error(
            line,
            format!(
                "cannot use '{}' with {left_type} and {right_type}",
                operator.symbol()
            ),
        ))
    }

    fn evaluate_numbers(
        &self,
        left: NumberLiteral,
        operator: &BinaryOperator,
        right: NumberLiteral,
        line: usize,
    ) -> Result<Value, RuntimeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Parser as ForgeParser};

    fn statements(source: &str) -> Vec<Stmt> {
        let tokens = Lexer::new(source).tokenize().expect("source should lex");
        ForgeParser::new(tokens)
            .parse()
            .expect("source should parse")
    }

    fn eval(source: &str) -> Result<Value, RuntimeError> {
        let parsed = statements(source);
        let interpreter = Interpreter::new();

        match parsed.into_iter().next().expect("expected one statement") {
            Stmt::Let { value, line, .. } => interpreter.evaluate(&value, line),
            other => panic!("expected a let statement, got {other:?}"),
        }
    }

    #[test]
    fn evaluates_string_literal() {
        assert_eq!(
            eval("let s = \"hi\"").expect("should evaluate").to_string(),
            "hi"
        );
    }

    #[test]
    fn evaluates_empty_string() {
        assert_eq!(
            eval("let s = \"\"").expect("should evaluate").to_string(),
            ""
        );
    }

    #[test]
    fn concatenates_two_strings() {
        let value = eval("let s = \"a\" + \"b\"").expect("should evaluate");
        assert_eq!(value.to_string(), "ab");
    }

    #[test]
    fn concatenates_escaped_string() {
        let value = eval(r#"let s = "a\"b" + "c""#).expect("should evaluate");
        assert_eq!(value.to_string(), "a\"bc");
    }

    #[test]
    fn concatenates_more_than_two_strings() {
        let value = eval(r#"let s = "a" + "b" + "c""#).expect("should evaluate");
        assert_eq!(value.to_string(), "abc");
    }

    #[test]
    fn resolves_string_from_variable() {
        let mut interpreter = Interpreter::new();

        for statement in &statements("let s = \"hi\"") {
            interpreter
                .execute_statement(statement)
                .expect("should execute");
        }

        assert_eq!(
            interpreter
                .variables
                .get("s")
                .expect("variable should exist")
                .to_string(),
            "hi"
        );
    }

    #[test]
    fn rejects_string_and_number_operands() {
        let error = eval("let s = \"a\" + 1").expect_err("mixed operands should fail");
        assert_eq!(error.message, "cannot use '+' with string and number");
    }

    #[test]
    fn rejects_string_subtraction() {
        let error = eval("let s = \"a\" - \"b\"").expect_err("only + concatenates");
        assert_eq!(error.message, "cannot use '-' with string and string");
    }

    #[test]
    fn rejects_unary_minus_on_string() {
        let error = eval("let s = -\"a\"").expect_err("cannot negate a string");
        assert_eq!(error.message, "cannot use '-' with string");
    }

    #[test]
    fn keeps_number_arithmetic_working() {
        let value = eval("let s = 10 + 5").expect("should evaluate");
        assert_eq!(value.to_string(), "15");
    }

    #[test]
    fn keeps_unary_minus_on_number_working() {
        let value = eval("let s = -10").expect("should evaluate");
        assert_eq!(value.to_string(), "-10");
    }
}
