# pyo3-pylogger

Enables log messages for pyo3 embedded Python applications using Python's `logging` module. 

# Usage
```rust
use log::{info, warn};
use pyo3::prelude::*;

fn main() {
    // register the host handler with python logger, providing a logger target
    pyo3_pylogger::register("example_application_py_logger");

    // initialize up a logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    info!("Just some normal information!");
    //just show the logger working from Rust.
    warn!("Something spooky happened!");

    // Ask pyo3 to set up embedded Python interpreter
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // Python code can now `import logging` as usual
        py.run("import logging", None, None).unwrap();
        // Log messages are forwarded to `tracing` and dealt with by the subscriber
        py.run("logging.error('Something bad happened')", None, None)
            .unwrap();
    });
}```

## Outputs

```
[2022-12-27T15:26:12Z INFO  example_project] Just some normal information!

[2022-12-27T15:26:12Z WARN  example_project] Something spooky happened!

[2022-12-27T15:26:12Z ERROR example_application_py_logger] Something bad happened
```
