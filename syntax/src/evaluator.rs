mod error;
mod value;

pub use crate::evaluator::error::EvaluatorError;
use crate::evaluator::error::{EvaluatorErrorType, EvaluatorResult};
pub use crate::evaluator::value::Value;
use crate::expression::{Expression, ExpressionType};
use crate::tokenizer::{Token, TokenType};

pub struct Evaluator();

impl Evaluator {
    pub fn evaluate_expression(&self, expression: &Expression) -> EvaluatorResult<Value> {
        match &expression.expression_type {
            ExpressionType::Number(num) => Ok(Value::new_constant(*num)),

            ExpressionType::Negation(expr) => self.evaluate_expression(expr).map(|v| v.negate()),

            ExpressionType::Variable(varname) => Ok(Value::new_monomial(1.0, varname.clone())),

            ExpressionType::Grouping(expression) => self.evaluate_expression(expression),

            ExpressionType::Binary {
                left,
                operator,
                right,
            } => match operator.token_type {
                TokenType::Plus => self.evaluate_addition(left, right),
                TokenType::Minus => self.evaluate_subtraction(left, right),
                TokenType::Star => self.evaluate_multiplication(left, right),
                TokenType::Slash => self.evaluate_division(left, right),
                TokenType::Hat => self.evaluate_exponent(left, right),

                _ => Err(EvaluatorError {
                    error_type: EvaluatorErrorType::InvalidBinaryOperator,
                    token: operator.clone(),
                }),
            },
        }
    }

    fn evaluate_exponent(
        &self,
        left: &Expression,
        exponent: &Expression,
    ) -> EvaluatorResult<Value> {
        let left_result = self.evaluate_expression(left)?;
        let exponent_value = self.evaluate_expression(exponent)?;

        match (left_result, exponent_value) {
            (
                Value::Monomial {
                    coefficient: c1,
                    variable: v1,
                },
                Value::Monomial {
                    coefficient: c2,
                    variable: v2,
                },
            ) => match (v1, v2) {
                (None, None) => Ok(Value::new_constant(c1.powf(c2))),
                (Some(_), _) => Err(EvaluatorError {
                    error_type: EvaluatorErrorType::NonConstantBase,
                    token: left.token.clone(),
                }),
                (_, Some(_)) => Err(EvaluatorError {
                    error_type: EvaluatorErrorType::NonConstantExponent,
                    token: exponent.token.clone(),
                }),
            },

            _ => Err(EvaluatorError {
                error_type: EvaluatorErrorType::NonConstantExponent,
                token: exponent.token.clone(),
            }),
        }
    }

    fn evaluate_addition(&self, left: &Expression, right: &Expression) -> EvaluatorResult<Value> {
        let left = self.evaluate_expression(left)?;
        let right = self.evaluate_expression(right)?;

        match (left, right) {
            (
                Value::Monomial {
                    coefficient: c1,
                    variable: v1,
                },
                Value::Monomial {
                    coefficient: c2,
                    variable: v2,
                },
            ) => {
                let value = match (v1, v2) {
                    (Option::None, Option::None) => Value::new_constant(c1 + c2),
                    (Some(v1), Some(v2)) if v1 == v2 => Value::new_monomial(c1 + c2, v1),
                    (Some(v), Option::None) => {
                        let left = Value::new_monomial(c1, v);
                        let right = Value::new_constant(c2);
                        Value::Sum(vec![left, right])
                    }
                    (Option::None, Some(v)) => {
                        let left = Value::new_constant(c1);
                        let right = Value::new_monomial(c2, v);
                        Value::Sum(vec![left, right])
                    }
                    (Some(v1), Some(v2)) => {
                        let left = Value::new_monomial(c1, v1);
                        let right = Value::new_monomial(c2, v2);
                        Value::Sum(vec![left, right])
                    }
                };

                Ok(value)
            }
            (
                Value::Sum(mut values),
                Value::Monomial {
                    coefficient,
                    variable,
                },
            ) => {
                values.push(Value::Monomial {
                    coefficient,
                    variable,
                });
                Ok(Value::Sum(values))
            }
            (
                Value::Monomial {
                    coefficient,
                    variable,
                },
                Value::Sum(mut values),
            ) => {
                values.push(Value::Monomial {
                    coefficient,
                    variable,
                });
                Ok(Value::Sum(values))
            }
            (Value::Sum(mut left_sum), Value::Sum(mut right_sum)) => {
                left_sum.append(&mut right_sum);
                Ok(Value::Sum(left_sum))
            }
        }
    }

