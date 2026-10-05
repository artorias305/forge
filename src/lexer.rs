use std::fmt;

#[derive(Debug, Clone)]
pub enum TokenKind {
    KwLet,
    Identifier,
    Number,
    KwPrint,
    LeftParen,
    RightParen,
    Equals,
    Plus,
    Minus,
    Star,
    Slash,
    String,
}

#[derive(Debug, Clone, Copy)]
pub enum NumberLiteral {
    Int(i32),
    Float(f64),
}

impl NumberLiteral {
    fn parse(s: &str) -> Result<Self, LexerError> {
        if s.contains('.') {
            s.parse::<f64>()
                .map(Self::Float)
                .map_err(|_| LexerError::new(0, format!("invalid number '{s}'")))
        } else {
            s.parse::<i32>()
                .map(Self::Int)
                .map_err(|_| LexerError::new(0, format!("invalid number '{s}'")))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Nil,
    Number(NumberLiteral),
    String(String),
}

impl From<NumberLiteral> for Literal {
    fn from(value: NumberLiteral) -> Self {
        Literal::Number(value)
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Literal::String(value)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: Literal,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    fn new(kind: TokenKind, literal: Literal, lexeme: String, line: usize) -> Self {
        Self {
            kind,
            literal,
            lexeme,
            line,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub line: usize,
    pub message: String,
}

impl LexerError {
    fn new(line: usize, message: String) -> Self {
        Self { line, message }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(ch) = self.advance() {
            let line = self.line - usize::from(ch == '\n');
            match ch {
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut buf = String::from(ch);

                    while let Some(next) = self.peek()
                        && (next.is_ascii_alphanumeric() || next == '_')
                    {
                        buf.push(self.advance().expect("peeked character must exist"));
                    }

                    let kind = match buf.to_lowercase().as_str() {
                        "let" => TokenKind::KwLet,
                        "print" => TokenKind::KwPrint,
                        _ => TokenKind::Identifier,
                    };
                    tokens.push(Token::new(kind, Literal::Nil, buf, line));
                }
                '0'..='9' => {
                    let mut buf = String::from(ch);
                    let mut has_dot = false;

                    while let Some(next) = self.peek() {
                        if next.is_ascii_digit() {
                            buf.push(self.advance().expect("peeked character must exist"));
                        } else if next == '.' && !has_dot {
                            has_dot = true;
                            buf.push(self.advance().expect("peeked character must exist"));
                        } else {
                            break;
                        }
                    }

                    let literal = NumberLiteral::parse(&buf)
                        .map_err(|error| LexerError::new(line, error.message))?;
                    tokens.push(Token::new(
                        TokenKind::Number,
                        Literal::from(literal),
                        buf,
                        line,
                    ));
                }
                '(' => tokens.push(Token::new(
                    TokenKind::LeftParen,
                    Literal::Nil,
                    "(".to_string(),
                    line,
                )),
                ')' => tokens.push(Token::new(
                    TokenKind::RightParen,
                    Literal::Nil,
                    ")".to_string(),
                    line,
                )),
                '=' => tokens.push(Token::new(
                    TokenKind::Equals,
                    Literal::Nil,
                    "=".to_string(),
                    line,
                )),
                '+' => tokens.push(Token::new(
                    TokenKind::Plus,
                    Literal::Nil,
                    "+".to_string(),
                    line,
                )),
                '-' => tokens.push(Token::new(
                    TokenKind::Minus,
                    Literal::Nil,
                    "-".to_string(),
                    line,
                )),
                '*' => tokens.push(Token::new(
                    TokenKind::Star,
                    Literal::Nil,
                    "*".to_string(),
                    line,
                )),
                '/' => tokens.push(Token::new(
                    TokenKind::Slash,
                    Literal::Nil,
                    "/".to_string(),
                    line,
                )),
                '"' => {
                    let mut buf = String::new();
                    let mut terminated = false;

                    while let Some(next) = self.advance() {
                        if next == '"' {
                            terminated = true;
                            break;
                        }
                        buf.push(next);
                    }

                    if !terminated {
                        return Err(LexerError::new(line, "unterminated string".to_string()));
                    }
                    tokens.push(Token::new(
                        TokenKind::String,
                        Literal::String(buf.clone()),
                        buf,
                        line,
                    ));
                }
                '#' => {
                    while let Some(next) = self.peek() {
                        if next == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                c if c.is_whitespace() => {}
                _ => {
                    return Err(LexerError::new(
                        line,
                        format!("unexpected character '{ch}'"),
                    ));
                }
            }
        }

        Ok(tokens)
    }
}
