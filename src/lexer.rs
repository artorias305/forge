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

    OpEqEq,
    OpGt,
    OpGtEq,
    OpLt,
    OpLtEq,
    OpNEq,
    OpNot,

    KwTrue,
    KwFalse,

    KwIf,
    KwElse,

    LeftBrace,
    RightBrace,
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
    Boolean(bool),
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

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    fn read_escape(&mut self, line: usize) -> Result<char, LexerError> {
        let escaped = self
            .advance()
            .ok_or_else(|| LexerError::new(line, "unterminated string".to_string()))?;

        match escaped {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '\n' => Err(LexerError::new(
                line,
                "unexpected line break in string".to_string(),
            )),
            _ => Err(LexerError::new(
                line,
                format!("invalid escape sequence '\\{escaped}'"),
            )),
        }
    }

    fn read_string(&mut self, line: usize) -> Result<String, LexerError> {
        let mut buf = String::new();

        loop {
            let Some(ch) = self.advance() else {
                return Err(LexerError::new(line, "unterminated string".to_string()));
            };

            match ch {
                '"' => return Ok(buf),
                '\\' => buf.push(self.read_escape(line)?),
                _ => buf.push(ch),
            }
        }
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

                    let (kind, literal) = match buf.to_lowercase().as_str() {
                        "let" => (TokenKind::KwLet, Literal::Nil),
                        "print" => (TokenKind::KwPrint, Literal::Nil),
                        "true" => (TokenKind::KwTrue, Literal::Boolean(true)),
                        "false" => (TokenKind::KwFalse, Literal::Boolean(false)),
                        "if" => (TokenKind::KwIf, Literal::Nil),
                        "else" => (TokenKind::KwElse, Literal::Nil),
                        _ => (TokenKind::Identifier, Literal::Nil),
                    };

                    tokens.push(Token::new(kind, literal, buf, line));
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
                '=' => {
                    if self.match_char('=') {
                        tokens.push(Token::new(
                            TokenKind::OpEqEq,
                            Literal::Nil,
                            "==".to_string(),
                            line,
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Equals,
                            Literal::Nil,
                            "=".to_string(),
                            line,
                        ));
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        tokens.push(Token::new(
                            TokenKind::OpGtEq,
                            Literal::Nil,
                            ">=".to_string(),
                            line,
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenKind::OpGt,
                            Literal::Nil,
                            ">".to_string(),
                            line,
                        ));
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        tokens.push(Token::new(
                            TokenKind::OpLtEq,
                            Literal::Nil,
                            "<=".to_string(),
                            line,
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenKind::OpLt,
                            Literal::Nil,
                            "<".to_string(),
                            line,
                        ));
                    }
                }
                '!' => {
                    if self.match_char('=') {
                        tokens.push(Token::new(
                            TokenKind::OpNEq,
                            Literal::Nil,
                            "!=".to_string(),
                            line,
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenKind::OpNot,
                            Literal::Nil,
                            "!".to_string(),
                            line,
                        ));
                    }
                }
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
                    let value = self.read_string(line)?;
                    tokens.push(Token::new(
                        TokenKind::String,
                        Literal::String(value.clone()),
                        value,
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
                '{' => {
                    tokens.push(Token::new(
                        TokenKind::LeftBrace,
                        Literal::Nil,
                        "{".to_string(),
                        line,
                    ));
                }
                '}' => {
                    tokens.push(Token::new(
                        TokenKind::RightBrace,
                        Literal::Nil,
                        "}".to_string(),
                        line,
                    ));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(source: &str) -> Result<Vec<Token>, LexerError> {
        Lexer::new(source).tokenize()
    }

    fn string_value(source: &str) -> String {
        let tokens = tokenize(source).expect("source should lex");

        match tokens.into_iter().next().expect("expected one token") {
            Token {
                kind: TokenKind::String,
                literal: Literal::String(value),
                ..
            } => value,
            other => panic!("expected a string token, got {other:?}"),
        }
    }

    #[test]
    fn lexes_plain_string() {
        assert_eq!(string_value("\"hello\""), "hello");
    }

    #[test]
    fn lexes_empty_string() {
        assert_eq!(string_value("\"\""), "");
    }

    #[test]
    fn lexes_escapes() {
        assert_eq!(string_value(r#""a\"b\\c\nd\te""#), "a\"b\\c\nd\te");
    }

    #[test]
    fn rejects_invalid_escape() {
        let error = tokenize(r#""a\qb""#).expect_err("invalid escape should fail");
        assert!(
            error.message.contains("invalid escape sequence"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_line_break_after_escape() {
        let error = tokenize("\"a\\\nb\"").expect_err("escaped newline should fail");
        assert_eq!(error.message, "unexpected line break in string");
    }

    #[test]
    fn rejects_unterminated_string() {
        let error = tokenize("\"abc").expect_err("unterminated string should fail");
        assert_eq!(error.message, "unterminated string");
    }

    #[test]
    fn rejects_unterminated_escape() {
        let error = tokenize("\"abc\\").expect_err("unterminated escape should fail");
        assert_eq!(error.message, "unterminated string");
    }

    #[test]
    fn tracks_lines_after_multiline_string() {
        let tokens = tokenize("\"a\nb\"\nprint(1)").expect("source should lex");
        let string = &tokens[0];
        let print = &tokens[1];

        assert_eq!(string.line, 1);
        assert!(matches!(print.kind, TokenKind::KwPrint));
        assert_eq!(print.line, 3);
    }

    #[test]
    fn ignores_comment_after_string() {
        let tokens = tokenize("\"a\" # trailing").expect("source should lex");
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn preserves_rhs_after_single_character_operators() {
        let tokens = tokenize("1<2 !true").expect("source should lex");

        assert!(matches!(tokens[0].kind, TokenKind::Number));
        assert!(matches!(tokens[1].kind, TokenKind::OpLt));
        assert!(matches!(tokens[2].kind, TokenKind::Number));
        assert!(matches!(tokens[3].kind, TokenKind::OpNot));
        assert!(matches!(tokens[4].kind, TokenKind::KwTrue));
    }
}
