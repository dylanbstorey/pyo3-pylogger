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
        // Log messages are forwarded to `log` and dealt with by the subscriber
        py.run("logging.error('Something bad happened')", None, None)
            .unwrap();
    });
}
