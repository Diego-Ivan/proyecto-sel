#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Number(f64),
    Identifier(String),
    Star,
    Plus,
    Slash,
    Minus,
    LeftParen,
    RightParen,
    Equal,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            column
        }
    }
}