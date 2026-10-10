use std::{collections::HashMap, fmt, rc::Rc};

use crate::{
    lexer::NumberLiteral,
    parser::{BinaryOperator, Expr, Stmt, UnaryOperator},
};

const MAX_CALL_DEPTH: usize = 250;

#[derive(Debug)]
struct FunctionDef {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
enum Value {
    Number(NumberLiteral),
    String(String),
    Boolean(bool),
    Nil,
    Function(Rc<FunctionDef>),
}

enum Flow {
    Normal,
    Return { value: Value, line: usize },
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Nil => "nil",
            Value::Function(_) => "function",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(NumberLiteral::Int(n)) => write!(f, "{n}"),
            Value::Number(NumberLiteral::Float(n)) => write!(f, "{n:?}"),
            Value::String(value) => write!(f, "{value}"),
            Value::Boolean(bool) => write!(f, "{bool}"),
            Value::Nil => write!(f, "nil"),
            Value::Function(function) => write!(f, "<fn {}>", function.name),
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
    call_stack: Vec<HashMap<String, Value>>,
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
            call_stack: vec![],
        }
    }

    pub fn execute(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        match self.execute_block(statements)? {
            Flow::Normal => Ok(()),
            Flow::Return { line, .. } => Err(self.error(
                line,
                "cannot use 'return' outside of a function".to_string(),
            )),
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<Flow, RuntimeError> {
        for statement in statements {
            if let flow @ Flow::Return { .. } = self.execute_statement(statement)? {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }

    fn execute_statement(&mut self, statement: &Stmt) -> Result<Flow, RuntimeError> {
        let (line, result) = match statement {
            Stmt::Let { name, value, line } => {
                let result = self.evaluate(value, *line).map(|value| {
                    self.bind(name.clone(), value);
                    Flow::Normal
                });
                (*line, result)
            }
            Stmt::Print { expr, line } => {
                let result = self.evaluate(expr, *line).map(|value| {
                    println!("{value}");
                    Flow::Normal
                });
                (*line, result)
            }
            Stmt::Function {
                name,
                params,
                body,
                line,
            } => {
                let function = Value::Function(Rc::new(FunctionDef {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                }));
                self.bind(name.clone(), function);
                (*line, Ok(Flow::Normal))
            }
            Stmt::Return { value, line } => {
                let flow = match value {
                    Some(expr) => self
                        .evaluate(expr, *line)
                        .map(|value| Flow::Return { value, line: *line }),
                    None => Ok(Flow::Return {
                        value: Value::Nil,
                        line: *line,
                    }),
                };
                (*line, flow)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                line,
            } => {
                let value = self.evaluate(condition, *line)?;

                match value {
                    Value::Boolean(true) => (*line, self.execute_block(then_branch)),
                    Value::Boolean(false) => {
                        if let Some(branch) = else_branch {
                            (*line, self.execute_block(branch))
                        } else {
                            (*line, Ok(Flow::Normal))
                        }
                    }
                    value => (
                        *line,
                        Err(self.error(
                            *line,
                            format!("if condition must be boolean, got {}", value.type_name()),
                        )),
                    ),
                }
            }
        };
        result.map_err(|mut error| {
            error.line = line;
            error
        })
    }

    fn evaluate(&mut self, expr: &Expr, line: usize) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(number) => Ok(Value::Number(*number)),
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Identifier(name) => self
                .lookup(name)
                .ok_or_else(|| self.error(line, format!("undefined variable '{name}'"))),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
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
            Expr::Call {
                callee,
                args,
                line: call_line,
            } => {
                let callee = self.evaluate(callee, *call_line)?;
                let function = match callee {
                    Value::Function(function) => function,
                    other => {
                        return Err(
                            self.error(*call_line, format!("cannot call {}", other.type_name()))
                        );
                    }
                };

                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.evaluate(arg, *call_line)?);
                }
                self.call_function(&function, values, *call_line)
            }
        }
    }

    fn call_function(
        &mut self,
        function: &Rc<FunctionDef>,
        args: Vec<Value>,
        line: usize,
    ) -> Result<Value, RuntimeError> {
        if args.len() != function.params.len() {
            return Err(self.error(
                line,
                format!(
                    "function '{}' expects {} argument{}, got {}",
                    function.name,
                    function.params.len(),
                    if function.params.len() == 1 { "" } else { "s" },
                    args.len()
                ),
            ));
        }

        if self.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(self.error(line, "maximum call depth exceeded".to_string()));
        }

        let mut scope = HashMap::new();
        for (param, value) in function.params.iter().zip(args) {
            scope.insert(param.clone(), value);
        }

        self.call_stack.push(scope);
        let flow = self.execute_block(&function.body);
        self.call_stack.pop();

        match flow? {
            Flow::Normal => Ok(Value::Nil),
            Flow::Return { value, .. } => Ok(value),
        }
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        self.call_stack
            .last()
            .and_then(|scope| scope.get(name))
            .cloned()
            .or_else(|| self.variables.get(name).cloned())
    }

    fn bind(&mut self, name: String, value: Value) {
        match self.call_stack.last_mut() {
            Some(scope) => {
                scope.insert(name, value);
            }
            None => {
                self.variables.insert(name, value);
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
        let value_type = value.type_name();

        match (operator, value) {
            (UnaryOperator::Plus, Value::Number(value)) => Ok(Value::Number(value)),

            (UnaryOperator::Minus, Value::Number(NumberLiteral::Int(value))) => value
                .checked_neg()
                .map(|value| Value::Number(NumberLiteral::Int(value)))
                .ok_or_else(|| self.error(line, "integer overflow".to_string())),

            (UnaryOperator::Minus, Value::Number(NumberLiteral::Float(value))) => {
                Ok(Value::Number(NumberLiteral::Float(-value)))
            }

            (UnaryOperator::Not, Value::Boolean(value)) => Ok(Value::Boolean(!value)),

            _ => Err(self.error(
                line,
                format!("cannot use '{}' with {value_type}", operator.symbol()),
            )),
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

        match (&left, &right, operator) {
            (Value::String(left), Value::String(right), BinaryOperator::Plus) => {
                return Ok(Value::String(format!("{left}{right}")));
            }

            (Value::String(left), Value::String(right), BinaryOperator::Eq) => {
                return Ok(Value::Boolean(left == right));
            }

            (Value::String(left), Value::String(right), BinaryOperator::NotEq) => {
                return Ok(Value::Boolean(left != right));
            }

            (Value::Boolean(left), Value::Boolean(right), BinaryOperator::Eq) => {
                return Ok(Value::Boolean(left == right));
            }

            (Value::Boolean(left), Value::Boolean(right), BinaryOperator::NotEq) => {
                return Ok(Value::Boolean(left != right));
            }

            (Value::Nil, Value::Nil, BinaryOperator::Eq) => {
                return Ok(Value::Boolean(true));
            }

            (Value::Nil, Value::Nil, BinaryOperator::NotEq) => {
                return Ok(Value::Boolean(false));
            }

            (Value::Number(left), Value::Number(right), operator) => {
                return self.evaluate_numbers(*left, operator, *right, line);
            }

            _ => {}
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
        let left_value = left.to_f64();
        let right_value = right.to_f64();

        let comparison = match operator {
            BinaryOperator::Eq => Some(left_value == right_value),
            BinaryOperator::NotEq => Some(left_value != right_value),
            BinaryOperator::Greater => Some(left_value > right_value),
            BinaryOperator::GreaterEq => Some(left_value >= right_value),
            BinaryOperator::Less => Some(left_value < right_value),
            BinaryOperator::LessEq => Some(left_value <= right_value),
            _ => None,
        };

        if let Some(result) = comparison {
            return Ok(Value::Boolean(result));
        }

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
            (left, operator, right)
                if matches!(
                    operator,
                    BinaryOperator::Plus
                        | BinaryOperator::Minus
                        | BinaryOperator::Star
                        | BinaryOperator::Slash
                ) =>
            {
                let (left, right) = (left.to_f64(), right.to_f64());
                let value = match operator {
                    BinaryOperator::Plus => left + right,
                    BinaryOperator::Minus => left - right,
                    BinaryOperator::Star => left * right,
                    BinaryOperator::Slash => left / right,
                    _ => unreachable!("guarded above"),
                };
                Ok(Value::Number(NumberLiteral::Float(value)))
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
        let mut interpreter = Interpreter::new();

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

    #[test]
    fn evaluates_numeric_comparisons() {
        let cases = [
            ("let x = 1 == 1", "true"),
            ("let x = 1 != 2", "true"),
            ("let x = 1 < 2", "true"),
            ("let x = 2 <= 2", "true"),
            ("let x = 3 > 2", "true"),
            ("let x = 3 >= 3", "true"),
            ("let x = 2 > 3", "false"),
        ];

        for (source, expected) in cases {
            assert_eq!(
                eval(source)
                    .expect("comparison should evaluate")
                    .to_string(),
                expected
            );
        }
    }

    #[test]
    fn compares_integer_and_float_numbers() {
        assert_eq!(
            eval("let x = 1 == 1.0")
                .expect("comparison should evaluate")
                .to_string(),
            "true"
        );
    }

    #[test]
    fn evaluates_boolean_comparisons() {
        assert_eq!(
            eval("let x = true == true")
                .expect("comparison should evaluate")
                .to_string(),
            "true"
        );

        assert_eq!(
            eval("let x = true != false")
                .expect("comparison should evaluate")
                .to_string(),
            "true"
        );
    }

    #[test]
    fn evaluates_boolean_negation() {
        assert_eq!(
            eval("let x = !false")
                .expect("negation should evaluate")
                .to_string(),
            "true"
        );

        assert_eq!(
            eval("let x = !!true")
                .expect("negation should evaluate")
                .to_string(),
            "true"
        );
    }

    #[test]
    fn compares_strings_for_equality() {
        assert_eq!(
            eval(r#"let x = "forge" == "forge""#)
                .expect("comparison should evaluate")
                .to_string(),
            "true"
        );
    }

    #[test]
    fn rejects_ordering_strings() {
        let error = eval(r#"let x = "a" < "b""#).expect_err("string ordering should fail");

        assert_eq!(error.message, "cannot use '<' with string and string");
    }

    #[test]
    fn rejects_mixed_type_comparisons() {
        let error = eval("let x = true == 1").expect_err("mixed comparison should fail");

        assert_eq!(error.message, "cannot use '==' with boolean and number");
    }

    #[test]
    fn promotes_mixed_int_and_float_arithmetic() {
        assert_eq!(
            eval("let x = 1 + 2.5")
                .expect("should evaluate")
                .to_string(),
            "3.5"
        );
        assert_eq!(
            eval("let x = 2.5 + 1")
                .expect("should evaluate")
                .to_string(),
            "3.5"
        );
    }

    #[test]
    fn float_arithmetic_keeps_decimal_display() {
        assert_eq!(
            eval("let x = 1.0 + 2.0")
                .expect("should evaluate")
                .to_string(),
            "3.0"
        );
        assert_eq!(
            eval("let x = 1.5 * 2")
                .expect("should evaluate")
                .to_string(),
            "3.0"
        );
    }

    #[test]
    fn float_division_is_not_truncated() {
        assert_eq!(
            eval("let x = 1.0 / 2")
                .expect("should evaluate")
                .to_string(),
            "0.5"
        );
        assert_eq!(
            eval("let x = 1 / 2.0")
                .expect("should evaluate")
                .to_string(),
            "0.5"
        );
        assert_eq!(
            eval("let x = 1 / 2").expect("should evaluate").to_string(),
            "0"
        );
    }

    #[test]
    fn float_division_by_zero_follows_ieee754() {
        assert_eq!(
            eval("let x = 1.0 / 0.0")
                .expect("should evaluate")
                .to_string(),
            "inf"
        );
        assert_eq!(
            eval("let x = -1.0 / 0.0")
                .expect("should evaluate")
                .to_string(),
            "-inf"
        );
        assert_eq!(
            eval("let x = 0.0 / 0.0")
                .expect("should evaluate")
                .to_string(),
            "NaN"
        );
    }

    #[test]
    fn integer_division_by_zero_still_errors() {
        let error = eval("let x = 1 / 0").expect_err("division by zero should fail");
        assert_eq!(error.message, "division by zero");
    }

    #[test]
    fn integer_overflow_still_errors() {
        let error = eval("let x = 2000000000 * 2000000000").expect_err("overflow should fail");
        assert_eq!(error.message, "integer overflow");
    }

    #[test]
    fn calls_a_function() {
        let program = statements("fn add(a, b) { return a + b } let x = add(2, 3)");
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("should execute");
        assert_eq!(interpreter.variables.get("x").unwrap().to_string(), "5");
    }

    #[test]
    fn function_without_return_yields_nil() {
        let program = statements("fn noop() { let y = 1 } let x = noop()");
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("should execute");
        assert_eq!(interpreter.variables.get("x").unwrap().to_string(), "nil");
    }

    #[test]
    fn recursion_works() {
        let program = statements(
            "fn fact(n) { if n <= 1 { return 1 } return n * fact(n - 1) } let x = fact(5)",
        );
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("should execute");
        assert_eq!(interpreter.variables.get("x").unwrap().to_string(), "120");
    }

    #[test]
    fn reports_arity_mismatch() {
        let program = statements("fn add(a, b) { return a + b } let x = add(1)");
        let mut interpreter = Interpreter::new();
        let error = interpreter
            .execute(&program)
            .expect_err("arity mismatch should fail");
        assert_eq!(error.message, "function 'add' expects 2 arguments, got 1");
    }

    #[test]
    fn call_scopes_do_not_leak() {
        let program = statements("let y = 1 fn f(y) { return y + 1 } let x = f(10)");
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("should execute");
        assert_eq!(interpreter.variables.get("y").unwrap().to_string(), "1");
        assert_eq!(interpreter.variables.get("x").unwrap().to_string(), "11");
    }

    #[test]
    fn functions_are_first_class() {
        let program = statements("fn add(a, b) { return a + b } let g = add let x = g(1, 2)");
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("should execute");
        assert_eq!(interpreter.variables.get("x").unwrap().to_string(), "3");
    }

    #[test]
    fn rejects_calling_non_function() {
        let program = statements("let x = 1 let y = x(2)");
        let mut interpreter = Interpreter::new();
        let error = interpreter
            .execute(&program)
            .expect_err("calling a number should fail");
        assert_eq!(error.message, "cannot call number");
    }

    #[test]
    fn compares_nil_to_nil() {
        let program = statements("fn f() {} let x = f() == f()");
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("should execute");
        assert_eq!(interpreter.variables.get("x").unwrap().to_string(), "true");
    }

    #[test]
    fn reports_undefined_variable_inside_body() {
        let program = statements("fn f() { return missing } let x = f()");
        let mut interpreter = Interpreter::new();
        let error = interpreter
            .execute(&program)
            .expect_err("missing variable should fail");
        assert_eq!(error.message, "undefined variable 'missing'");
    }

    #[test]
    fn enforces_call_depth_limit() {
        let program = statements("fn f() { return f() } let x = f()");
        let mut interpreter = Interpreter::new();
        let error = interpreter
            .execute(&program)
            .expect_err("infinite recursion should fail");
        assert_eq!(error.message, "maximum call depth exceeded");
    }
}
