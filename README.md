# pyo3-pylogger

Enables log messages for pyo3 embedded Python applications using Python's `logging` or module.

# Features
- Logging integration between Python's `logging` module and Rust's `log` crate
- Structured logging support via the logging [extra](https://docs.python.org/3/library/logging.html#logging.Logger.debug) field (requires `kv` or `tracing-kv`feature)
- Integration with Rust's `tracing` library (requires `tracing` feature)

# Usage
```rust
use log::{info, warn};
use pyo3::{ffi::c_str, prelude::*};
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
        py.run(
            c_str!(
                r#"
import logging
logging.getLogger().setLevel(0)
logging.debug('DEBUG')
logging.info('INFO')
logging.warning('WARNING')
logging.error('ERROR')
logging.getLogger('foo.bar.baz').info('INFO')"#
            ),
            None,
            None,
        )
        .unwrap();
    })
}


```

## Outputs

```bash
[2025-03-28T01:12:29Z INFO  helloworld] Just some normal information!
[2025-03-28T01:12:29Z WARN  helloworld] Something spooky happened!
[2025-03-28T01:12:29Z DEBUG example_application_py_logger] DEBUG
[2025-03-28T01:12:29Z INFO  example_application_py_logger] INFO
[2025-03-28T01:12:29Z WARN  example_application_py_logger] WARNING
[2025-03-28T01:12:29Z ERROR example_application_py_logger] ERROR
[2025-03-28T01:12:29Z INFO  example_application_py_logger::foo::bar::baz] INFO
```

## Structured Logging

To enable structured logging support, add the `kv` feature to your `Cargo.toml`:

```toml
[dependencies]
pyo3-pylogger = { version = "0.3", features = ["kv"] }
```

Then you can use Python's `extra` parameter to pass structured data:

```python
logging.info("Processing order", extra={"order_id": "12345", "amount": 99.99})
```

When using a structured logging subscriber in Rust, these key-value pairs will be properly captured, for example:

```bash
[2025-03-28T01:12:29Z INFO  example_application_py_logger] Processing order order_id=12345 amount=99.99
```
## Tracing Support

To enable integration with Rust's `tracing` library, add the `tracing` feature to your `Cargo.toml`:

```toml
[dependencies]
pyo3-pylogger = { version = "0.3", default-features = false, features = ["tracing"] }
```

When the `tracing` feature is enabled, Python logs will be forwarded to the active tracing subscriber:

```rust
use tracing::{info, warn};
use pyo3::{ffi::c_str, prelude::*};

fn main() {
    // Register the tracing handler with Python logger
    pyo3_pylogger::register_tracing("example_application_py_logger");

    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    
    // Tracing events from Rust
    info!("Tracing information from Rust");
    
    // Python logging will be captured by the tracing subscriber
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        py.run(
            c_str!(
                r#"
import logging
logging.getLogger().setLevel(0)
logging.info('This will be captured by tracing')"#
            ),
            None,
            None,
        )
        .unwrap();
    })
}
```

### Structured Data with Tracing

The `tracing` feature automatically supports Python's `extra` field for structured data. However, the KV fields are json serialized and not available as tracing attributes. This is a limitation of the `tracing` library and is not specific to this crate. See [this issue](https://github.com/tokio-rs/tracing/issues/372) for more information.

# Feature Flags

- `kv`: Enables structured logging support via Python's `extra` fields. This adds support for the `log` crate's key-value system.
- `tracing`: Enables integration with Rust's `tracing` library.
- `tracing-kv`: Enables structured logging support via Python's `extra` fields and integration with Rust's `tracing` library.

