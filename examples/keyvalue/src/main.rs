use log::{info, warn};
use pyo3::{ffi::c_str, prelude::*};

fn main() {
    // register the host handler with python logger, providing a logger target
    pyo3_pylogger::register("example_application_py_logger");

    // initialize up a logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .format_timestamp(None)
        .init();

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
logging.debug('DEBUG', extra={'some_key': 'some_value'})
logging.info('INFO', extra={'some_dict': {'a': 'b', 'c': 'd'}})
logging.warning('WARNING', extra={'some_list': ['a', 'b', 'c']})
logging.error('ERROR', extra={'some_int': 42})
logging.critical('CRITICAL', extra={'some_float': 3.14, 4: 'four'})
            "#
            ),
            None,
            None,
        )
    })
    .unwrap()
}
