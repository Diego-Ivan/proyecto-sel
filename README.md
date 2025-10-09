# Proyecto SEL.

This project is a parser for equations, and simplifies them into a canonical linear equation, if possible. It contains support for:

* Sums and substractions.
* Multiplications and divisions
* Exponentiation (e.g `2^10`)
* Implicit multiplication, like `2(9)x`.
* Using certain defined functions, like `sqrt`, `ln`, among others with the syntax `\FUNCTIONNAME`.

It includes Python bindings using `pyo3`.

## Installation Instructions.

### Python

You can install this project as a Python package using the following command:

```sh
pip install syntax-pyo3@git+https://github.com/Diego-Ivan/proyecto-sel#subdirectory=syntax-pyo3
```

Then you can use it in a Python script as follows:

```python
from sel_simplifier import simplify_expression

expr = simplify_expression("2x + 5y = -12 + 3x -9(y - 5)")

print(expr.terms)
print(expr.constant);
```
