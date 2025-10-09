use std::collections::HashMap;
use std::io::Cursor;

use crate::{
    evaluator::{Evaluator, Value},
    lexer::Lexer,
    simplifier::error::SimplifierResult,
    tokenizer::Tokenizer,
};
mod error;

pub struct Simplifier();

pub struct CanonicalEquation {
    pub terms: HashMap<String, f64>,
    pub constant: f64,
}

impl Simplifier {
    pub fn simplify_equation(&self, user_input: &str) -> SimplifierResult<CanonicalEquation> {
        let mut terms = HashMap::new();
        let mut constant = 0.0f64;

        let tokenizer = Tokenizer::new(Cursor::new(user_input));
        let mut tokens = Vec::new();

        for token in tokenizer {
            tokens.push(token?);
        }

        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation()?;

        let evaluator = Evaluator::new();

        let left = evaluator.evaluate_expression(&equation.left)?;
        let right = evaluator.evaluate_expression(&equation.right)?;

        match left {
            Value::Monomial {
                coefficient,
                variable,
            } => match variable {
                Some(variable) => *terms.entry(variable).or_default() += coefficient,
                None => constant += coefficient,
            },
            Value::Sum(values) => self.simplify_into_map(values, &mut terms, &mut constant, 1.0),
        }

        match right {
            Value::Monomial {
                coefficient,
                variable,
            } => match variable {
                Some(variable) => *terms.entry(variable).or_default() += coefficient,
                None => constant -= coefficient,
            },

            Value::Sum(values) => self.simplify_into_map(values, &mut terms, &mut constant, -1.0),
        }

        Ok(CanonicalEquation {
            terms: terms,
            constant,
        })
    }

    fn simplify_into_map(
        &self,
        values: Vec<Value>,
        terms_map: &mut HashMap<String, f64>,
        constant: &mut f64,
        multiply_by: f64,
    ) {
        for value in values {
            match value {
                Value::Monomial {
                    coefficient,
                    variable,
                } => match variable {
                    Some(variable) => {
                        *terms_map.entry(variable).or_default() += coefficient * multiply_by
                    }
                    None => *constant += coefficient * multiply_by * -1.0,
                },

                Value::Sum(values) => {
                    self.simplify_into_map(values, terms_map, constant, multiply_by)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::simplifier::Simplifier;
    use std::collections::HashMap;

    #[test]
    pub fn test_sums() {
        let expr = "2x + 3x - 2 = x + y + 2";
        let simplifier = Simplifier();

        let result = simplifier.simplify_equation(expr).unwrap();

        assert_eq!(
            result.terms,
            HashMap::from([(String::from("x"), 4.0f64), (String::from("y"), -1.0)])
        );
        assert_eq!(result.constant, 4.0)
    }

    #[test]
    pub fn test_substraction() {
        let expr = "-2x -6x -3y = -5 -x -y -10";
        let simplifier = Simplifier();

        let result = simplifier.simplify_equation(expr).unwrap();

        assert_eq!(
            result.terms,
            HashMap::from([(String::from("x"), -7.0f64), (String::from("y"), -2.0)])
        );

        assert_eq!(result.constant, -15.0)
    }

    #[test]
    pub fn test_multiplication() {
        let expr = "3*(x + 2y -5) = -4*(-8y + 10x + 2)";

        let simplifier = Simplifier();

        let result = simplifier.simplify_equation(expr).unwrap();

        assert_eq!(
            result.terms,
            HashMap::from([(String::from("x"), 43.0f64), (String::from("y"), -26.0)])
        );

        assert_eq!(result.constant, 7.0);
    }

    #[test]
    pub fn test_division() {
        let expr = "(24x + 12y + 6)/3 = 0";

        let simplifier = Simplifier();

        let result = simplifier.simplify_equation(expr).unwrap();

        assert_eq!(
            result.terms,
            HashMap::from([(String::from("x"), 8.0f64), (String::from("y"), 4.0)])
        );

        assert_eq!(result.constant, -2.0);
    }
}
