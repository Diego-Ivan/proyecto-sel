# Proyecto SEL

Este proyecto es un analizador (parser) de ecuaciones que las simplifica a una ecuación lineal canónica, si es posible. Ofrece soporte para:

* Sumas y restas.
* Multiplicaciones y divisiones.
* Exponenciación (por ejemplo, `2^10`).
* Multiplicación implícita, como en `2(9)x`.
* Uso de ciertas funciones definidas, como `sqrt`, `ln`, entre otras, utilizando la sintaxis `\NOMBREFUNCION`.

Incluye enlaces (bindings) para Python mediante `pyo3`.

## Instrucciones de instalación

### Python

Puede instalar este proyecto como un paquete de Python utilizando el siguiente comando:

```sh
pip install syntax-pyo3@git+https://github.com/Diego-Ivan/proyecto-sel#subdirectory=syntax-pyo3
```

Luego puede usarlo en un script de Python de la siguiente manera:

```python
from sel_simplifier import simplify_expression

expr = simplify_expression("2x + 5y = -12 + 3x -9(y - 5)")

print(expr.terms)
print(expr.constant)
```
