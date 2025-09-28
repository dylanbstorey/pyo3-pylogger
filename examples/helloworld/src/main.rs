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
    Python::attach(|py| {
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
    }).unwrap()
}
