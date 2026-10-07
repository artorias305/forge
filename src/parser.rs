use std::fmt;

use crate::lexer::{Literal, NumberLiteral, Token, TokenKind};

#[derive(Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    NotEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,
}

impl BinaryOperator {
    fn from(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Plus => Some(Self::Plus),
            TokenKind::Minus => Some(Self::Minus),
            TokenKind::Star => Some(Self::Star),
            TokenKind::Slash => Some(Self::Slash),
            TokenKind::OpEqEq => Some(Self::Eq),
            TokenKind::OpNEq => Some(Self::NotEq),
            TokenKind::OpGt => Some(Self::Greater),
            TokenKind::OpGtEq => Some(Self::GreaterEq),
            TokenKind::OpLt => Some(Self::Less),
            TokenKind::OpLtEq => Some(Self::LessEq),
            _ => None,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Greater => ">",
            Self::GreaterEq => ">=",
            Self::Less => "<",
            Self::LessEq => "<=",
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Eq | Self::NotEq => 1,
            Self::Greater | Self::GreaterEq | Self::Less | Self::LessEq => 2,
            Self::Plus | Self::Minus => 3,
            Self::Star | Self::Slash => 4,
        }
    }
}

#[derive(Debug)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
}

impl UnaryOperator {
    fn from(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Plus => Some(Self::Plus),
            TokenKind::Minus => Some(Self::Minus),
            TokenKind::OpNot => Some(Self::Not),
            _ => None,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Number(NumberLiteral),
    String(String),
    Identifier(String),
    Boolean(bool),
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: BinaryOperator,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Print {
        expr: Expr,
        line: usize,
    },
    Let {
        name: String,
        value: Expr,
        line: usize,
    },
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    last_line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            last_line: 1,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn line(&self) -> usize {
        self.peek().map_or(self.last_line, |token| token.line)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        self.last_line = token.line;
        Some(token)
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            line: self.line(),
            message: message.into(),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_expr(0)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let operator = self
            .peek()
            .and_then(|token| UnaryOperator::from(&token.kind));

        match operator {
            Some(operator) => {
                self.advance();
                Ok(Expr::Unary {
                    operator,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            None => self.parse_primary(),
        }
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(token) = self.peek() {
            let Some(operator) = BinaryOperator::from(&token.kind) else {
                break;
            };
            if operator.precedence() < min_precedence {
                break;
            }
            self.advance();
            let right = self.parse_binary_expr(operator.precedence() + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self
            .advance()
            .ok_or_else(|| self.error("expected expression"))?;

        match token.kind {
            TokenKind::Number => match token.literal {
                Literal::Number(number) => Ok(Expr::Number(number)),
                _ => Err(ParseError {
                    line: token.line,
                    message: "invalid number literal".to_string(),
                }),
            },
            TokenKind::String => match token.literal {
                Literal::String(value) => Ok(Expr::String(value)),
                _ => Err(ParseError {
                    line: token.line,
                    message: "invalid string literal".to_string(),
                }),
            },
            TokenKind::Identifier => Ok(Expr::Identifier(token.lexeme)),
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;
                let closing = self.advance().ok_or_else(|| self.error("expected ')'"))?;
                if !matches!(closing.kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        line: closing.line,
                        message: "expected ')'".to_string(),
                    });
                }
                Ok(expr)
            }
            TokenKind::KwTrue => Ok(Expr::Boolean(true)),
            TokenKind::KwFalse => Ok(Expr::Boolean(false)),
            _ => Err(ParseError {
                line: token.line,
                message: "expected expression".to_string(),
            }),
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self
            .advance()
            .ok_or_else(|| self.error("expected statement"))?;

        match token.kind {
            TokenKind::KwPrint => {
                let opening = self.advance().ok_or_else(|| self.error("expected '('"))?;
                if !matches!(opening.kind, TokenKind::LeftParen) {
                    return Err(ParseError {
                        line: opening.line,
                        message: "expected '('".to_string(),
                    });
                }
                let expr = self.parse_expr()?;
                let closing = self.advance().ok_or_else(|| self.error("expected ')'"))?;
                if !matches!(closing.kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        line: closing.line,
                        message: "expected ')'".to_string(),
                    });
                }
                Ok(Stmt::Print {
                    expr,
                    line: token.line,
                })
            }
            TokenKind::KwLet => {
                let name_token = self
                    .advance()
                    .ok_or_else(|| self.error("expected identifier"))?;
                if !matches!(name_token.kind, TokenKind::Identifier) {
                    return Err(ParseError {
                        line: name_token.line,
                        message: "expected identifier".to_string(),
                    });
                }
                let equals = self.advance().ok_or_else(|| self.error("expected '='"))?;
                if !matches!(equals.kind, TokenKind::Equals) {
                    return Err(ParseError {
                        line: equals.line,
                        message: "expected '='".to_string(),
                    });
                }
                Ok(Stmt::Let {
                    name: name_token.lexeme,
                    value: self.parse_expr()?,
                    line: token.line,
                })
            }
            _ => Err(ParseError {
                line: token.line,
                message: "expected statement".to_string(),
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Vec<Stmt>, ParseError> {
        let tokens = Lexer::new(source).tokenize().expect("source should lex");
        Parser::new(tokens).parse()
    }

    #[test]
    fn parses_string_print_statement() {
        let statements = parse("print(\"hi\")").expect("source should parse");

        match &statements[0] {
            Stmt::Print {
                expr: Expr::String(value),
                ..
            } => assert_eq!(value, "hi"),
            other => panic!("expected print of a string, got {other:?}"),
        }
    }

    #[test]
    fn parses_string_let_statement() {
        let statements = parse("let greeting = \"hello\"").expect("source should parse");

        match &statements[0] {
            Stmt::Let {
                name,
                value: Expr::String(value),
                ..
            } => {
                assert_eq!(name, "greeting");
                assert_eq!(value, "hello");
            }
            other => panic!("expected a string let, got {other:?}"),
        }
    }

    #[test]
    fn parses_string_concatenation() {
        let statements = parse("let s = \"a\" + \"b\"").expect("source should parse");

        match &statements[0] {
            Stmt::Let {
                value:
                    Expr::Binary {
                        left,
                        right,
                        operator: BinaryOperator::Plus,
                    },
                ..
            } => {
                assert!(matches!(**left, Expr::String(ref value) if value == "a"));
                assert!(matches!(**right, Expr::String(ref value) if value == "b"));
            }
            other => panic!("expected string concatenation, got {other:?}"),
        }
    }

    #[test]
    fn respects_precedence_in_string_concatenation() {
        let statements = parse("let s = \"a\" + \"b\" + \"c\"").expect("source should parse");

        match &statements[0] {
            Stmt::Let {
                value:
                    Expr::Binary {
                        left,
                        operator: BinaryOperator::Plus,
                        right,
                    },
                ..
            } => {
                assert!(matches!(**right, Expr::String(ref value) if value == "c"));
                assert!(matches!(
                    **left,
                    Expr::Binary {
                        operator: BinaryOperator::Plus,
                        ..
                    }
                ));
            }
            other => panic!("expected left associative concatenation, got {other:?}"),
        }
    }

    #[test]
    fn rejects_string_as_binding_name() {
        let error = parse("let \"name\" = 1").expect_err("a string is not an identifier");
        assert_eq!(error.message, "expected identifier");
    }

    #[test]
    fn rejects_missing_operand_after_string() {
        let error = parse("let s = \"a\" +").expect_err("missing right operand");
        assert_eq!(error.message, "expected expression");
    }
}
