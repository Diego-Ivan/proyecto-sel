use crate::tokenizer::Token;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum ExpressionType {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
    Negation(Box<Expression>),
    Number(f64),
    Variable(String),
    FunctionCall {
        name: String,
        parameter: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub expression_type: ExpressionType,
    pub token: Token,
}

fn parenthesize(f: &mut Formatter<'_>, token: &str, exprs: &[&Expression]) -> std::fmt::Result {
    f.write_str("(")?;
    f.write_str(token)?;
    for expr in exprs {
        write!(f, " {expr}")?;
    }
    f.write_str(")")
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.expression_type {
            ExpressionType::Binary {
                left,
                operator,
                right,
            } => parenthesize(f, &operator.lexeme, &[&left, &right]),
            ExpressionType::Variable(varname) => f.write_str(varname),
            ExpressionType::Grouping(expr) => parenthesize(f, "group", &[&expr]),
            ExpressionType::Number(num) => write!(f, "{num}"),
            ExpressionType::Negation(expr) => parenthesize(f, "-", &[&expr]),
            ExpressionType::FunctionCall { name, parameter } => {
                write!(f, "(call {name} {parameter})")
            }
        }
    }
}
