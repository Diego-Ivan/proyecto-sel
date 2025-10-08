mod error;

use crate::expression::{Expression, ExpressionType};
use crate::lexer::error::LexerResult;
use crate::tokenizer::{Token, TokenType};

pub use crate::lexer::error::LexerError;

pub struct Lexer {
    tokens: Vec<Token>,
    current: usize,
}

pub struct Equation {
    pub left: Expression,
    pub right: Expression,
}

macro_rules! match_token {
    ($parser: ident, $pattern: pat) => {{
        match $parser.peek() {
            Some(next_token) => {
                if matches!(next_token.token_type, $pattern) {
                    $parser.advance();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }};
}

macro_rules! expect_token {
    ($parser: ident, $pattern: pat, $token_type: ident) => {{
        match $parser.peek() {
            Some(next_token) if matches!(next_token.token_type, $pattern) => {
                $parser.advance();
            }
            Some(next_token) => {
                return Err(LexerError::WrongToken {
                    expected: TokenType::$token_type,
                    found: next_token.token_type.clone(),
                });
            }
            None => {
                return Err(LexerError::ExpectedTokenFoundEof {
                    expected: TokenType::$token_type,
                });
            }
        }
    }};
}

impl Lexer {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn equation(&mut self) -> LexerResult<Equation> {
        let left = self.expression()?;

        expect_token!(self, TokenType::Equal, Equal);

        let right = self.expression()?;

        if self.current < self.tokens.len() {
            return Err(LexerError::ExpectedEof {
                found: self.peek().unwrap().token_type.clone(),
            });
        }

        Ok(Equation { left, right })
    }

    fn expression(&mut self) -> LexerResult<Expression> {
        let mut expression = self.factor()?;

        while match_token!(self, TokenType::Plus | TokenType::Minus) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => break,
            };

            let right = self.factor()?;

            expression = Expression {
                expression_type: ExpressionType::Binary {
                    left: Box::new(expression),
                    right: Box::new(right),
                    operator: operator.clone(),
                },
                token: operator.clone(),
            }
        }

        Ok(expression)
    }

    fn factor(&mut self) -> LexerResult<Expression> {
        let mut factor = self.monomial()?;

        while match_token!(self, TokenType::Star | TokenType::Slash) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => break,
            };
            let monomial = self.monomial()?;

            factor = Expression {
                expression_type: ExpressionType::Binary {
                    operator: operator.clone(),
                    left: Box::new(factor),
                    right: Box::new(monomial),
                },
                token: operator,
            }
        }

        Ok(factor)
    }

    fn monomial(&mut self) -> LexerResult<Expression> {
        if match_token!(self, TokenType::Minus) {
            let right = self.monomial()?;
            return Ok(Expression {
                expression_type: ExpressionType::Negation(Box::new(right)),
                token: self.previous().unwrap().clone(),
            });
        }

        let mut primary = self.primary()?;

        let next = match self.peek() {
            Some(next) => next.clone(),
            None => return Ok(primary),
        };

        match &next.token_type {
            TokenType::Identifier(_) => {
                let right = self.monomial()?;
                primary = Expression {
                    expression_type: ExpressionType::Binary {
                        left: Box::new(primary),
                        right: Box::new(right),
                        operator: Token::new(TokenType::Star, String::from("*"), next.column),
                    },
                    token: next.clone(),
                }
            }
            TokenType::LeftParen => {
                let right = self.monomial()?;
                primary = Expression {
                    expression_type: ExpressionType::Binary {
                        left: Box::new(primary),
                        operator: Token::new(TokenType::Star, String::from("*"), next.column),
                        right: Box::new(right),
                    },
                    token: next.clone(),
                }
            }
            TokenType::Hat => {
                self.advance();
                let exponent = self.parse_exponent()?;

                primary = Expression {
                    expression_type: ExpressionType::Binary {
                        left: Box::new(primary),
                        operator: next.clone(),
                        right: Box::new(exponent),
                    },
                    token: next.clone(),
                }
            }
            _ => {}
        }

        Ok(primary)
    }

    fn primary(&mut self) -> LexerResult<Expression> {
        let token = match self.peek() {
            Some(token) => token.clone(),
            None => return Err(LexerError::UnexpectedEof),
        };

        match &token.token_type {
            TokenType::Number(num) => {
                self.advance();
                Ok(Expression {
                    expression_type: ExpressionType::Number(*num),
                    token,
                })
            }
            TokenType::LeftParen => {
                self.advance();
                self.parse_group(token)
            }
            TokenType::Identifier(varname) => {
                self.advance();
                Ok(Expression {
                    expression_type: ExpressionType::Variable(varname.clone()),
                    token,
                })
            }
            other => Err(LexerError::ExpectedPrimary {
                found: other.clone(),
            }),
        }
    }

    fn parse_exponent(&mut self) -> LexerResult<Expression> {
        let next = match self.peek() {
            Some(next) => next.clone(),
            None => return Err(LexerError::UnexpectedEof),
        };

        self.primary().map_err(|_| LexerError::InvalidExponent {
            found: next.token_type,
        })
    }

    fn parse_group(&mut self, token: Token) -> LexerResult<Expression> {
        let group = self.expression()?;
        expect_token!(self, TokenType::RightParen, RightParen);

        Ok(Expression {
            expression_type: ExpressionType::Grouping(Box::new(group)),
            token: token.clone(),
        })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.current >= self.tokens.len() {
            None
        } else {
            let current = &self.tokens[self.current];
            self.current += 1;
            Some(current)
        }
    }

    fn previous(&mut self) -> Option<&Token> {
        if self.current == 0 {
            None
        } else {
            self.tokens.get(self.current - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::ExpressionType;
    use crate::lexer::Lexer;
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
    #[test]
    fn test_uniques() {
        let tokens = text_into_tokens("3 = 3");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        assert_eq!(equation.left.expression_type, ExpressionType::Number(3.0));
        assert_eq!(equation.right.expression_type, ExpressionType::Number(3.0));
    }

    #[test]
    fn test_negation() {
        let tokens = text_into_tokens("-3 = -10");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        assert_eq!(left, "(- 3)");

        let right = format!("{}", equation.right);
        assert_eq!(right, "(- 10)");
    }

    #[test]
    fn test_variable() {
        let tokens = text_into_tokens("x = y");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        assert_eq!(left, "x");

        let right = format!("{}", equation.right);
        assert_eq!(right, "y");
    }

    #[test]
    fn test_binary_operations() {
        let tokens = text_into_tokens("x + 2 = y + x- 3");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        assert_eq!(left, "(+ x 2)");

        let right = format!("{}", equation.right);
        assert_eq!(right, "(- (+ y x) 3)");

        let tokens = text_into_tokens("23 * x = 19 / y");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        assert_eq!(left, "(* 23 x)");

        let right = format!("{}", equation.right);
        assert_eq!(right, "(/ 19 y)");
    }

    #[test]
    fn test_grouping() {
        let tokens = text_into_tokens("(x + 1) = (y * 2)");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();
        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(group (+ x 1))");
        assert_eq!(right, "(group (* y 2))");
    }

    #[test]
    fn test_implicit_multiplication() {
        let tokens = text_into_tokens("3x = 6y");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();
        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(* 3 x)");
        assert_eq!(right, "(* 6 y)");

        let tokens = text_into_tokens("(-3 + y)x = (z - 6)x");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();
        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(* (group (+ (- 3) y)) x)");
        assert_eq!(right, "(* (group (- z 6)) x)");
    }

    #[test]
    fn test_invalid_implicit_multiplication_right() {
        let tokens = text_into_tokens("x = 2x(1 + y)");
        let mut lexer = Lexer::new(tokens);
        let right = lexer.equation().unwrap().right;

        assert_eq!(format!("{right}"), "(* 2 (* x (group (+ 1 y))))");
    }

    #[test]
    fn test_group_implicit_multiplication() {
        let tokens = text_into_tokens("x(1 + y) = (3 + 6)(2 + x)");
        let mut lexer = Lexer::new(tokens);
        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(* x (group (+ 1 y)))");
        assert_eq!(right, "(* (group (+ 3 6)) (group (+ 2 x)))");
    }

    #[test]
    fn test_triple_group_implicit_multiplication() {
        let tokens = text_into_tokens("(1 + 6) (x + 9) (y - 2) = (1 + 6)*(x + 9)*(y - 2)");
        let mut lexer = Lexer::new(tokens);

        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(
            left,
            "(* (group (+ 1 6)) (* (group (+ x 9)) (group (- y 2))))"
        );

        assert_eq!(
            right,
            "(* (* (group (+ 1 6)) (group (+ x 9))) (group (- y 2)))"
        );
    }

    #[test]
    fn test_group_times_variable_multiplication() {
        let tokens = text_into_tokens("(1 + y)x = (-9 + x)y");

        let mut lexer = Lexer::new(tokens);
        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(* (group (+ 1 y)) x)");
        assert_eq!(right, "(* (group (+ (- 9) x)) y)")
    }

    #[test]
    fn test_exponent_to_numbers() {
        let tokens = text_into_tokens("9^16 = -12.25^2.5");

        let mut lexer = Lexer::new(tokens);
        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(^ 9 16)");
        assert_eq!(right, "(- (^ 12.25 2.5))")
    }

    #[test]
    fn test_exponent_to_identifiers() {
        let tokens = text_into_tokens("12^x = -2.5^y");

        let mut lexer = Lexer::new(tokens);
        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(^ 12 x)");
        assert_eq!(right, "(- (^ 2.5 y))")
    }

    #[test]
    fn test_exponent_to_groupings() {
        let tokens = text_into_tokens("x^(9 + 7 - y) = y^(7 + x)");

        let mut lexer = Lexer::new(tokens);
        let equation = lexer.equation().unwrap();

        let left = format!("{}", equation.left);
        let right = format!("{}", equation.right);

        assert_eq!(left, "(^ x (group (- (+ 9 7) y)))");
        assert_eq!(right, "(^ y (group (+ 7 x)))");
    }

    #[test]
    #[should_panic]
    fn test_panics_on_invalid_exponent() {
        let tokens = text_into_tokens("x^* = y");

        let mut lexer = Lexer::new(tokens);
        lexer.equation().unwrap();
    }
}
