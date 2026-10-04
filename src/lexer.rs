#[derive(Debug)]
enum TokenKind {
    KW_LET,
    IDENTIFIER,
}

#[derive(Debug)]
enum NumberLiteral {
    Int(i32),
    Float(f64),
}

#[derive(Debug)]
enum Literal {
    Nil,
    Number(NumberLiteral),
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    literal: Literal,
    lexeme: String,
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
                    // could be identifier or keyword
                    let mut buf = String::from(ch);

                    while let Some(r) = self.peek()
                        && !r.is_whitespace()
                    {
                        buf.push(self.advance().unwrap());
                    }

                    match buf.to_lowercase().as_str() {
                        "let" => {
                            tokens.push(Token::new(TokenKind::KW_LET, Literal::Nil, buf));
                        }
                        _ => {
                            tokens.push(Token::new(TokenKind::IDENTIFIER, Literal::Nil, buf));
                        }
                    }
                }
                _ => {}
            }
        }

        tokens
    }
}