    fn evaluate_subtraction(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> EvaluatorResult<Value> {
        let left = self.evaluate_expression(left)?;
        let right = self.evaluate_expression(right)?;

        match (left, right) {
            (
                Value::Monomial {
                    coefficient: c1,
                    variable: v1,
                },
                Value::Monomial {
                    coefficient: c2,
                    variable: v2,
                },
            ) => {
                let value = match (v1, v2) {
                    (Option::None, Option::None) => Value::new_constant(c1 - c2),
                    (Option::None, Some(v)) => {
                        let left = Value::new_constant(c1);
                        let right = Value::new_monomial(-c2, v);
                        Value::Sum(vec![left, right])
                    }
                    (Some(v), Option::None) => {
                        let left = Value::new_monomial(c1, v);
                        let right = Value::new_constant(-c2);
                        Value::Sum(vec![left, right])
                    }
                    (Some(v1), Some(v2)) if v1 == v2 => Value::new_monomial(c1 - c2, v1),
                    (Some(v1), Some(v2)) => {
                        let left = Value::new_monomial(c1, v1);
                        let right = Value::new_monomial(-c2, v2);
                        Value::Sum(vec![left, right])
                    }
                };

                Ok(value)
            }
            (
                Value::Sum(mut values),
                Value::Monomial {
                    coefficient,
                    variable,
                },
            ) => {
                values.push(Value::Monomial {
                    coefficient: -coefficient,
                    variable,
                });
                Ok(Value::Sum(values))
            }
            (
                Value::Monomial {
                    coefficient,
                    variable,
                },
                Value::Sum(values),
            ) => {
                let mut values_result = Vec::new();

                values_result.push(Value::Monomial {
                    coefficient,
                    variable,
                });

                for value in values.into_iter() {
                    let value = value.negate();
                    values_result.push(value);
                }
                Ok(Value::Sum(values_result))
            }
            (Value::Sum(mut left_sum), Value::Sum(mut right_sum)) => {
                left_sum.append(&mut right_sum);
                Ok(Value::Sum(left_sum))
            }
        }
    }

    fn evaluate_multiplication(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> EvaluatorResult<Value> {
        let left_result = self.evaluate_expression(left)?;
        let right_result = self.evaluate_expression(right)?;

        match (left_result, right_result) {
            (
                Value::Monomial {
                    coefficient: c1,
                    variable: v1,
                },
                Value::Monomial {
                    coefficient: c2,
                    variable: v2,
                },
            ) => match (v1, v2) {
                (Option::None, Option::None) => Ok(Value::Monomial {
                    coefficient: c1 * c2,
                    variable: None,
                }),
                (Some(v), Option::None) | (Option::None, Some(v)) => Ok(Value::Monomial {
                    coefficient: c1 * c2,
                    variable: Some(v.clone()),
                }),
                (Some(_), Some(_)) => Err(EvaluatorError {
                    error_type: EvaluatorErrorType::VariableMultiplication {
                        left: left.token.clone(),
                        right: right.token.clone(),
                    },
                    token: left.token.clone(),
                }),
            },
            (value_a, value_b) => {
                let left_values = match value_a {
                    Value::Sum(sum) => sum,
                    Value::Monomial {
                        coefficient,
                        variable,
                    } => vec![Value::Monomial {
                        coefficient,
                        variable,
                    }],
                };

                let right_values = match value_b {
                    Value::Sum(sum) => sum,
                    Value::Monomial {
                        coefficient,
                        variable,
                    } => vec![Value::Monomial {
                        coefficient,
                        variable,
                    }],
                };

                let values = self.evaluate_multiplication_values(
                    &left_values,
                    &right_values,
                    &left.token,
                    &right.token,
                )?;

                Ok(Value::Sum(values))
            }
        }
    }

    fn evaluate_multiplication_values(
        &self,
        left: &[Value],
        right: &[Value],
        left_token: &Token,
        right_token: &Token,
    ) -> EvaluatorResult<Vec<Value>> {
        let mut result = Vec::new();
        for left_value in left {
            for right_value in right {
                let mult_result = match (left_value, right_value) {
                    (
                        Value::Monomial {
                            coefficient: c1,
                            variable: v1,
                        },
                        Value::Monomial {
                            coefficient: c2,
                            variable: v2,
                        },
                    ) => match (v1, v2) {
                        (Option::None, Option::None) => Ok(Value::Monomial {
                            coefficient: c1 * c2,
                            variable: None,
                        }),
                        (Some(v), Option::None) | (Option::None, Some(v)) => Ok(Value::Monomial {
                            coefficient: c1 * c2,
                            variable: Some(v.clone()),
                        }),
                        (Some(_), Some(_)) => Err(EvaluatorError {
                            error_type: EvaluatorErrorType::VariableMultiplication {
                                left: left_token.clone(),
                                right: right_token.clone(),
                            },
                            token: left_token.clone(),
                        }),
                    },
                    (
                        Value::Monomial {
                            coefficient,
                            variable,
                        },
                        Value::Sum(list),
                    )
                    | (
                        Value::Sum(list),
                        Value::Monomial {
                            coefficient,
                            variable,
                        },
                    ) => {
                        let result = self.evaluate_multiplication_values(
                            &[Value::Monomial {
                                coefficient: *coefficient,
                                variable: variable.clone(),
                            }],
                            list,
                            left_token,
                            right_token,
                        )?;

                        Ok(Value::Sum(result))
                    }
                    (Value::Sum(sum1), Value::Sum(sum2)) => {
                        let result = self.evaluate_multiplication_values(
                            sum1,
                            sum2,
                            left_token,
                            right_token,
                        )?;

                        Ok(Value::Sum(result))
                    }
                };
                result.push(mult_result?);
            }
        }
        Ok(result)
    }

    fn evaluate_division(&self, left: &Expression, right: &Expression) -> EvaluatorResult<Value> {
        let left_result = self.evaluate_expression(left)?;
        let right_result = self.evaluate_expression(right)?;

        match (left_result, right_result) {
            (
                Value::Monomial {
                    coefficient: c1,
                    variable: v1,
                },
                Value::Monomial {
                    coefficient: c2,
                    variable: v2,
                },
            ) => match (v1, v2) {
                (Option::None, Option::None) => Ok(Value::Monomial {
                    coefficient: c1 / c2,
                    variable: None,
                }),
                (_, Some(_)) => Err(EvaluatorError {
                    error_type: EvaluatorErrorType::VariableDivision {
                        numerator: left.token.clone(),
                        denominator: right.token.clone(),
                    },
                    token: left.token.clone(),
                }),
                (Some(v), Option::None) => Ok(Value::Monomial {
                    coefficient: c1 / c2,
                    variable: Some(v.clone()),
                }),
            },
            (value_a, value_b) => {
                let left_values = match value_a {
                    Value::Sum(sum) => sum,
                    Value::Monomial {
                        coefficient,
                        variable,
                    } => vec![Value::Monomial {
                        coefficient,
                        variable,
                    }],
                };

                let right_values = match value_b {
                    Value::Sum(sum) => sum,
                    Value::Monomial {
                        coefficient,
                        variable,
                    } => vec![Value::Monomial {
                        coefficient,
                        variable,
                    }],
                };

                let values = self.evaluate_division_values(
                    &left_values,
                    &right_values,
                    &left.token,
                    &right.token,
                )?;

                Ok(Value::Sum(values))
            }
        }
    }

    fn evaluate_division_values(
        &self,
        left: &[Value],
        right: &[Value],
        left_token: &Token,
        right_token: &Token,
    ) -> EvaluatorResult<Vec<Value>> {
        let mut result = Vec::new();
        for left_value in left {
            for right_value in right {
                let mult_result = match (left_value, right_value) {
                    (
                        Value::Monomial {
                            coefficient: c1,
                            variable: v1,
                        },
                        Value::Monomial {
                            coefficient: c2,
                            variable: v2,
                        },
                    ) => match (v1, v2) {
                        (Option::None, Option::None) => Ok(Value::Monomial {
                            coefficient: c1 / c2,
                            variable: None,
                        }),
                        (Some(v), Option::None) | (Option::None, Some(v)) => Ok(Value::Monomial {
                            coefficient: c1 / c2,
                            variable: Some(v.clone()),
                        }),
                        (Some(_), Some(_)) => Err(EvaluatorError {
                            error_type: EvaluatorErrorType::VariableMultiplication {
                                left: left_token.clone(),
                                right: right_token.clone(),
                            },
                            token: left_token.clone(),
                        }),
                    },
                    (
                        Value::Monomial {
                            coefficient,
                            variable,
                        },
                        Value::Sum(list),
                    )
                    | (
                        Value::Sum(list),
                        Value::Monomial {
                            coefficient,
                            variable,
                        },
                    ) => {
                        let result = self.evaluate_multiplication_values(
                            &[Value::Monomial {
                                coefficient: *coefficient,
                                variable: variable.clone(),
                            }],
                            list,
                            left_token,
                            right_token,
                        )?;

                        Ok(Value::Sum(result))
                    }
                    (Value::Sum(sum1), Value::Sum(sum2)) => {
                        let result = self.evaluate_multiplication_values(
                            sum1,
                            sum2,
                            left_token,
                            right_token,
                        )?;

                        Ok(Value::Sum(result))
                    }
                };
                result.push(mult_result?);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluator::value::Value;
    use crate::lexer::{Equation, Lexer};
    use crate::tokenizer::{Token, Tokenizer};
    use std::io::{BufReader, Cursor};

    #[cfg(test)]
    fn text_into_tokens(text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let cursor = Cursor::new(text);
        let reader = BufReader::new(cursor);

        let tokenizer = Tokenizer::new(reader);

        for token in tokenizer {
            match token {
                Ok(token) => tokens.push(token),
                Err(e) => panic!("No se pudo leer la expresión: {e}"),
            }
        }

        tokens
    }

    #[cfg(test)]
    fn equation_from_text(text: &str) -> Equation {
        let tokens = text_into_tokens(text);
        let mut lexer = Lexer::new(tokens);

        lexer.equation().unwrap()
    }

    #[test]
    fn test_negation() {
        let equation = equation_from_text("-x = -3");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        assert_eq!(left, Value::new_monomial(-1.0, String::from("x")));

        let right = evaluator.evaluate_expression(&equation.right).unwrap();
        assert_eq!(right, Value::new_constant(-3.0));
    }

    #[test]
    fn test_sum() {
        let tokens = text_into_tokens("x + 2 = y - 3");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let evaluator = super::Evaluator {};

        let left_result = evaluator.evaluate_expression(&equation.left).unwrap();
        let right_result = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(
            left_result,
            Value::Sum(vec![
                Value::Monomial {
                    variable: Some(String::from("x")),
                    coefficient: 1.0,
                },
                Value::Monomial {
                    variable: None,
                    coefficient: 2.0,
                }
            ])
        );

        assert_eq!(
            right_result,
            Value::Sum(vec![
                Value::new_monomial(1.0, String::from("y")),
                Value::new_constant(-3.0)
            ])
        );

        let tokens = text_into_tokens("-2 - x = 3 + y");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let evaluator = super::Evaluator {};

        let left_result = evaluator.evaluate_expression(&equation.left).unwrap();
        let right_result = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(
            left_result,
            Value::Sum(vec![
                Value::Monomial {
                    variable: None,
                    coefficient: -2.0,
                },
                Value::Monomial {
                    variable: Some(String::from("x")),
                    coefficient: -1.0,
                },
            ])
        );

        assert_eq!(
            right_result,
            Value::Sum(vec![
                Value::new_constant(3.0),
                Value::new_monomial(1.0, String::from("y")),
            ])
        )
    }

    #[test]
    fn test_substraction() {
        let equation = equation_from_text("5 + y -x = 2y - 10");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(
            left,
            Value::Sum(vec![
                Value::new_constant(5.0),
                Value::new_monomial(1.0, String::from("y")),
                Value::new_monomial(-1.0, String::from("x")),
            ])
        );

        assert_eq!(
            right,
            Value::Sum(vec![
                Value::new_monomial(2.0, String::from("y")),
                Value::new_constant(-10.0),
            ])
        )
    }

    #[test]
    fn test_implicit_multiplication() {
        let equation = equation_from_text("3x = -6y");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_monomial(3.0, String::from("x")));
        assert_eq!(right, Value::new_monomial(-6.0, String::from("y")));
    }

    #[test]
    fn test_explicit_multiplication() {
        let equation = equation_from_text("3*x = -6*y");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_monomial(3.0, String::from("x")));
        assert_eq!(right, Value::new_monomial(-6.0, String::from("y")));

        let equation = equation_from_text("3*x*2 = -6*3*2y");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_monomial(3.0 * 2.0, String::from("x")));
        assert_eq!(
            right,
            Value::new_monomial(-6.0 * 3.0 * 2.0, String::from("y"))
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_multiplication() {
        let equation = equation_from_text("3*x*y = -6*y*z");
        let evaluator = super::Evaluator {};

        evaluator.evaluate_expression(&equation.left).unwrap();
        evaluator.evaluate_expression(&equation.right).unwrap();
    }

    #[test]
    fn test_single_times_group_multiplication() {
        let equation = equation_from_text("3*(x+1) = -6x*(3 + 2)");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(
            left,
            Value::Sum(vec![
                Value::new_monomial(3.0, String::from("x")),
                Value::new_constant(3.0)
            ])
        );

        assert_eq!(right, Value::new_monomial(-30.0, String::from("x")));

        let equation = equation_from_text("3*(x+1+y) = 3 + 2x - y");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(
            left,
            Value::Sum(vec![
                Value::new_monomial(3.0, String::from("x")),
                Value::new_constant(3.0),
                Value::new_monomial(3.0, String::from("y")),
            ])
        );

        assert_eq!(
            right,
            Value::Sum(vec![
                Value::new_constant(3.0),
                Value::new_monomial(2.0, String::from("x")),
                Value::new_monomial(-1.0, String::from("y"))
            ])
        )
    }

    #[test]
    #[should_panic]
    fn test_invalid_group_multiplication_left() {
        let equation = equation_from_text("(3 + x)*(1 - y) = (x - 2)*(-y+3)");
        let evaluator = super::Evaluator {};

        evaluator.evaluate_expression(&equation.left).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_group_multiplication_right() {
        let equation = equation_from_text("2 = (x - 2)*(-x+3)");
        let evaluator = super::Evaluator {};

        evaluator.evaluate_expression(&equation.right).unwrap();
    }

    #[test]
    fn evaluate_number_division() {
        let equation = equation_from_text("1 / 2 = 6 / 3");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_constant(0.5));
        assert_eq!(right, Value::new_constant(2.0));

        let equation = equation_from_text("0.5 / 0.5 = 4 / 0.25");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_constant(1.0));
        assert_eq!(right, Value::new_constant(16.0));
    }

    #[test]
    fn test_simple_variable_to_constant_division() {
        let equation = equation_from_text("(1/4) * x = y/2");
        let evaluator = super::Evaluator {};

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_monomial(0.25, String::from("x")));
        assert_eq!(right, Value::new_monomial(0.5, String::from("y")));
    }

    #[test]
    #[should_panic]
    fn panics_on_division_with_variable_left() {
        let equation = equation_from_text("y/x = 2");
        let evaluator = super::Evaluator {};

        evaluator.evaluate_expression(&equation.left).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic_on_division_variable_right() {
        let equation = equation_from_text("1.2x = 6 / y");
        let evaluator = super::Evaluator {};

        evaluator.evaluate_expression(&equation.right).unwrap();
    }

    #[test]
    fn test_group_implicit_multiplication() {
        let equation = equation_from_text("x(1 + 18) = (3 + 6)(2 + 9x)");
        let evaluator = super::Evaluator();

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_monomial(19.0, String::from("x")));
        assert_eq!(
            right,
            Value::Sum(vec![
                Value::new_constant(18.0),
                Value::new_monomial(81.0, String::from("x"))
            ])
        );
    }

    #[test]
    fn test_exponentiation() {
        let equation = equation_from_text("9^2 = 9^(1/2)");
        let evaluator = super::Evaluator();

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_constant(81.0));
        assert_eq!(right, Value::new_constant(3.0));
    }

    #[test]
    fn test_allows_exponent_to_sum_of_constants() {
        let equation = equation_from_text("2^(1 + 3 + 1) = 3^(2 + 1)");
        let evaluator = super::Evaluator();

        let left = evaluator.evaluate_expression(&equation.left).unwrap();
        let right = evaluator.evaluate_expression(&equation.right).unwrap();

        assert_eq!(left, Value::new_constant(32.0));
        assert_eq!(right, Value::new_constant(27.0));
    }

    #[test]
    #[should_panic]
    fn test_panics_on_variable_to_exponent() {
        let equation = equation_from_text("x^2 = y");
        let evaluator = super::Evaluator();

        evaluator.evaluate_expression(&equation.left).unwrap();
    }
}
