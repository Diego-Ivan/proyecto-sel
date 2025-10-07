use std::error::Error;
use std::fmt::{Display, Formatter};

pub type TokenizerResult<T> = Result<T, TokenizerError>;

#[derive(Debug)]
pub enum TokenizerError {
    UnknownCharacter(u8, usize),
    NoUtf8(usize),
}

impl Display for TokenizerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCharacter(c, col) => write!(f, "Character {c} is not recognized by the tokenizer in column {col}"),
            Self::NoUtf8(col) => write!(f, "Input string contains non-UTF8 sequences in column {col}"),
        }
    }
}

impl Error for TokenizerError {}