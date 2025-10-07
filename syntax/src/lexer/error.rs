use crate::tokenizer::{TokenType, TokenizerError};
use std::error::Error;
use std::fmt::{Display, Formatter};

pub type LexerResult<T> = Result<T, LexerError>;

#[derive(Debug)]
pub enum LexerError {
    TokenizerError(TokenizerError),
    WrongToken {
        found: TokenType,
        expected: TokenType,
    },
    ExpectedTokenFoundEof {
        expected: TokenType,
    },
    UnexpectedEof,
    ExpectedEof {
        found: TokenType,
    },
    ExpectedPrimary {
        found: TokenType,
    },
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenizerError(e) => write!(f, "Syntax Error: {e}"),
            Self::WrongToken { found, expected } => {
                write!(
                    f,
                    "Expected token {expected:?}, but found {found:?} instead"
                )
            }
            Self::ExpectedTokenFoundEof { expected } => write!(
                f,
                "Expected token {expected:?}, but the input ended unexpectedly"
            ),
            Self::UnexpectedEof => f.write_str("Unexpected end of file"),
            Self::ExpectedPrimary { found } => write!(
                f,
                "Expected number, identifier or left parenthesis, but found {found:?} instead"
            ),
            Self::ExpectedEof { found } => write!(f, "Expected EOF, found {found:?} instead"),
        }
    }
}

impl Error for LexerError {}
