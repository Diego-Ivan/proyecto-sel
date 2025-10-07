#[pyo3::pymodule]
mod sel_simplifier {
    use pyo3::{exceptions::PyRuntimeError, prelude::*};
    use std::collections::HashMap;
    use syntax::Simplifier;

    #[pyclass(str)]
    pub struct CanonEquation {
        terms: HashMap<String, f64>,
        constant: f64,
    }

    #[pymethods]
    impl CanonEquation {
        #[getter]
        pub fn terms(&self) -> &HashMap<String, f64> {
            &self.terms
        }

        #[getter]
        pub fn constant(&self) -> f64 {
            self.constant
        }
    }

    impl CanonEquation {
        pub fn new(terms: HashMap<String, f64>, constant: f64) -> Self {
            Self { terms, constant }
        }
    }

    const CMP_EPSILON: f64 = 1e-20;

    impl std::fmt::Display for CanonEquation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut terms = self.terms.iter();

            match terms.next() {
                Some((var, coeff)) => write!(f, "{coeff}{var}"),
                None => write!(f, "0"),
            }?;

            for (variable, coeff) in terms {
                if coeff.abs() < CMP_EPSILON {
                    continue;
                }

                let sign = if *coeff > 0.0 { '+' } else { '-' };

                write!(f, " {sign} {variable}{coeff}")?;
            }

            write!(f, " = {}", self.constant)
        }
    }

    #[pyfunction]
    pub fn simplify_expression(input: &str) -> PyResult<CanonEquation> {
        let simplifier = Simplifier();
        let simplified_equation = simplifier.simplify_equation(input);

        match simplified_equation {
            Ok(eq) => Ok(CanonEquation::new(eq.terms, eq.constant)),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
    }
}
