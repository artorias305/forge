#[derive(Debug, Clone)]
pub enum TokenKind {
    KwLet,
    Identifier,
    Number,
    KwPrint,

    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, Copy)]
pub enum NumberLiteral {
    Int(i32),
    Float(f64),
}

impl NumberLiteral {
    fn from(s: &str) -> Self {
        match s.contains('.') {
            true => {
                // yayy we are floating
                Self::Float(s.parse::<f64>().unwrap())
            }
            false => Self::Int(s.parse::<i32>().unwrap()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Nil,
    Number(NumberLiteral),
}

impl From<NumberLiteral> for Literal {
    fn from(value: NumberLiteral) -> Self {
        Literal::Number(value)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: Literal,
    pub lexeme: String,
}

impl Token {
    fn new(kind: TokenKind, literal: Literal, lexeme: String) -> Self {
        Self {
            kind,
            literal,
            lexeme,
        }
    }
}

pub struct Lexer {
    source: String,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(ch) = self.advance() {
            match ch {
                'a'..='z' => {
                    let mut buf = String::from(ch);

                    while let Some(r) = self.peek()
                        && (r.is_ascii_alphanumeric() || r == '_')
                    {
                        buf.push(self.advance().unwrap());
                    }

                    match buf.to_lowercase().as_str() {
                        "let" => {
                            tokens.push(Token::new(TokenKind::KwLet, Literal::Nil, buf));
                        }

                        "print" => {
                            tokens.push(Token::new(TokenKind::KwPrint, Literal::Nil, buf));
                        }

                        _ => {
                            tokens.push(Token::new(TokenKind::Identifier, Literal::Nil, buf));
                        }
                    }
                }

                '0'..='9' => {
                    // number literal
                    let mut buf = String::from(ch);
                    let mut has_dot = false;

                    while let Some(r) = self.peek() {
                        if r.is_digit(10) {
                            buf.push(self.advance().unwrap());
                        } else if r == '.' && !has_dot {
                            has_dot = true;
                            buf.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }

                    tokens.push(Token::new(
                        TokenKind::Number,
                        Literal::from(NumberLiteral::from(buf.as_str())),
                        buf,
                    ));
                }
                '(' => {
                    tokens.push(Token::new(
                        TokenKind::LeftParen,
                        Literal::Nil,
                        "(".to_string(),
                    ));
                }
                ')' => {
                    tokens.push(Token::new(
                        TokenKind::RightParen,
                        Literal::Nil,
                        ")".to_string(),
                    ));
                }
                _ => {}
            }
        }

        tokens
    }
}
