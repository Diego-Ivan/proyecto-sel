use std::fmt::{Display, Formatter};

pub type SimplifierResult<T> = Result<T, SimplifierError>;

#[derive(Debug)]
pub enum SimplifierError {
    LexerError(crate::lexer::LexerError),
    TokenizerError(crate::tokenizer::TokenizerError),
    EvaluatorError(crate::evaluator::EvaluatorError),
}

impl From<crate::tokenizer::TokenizerError> for SimplifierError {
    fn from(err: crate::tokenizer::TokenizerError) -> Self {
        Self::TokenizerError(err)
    }
}

impl From<crate::evaluator::EvaluatorError> for SimplifierError {
    fn from(err: crate::evaluator::EvaluatorError) -> Self {
        Self::EvaluatorError(err)
    }
}

impl From<crate::lexer::LexerError> for SimplifierError {
    fn from(err: crate::lexer::LexerError) -> Self {
        Self::LexerError(err)
    }
}

impl Display for SimplifierError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenizerError(err) => {
                write!(f, "Syntax Error: {err}")
            }
            Self::EvaluatorError(err) => {
                write!(f, "Evaluation Error: {err}")
            }
            Self::LexerError(err) => write!(f, "Lexer error: {err}"),
        }
    }
}
