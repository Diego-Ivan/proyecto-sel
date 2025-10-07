use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Value {
    Sum(Vec<Value>),
    Monomial {
        coefficient: f64,
        variable: Option<String>,
    },
}

impl Value {
    pub fn new_constant(coefficient: f64) -> Value {
        Value::Monomial {
            coefficient,
            variable: None,
        }
    }

    pub fn new_monomial(coefficient: f64, variable: String) -> Value {
        Value::Monomial {
            coefficient,
            variable: Some(variable),
        }
    }

    pub fn negate(self) -> Self {
        match self {
            Self::Monomial {
                coefficient,
                variable,
            } => Self::Monomial {
                coefficient: -coefficient,
                variable,
            },
            Self::Sum(values) => {
                let values = values.into_iter().map(|v| v.negate()).collect();
                Self::Sum(values)
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Monomial {
                coefficient,
                variable,
            } => {
                let variable_fmt = match variable {
                    Some(var) => var,
                    None => "",
                };
                write!(f, "{coefficient}{variable_fmt}")
            }
            Self::Sum(values) => {
                let mut iter = values.iter();
                if let Some(first) = iter.next() {
                    write!(f, "{first}")?;
                }

                for value in iter {
                    write!(f, " + ({value})")?;
                }
                Ok(())
            }
        }
    }
}
