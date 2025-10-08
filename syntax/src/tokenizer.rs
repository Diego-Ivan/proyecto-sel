mod error;
mod token;

use std::io::BufRead;

pub use error::{TokenizerError, TokenizerResult};
pub use token::{Token, TokenType};

const DECIMAL_SEPARATOR: u8 = b'.';

#[derive(Debug, PartialEq, Eq)]
enum NumberParseSection {
    Integer,
    Decimal,
}

pub struct Tokenizer<R: BufRead> {
    input: R,
    column: usize,
    current_byte: Option<u8>,
}

impl<R: BufRead> Tokenizer<R> {
    pub fn new(input: R) -> Self {
        Self {
            input,
            column: 0,
            current_byte: None,
        }
    }

    fn scan_token(&mut self) -> Option<TokenizerResult<Token>> {
        use TokenType::*;
        let current = self.consume_whitespace()?;
        let mut lexeme: Vec<u8> = Vec::new();

        macro_rules! push_token {
            ($tt: ident) => {{
                lexeme.push(current);
                self.add_token($tt, lexeme)
            }};
        }

        let token = match current {
            b'(' => push_token!(LeftParen),
            b')' => push_token!(RightParen),
            b'+' => push_token!(Plus),
            b'-' => push_token!(Minus),
            b'*' => push_token!(Star),
            b'=' => push_token!(Equal),
            b'/' => push_token!(Slash),
            b'^' => push_token!(Hat),
            b'0'..=b'9' => {
                lexeme.push(current);
                self.consume_number(lexeme)
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                lexeme.push(current);
                self.consume_identifier(lexeme)
            }
            a => Err(error::TokenizerError::UnknownCharacter(a, self.column)),
        };

        Some(token)
    }

    fn add_token(&mut self, token_type: TokenType, lexeme: Vec<u8>) -> TokenizerResult<Token> {
        self.add_token_with_column(token_type, lexeme, self.column)
    }

    fn add_token_with_column(
        &self,
        token_type: TokenType,
        lexeme: Vec<u8>,
        column: usize,
    ) -> TokenizerResult<Token> {
        let lexeme = self.lexeme_into_utf8(lexeme)?;

        Ok(Token::new(token_type, lexeme, column - 1))
    }

    fn advance(&mut self) -> Option<u8> {
        let mut buf = [0u8; 1];
        match self.input.read_exact(&mut buf) {
            Ok(_) => {
                let current_byte = self.current_byte.take();

                self.current_byte = Some(buf[0]);
                // This will only happen on the last byte
                self.column += 1;
                current_byte
            }
            /*
             * If we have finished reading from the Reader, it is still also possible that
             * we have one single byte remaining on the scanner, which would be the current byte
             */
            Err(_) => {
                self.column += 1;
                self.current_byte.take()
            }
        }
    }

    fn consume_number(&mut self, mut lexeme: Vec<u8>) -> TokenizerResult<Token> {
        // Parse the first digit.
        let mut decimal: f64 = (lexeme[0] - 0x30) as f64;
        let mut decimal_power = 0;
        let mut current_part = NumberParseSection::Integer;
        let first_col = self.column;

        while let Some(c) = self.current_byte {
            if c == DECIMAL_SEPARATOR {
                if current_part == NumberParseSection::Decimal {
                    break;
                }
                current_part = NumberParseSection::Decimal;
                self.advance();
                lexeme.push(c);
                continue;
            }

            if !c.is_ascii_digit() {
                break;
            }

            let current_value = (c - 0x30) as f64;
            lexeme.push(c);

            match current_part {
                NumberParseSection::Integer => {
                    decimal *= 10f64;
                    decimal += current_value;
                }
                NumberParseSection::Decimal => {
                    decimal_power -= 1;
                    decimal += current_value * 10f64.powi(decimal_power);
                }
            }
            self.advance();
        }

        self.add_token_with_column(TokenType::Number(decimal), lexeme, first_col)
    }

    fn consume_identifier(&mut self, lexeme: Vec<u8>) -> TokenizerResult<Token> {
        self.add_token(
            TokenType::Identifier(self.lexeme_into_utf8(lexeme.clone())?),
            lexeme,
        )
    }

    fn consume_whitespace(&mut self) -> Option<u8> {
        loop {
            let current = self.advance()?;
            match current {
                b'\n' | b'\r' => {
                    self.column = 0;
                }
                b' ' | b'\t' => {}

                _ => break Some(current),
            }
        }
    }

    fn lexeme_into_utf8(&self, lexeme: Vec<u8>) -> TokenizerResult<String> {
        match String::from_utf8(lexeme) {
            Ok(s) => Ok(s),
            Err(_) => Err(error::TokenizerError::NoUtf8(self.column)),
        }
    }
}

