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
    //just show the logger working from Rust.
    info!("Just some normal information!");
    warn!("Something spooky happened!");

    // Ask pyo3 to set up embedded Python interpreter
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // Python code can now `import logging` as usual
        py.run("import logging", None, None).unwrap();
        // The root python logger will be set to "WARNING" by default

        py.run("logging.getLogger().setLevel(0)", None, None).unwrap();
        
        // Log messages are forwarded to `log` and dealt with by the subscriber
        py.run("logging.debug('DEBUG')", None, None).unwrap();
        py.run("logging.info('INFO')", None, None).unwrap();
        py.run("logging.warning('WARNING')", None, None).unwrap();
        py.run("logging.error('ERROR')", None, None).unwrap();
        py.run("logging.critical('CRITICAL')", None, None).unwrap();
    });
}

```

## Outputs

```bash
[2023-03-06T20:14:15Z INFO  example_project] Just some normal information!
[2023-03-06T20:14:15Z WARN  example_project] Something spooky happened!
[2023-03-06T20:14:15Z DEBUG example_application_py_logger] DEBUG
[2023-03-06T20:14:15Z INFO  example_application_py_logger] INFO
[2023-03-06T20:14:15Z WARN  example_application_py_logger] WARNING
[2023-03-06T20:14:15Z ERROR example_application_py_logger] ERROR
[2023-03-06T20:14:15Z ERROR example_application_py_logger] CRITICAL
```
