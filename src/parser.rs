use crate::lexer::{Literal, NumberLiteral, Token, TokenKind};

#[derive(Debug)]
pub enum Expr {
    Number(NumberLiteral),
    Identifier(String),
}

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    Let { name: String, value: Expr },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(token)
    }

    fn parse_expr(&mut self) -> Expr {
        let token = self.advance().expect("expected expression");

        match token.kind {
            TokenKind::Number => match token.literal {
                Literal::Number(number) => Expr::Number(number),
                _ => unreachable!(),
            },

            TokenKind::Identifier => Expr::Identifier(token.lexeme),

            _ => panic!("expected expression"),
        }
    }

    fn parse_statement(&mut self) -> Stmt {
        let token = self.advance().expect("expected statement");

        match token.kind {
            TokenKind::KwPrint => {
                if !matches!(
                    self.advance().expect("expected token").kind,
                    TokenKind::LeftParen,
                ) {
                    panic!("expected '('");
                }

                let expr = self.parse_expr();

                if !matches!(
                    self.advance().expect("expected token").kind,
                    TokenKind::RightParen
                ) {
                    panic!("expected ')'");
                }

                Stmt::Print(expr)
            }
            TokenKind::KwLet => {
                let name_token = self.advance().expect("expected token");

                if !matches!(name_token.kind, TokenKind::Identifier) {
                    panic!("expected identifier");
                }

                let name = name_token.lexeme;

                if !matches!(
                    self.advance().expect("expected '='").kind,
                    TokenKind::Equals
                ) {
                    panic!("expected '='");
                }

                let value = self.parse_expr();

                Stmt::Let { name, value }
            }
            _ => panic!("expected statement"),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while self.peek().is_some() {
            statements.push(self.parse_statement());
        }

        statements
    }
}