impl<R: BufRead> Iterator for Tokenizer<R> {
    type Item = TokenizerResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.column == 0 {
            self.advance();
        }
        self.scan_token()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::Token;
    use crate::tokenizer::token::TokenType;
    use std::io::Cursor;

    #[test]
    fn test_identifier() {
        let source = "x yz a";
        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(
                    TokenType::Identifier(String::from("x")),
                    String::from("x"),
                    1
                ),
                Token::new(
                    TokenType::Identifier(String::from("y")),
                    String::from("y"),
                    3
                ),
                Token::new(
                    TokenType::Identifier(String::from("z")),
                    String::from("z"),
                    4
                ),
                Token::new(
                    TokenType::Identifier(String::from("a")),
                    String::from("a"),
                    6
                ),
            ]
        )
    }

    #[test]
    fn test_number_literal() {
        let source = "1.5 2.5 10 32.5 1.2";
        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Number(1.5), String::from("1.5"), 1),
                Token::new(TokenType::Number(2.5), String::from("2.5"), 5),
                Token::new(TokenType::Number(10.0), String::from("10"), 9),
                Token::new(TokenType::Number(32.5), String::from("32.5"), 12),
                Token::new(TokenType::Number(1.2), String::from("1.2"), 17),
            ]
        )
    }

    #[test]
    fn test_sum_expression() {
        let source = "1.5x + 3y +2";
        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Number(1.5), String::from("1.5"), 1),
                Token::new(
                    TokenType::Identifier(String::from("x")),
                    String::from("x"),
                    4
                ),
                Token::new(TokenType::Plus, String::from("+"), 6),
                Token::new(TokenType::Number(3.0), String::from("3"), 8),
                Token::new(
                    TokenType::Identifier(String::from("y")),
                    String::from("y"),
                    9
                ),
                Token::new(TokenType::Plus, String::from("+"), 11),
                Token::new(TokenType::Number(2.0), String::from("2"), 12),
            ]
        );
    }

    #[test]
    fn test_subtract_expression() {
        let source = "1.5x - 3y -2";
        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Number(1.5), String::from("1.5"), 1),
                Token::new(
                    TokenType::Identifier(String::from("x")),
                    String::from("x"),
                    4
                ),
                Token::new(TokenType::Minus, String::from("-"), 6),
                Token::new(TokenType::Number(3.0), String::from("3"), 8),
                Token::new(
                    TokenType::Identifier(String::from("y")),
                    String::from("y"),
                    9
                ),
                Token::new(TokenType::Minus, String::from("-"), 11),
                Token::new(TokenType::Number(2.0), String::from("2"), 12),
            ]
        );
    }

    #[test]
    fn test_grouping_multiplication() {
        let source = "(1.5x - 3y) * 2 / 4";
        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::LeftParen, String::from("("), 1),
                Token::new(TokenType::Number(1.5), String::from("1.5"), 2),
                Token::new(
                    TokenType::Identifier(String::from("x")),
                    String::from("x"),
                    5
                ),
                Token::new(TokenType::Minus, String::from("-"), 7),
                Token::new(TokenType::Number(3.0), String::from("3"), 9),
                Token::new(
                    TokenType::Identifier(String::from("y")),
                    String::from("y"),
                    10
                ),
                Token::new(TokenType::RightParen, String::from(")"), 11),
                Token::new(TokenType::Star, String::from("*"), 13),
                Token::new(TokenType::Number(2.0), String::from("2"), 15),
                Token::new(TokenType::Slash, String::from("/"), 17),
                Token::new(TokenType::Number(4.0), String::from("4"), 19),
            ]
        );
    }

    #[test]
    fn test_simple_equality() {
        let source = "3 = 3";
        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Number(3.0), String::from("3"), 1),
                Token::new(TokenType::Equal, String::from("="), 3),
                Token::new(TokenType::Number(3.0), String::from("3"), 5),
            ]
        );
    }

    #[test]
    fn test_exponent() {
        let source = "3^2 = 9^(y + 2)";

        let scanner = super::Tokenizer::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|t| t.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Number(3.0), String::from("3"), 1),
                Token::new(TokenType::Hat, String::from("^"), 2),
                Token::new(TokenType::Number(2.0), String::from("2"), 3),
                Token::new(TokenType::Equal, String::from("="), 5),
                Token::new(TokenType::Number(9.0), String::from("9"), 7),
                Token::new(TokenType::Hat, String::from("^"), 8),
                Token::new(TokenType::LeftParen, String::from("("), 9),
                Token::new(
                    TokenType::Identifier(String::from("y")),
                    String::from("y"),
                    10
                ),
                Token::new(TokenType::Plus, String::from("+"), 12),
                Token::new(TokenType::Number(2.0), String::from("2"), 14),
                Token::new(TokenType::RightParen, String::from(")"), 15)
            ]
        )
    }
}
