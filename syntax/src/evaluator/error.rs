use crate::tokenizer::{Token, TokenType};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum EvaluatorErrorType {
    ZeroDivision,
    VariableDivision {
        numerator: Token,
        denominator: Token,
    },
    VariableMultiplication {
        left: Token,
        right: Token,
    },
    InvalidBinaryOperator,
    NonConstantExponent,
    NonConstantBase,
}

pub type EvaluatorResult<T> = Result<T, EvaluatorError>;

#[derive(Debug)]
pub struct EvaluatorError {
    pub error_type: EvaluatorErrorType,
    pub token: Token,
}

impl Display for EvaluatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use EvaluatorErrorType::*;
        match &self.error_type {
            ZeroDivision => write!(
                f,
                "Division by zero is not possible. Column {}",
                self.token.column
            ),
            VariableDivision { .. } => write!(
                f,
                "Cannot divide between a variable denominator. Column {}",
                self.token.column
            ),
            VariableMultiplication { .. } => write!(
                f,
                "Cannot multiply a variable times another variable. Column {}",
                self.token.column
            ),
            InvalidBinaryOperator => write!(
                f,
                "Token: {:?} is not a valid binary operator. Column {}",
                self.token, self.token.column
            ),
            NonConstantBase => write!(
                f,
                "The base of an exponentiation operation may only be a constant. Found {:?}, column {}",
                self.token, self.token.column
            ),
            NonConstantExponent => write!(
                f,
                "The exponent of an exponentiation operation may only be a constant. Found {:?} in column {}",
                self.token, self.token.column
            ),
        }
    }
}
