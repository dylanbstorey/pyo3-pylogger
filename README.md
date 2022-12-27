# pyo3-pylogger

Enables log messages for pyo3 embedded Python applications using Python's `logging` module. 

# Usage

Within your pyo3 based Rust application:
```rust
use pyo3-pylogger;
pyo3-pylogger::register();
```

Within your python code: 
```python
import logging

logging.error("Something bad happened !")
```
