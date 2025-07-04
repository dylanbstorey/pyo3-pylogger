use pyo3::{ffi::c_str, prelude::*};
use tracing::metadata::LevelFilter;
use tracing::{info, warn};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
fn main() {
    // register the host handler with python logger, providing a logger target
    let standard_layer = tracing_logfmt::builder()
        .with_location(true)
        .with_target(true)
        .with_module_path(true)
        .with_span_name(true)
        .layer()
        // Only apply this layer if the target is NOT pyo3_logger
        .with_filter(filter_fn(|metadata| {
            metadata.target() != "pyo3_pylogger" && metadata.target() != "tracing"
        }));

    // Layer specifically for pyo3_logger targets - omits location and module_path
    let pyo3_layer = tracing_logfmt::builder()
        .with_location(false) // No location
        .with_target(false)
        .with_module_path(false) // No module path
        .with_span_name(true)
        .layer()
        // Only apply this layer if the target IS pyo3_logger
        .with_filter(filter_fn(|metadata| {
            metadata.target() == "pyo3_pylogger" || metadata.target() == "tracing"
        }));

    tracing_subscriber::registry()
        .with(LevelFilter::TRACE)
        .with(standard_layer)
        .with(pyo3_layer)
        .try_init()
        .unwrap();

    pyo3_pylogger::register("example_application_py_logger");

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
logging.critical('CRITICAL')

            "#
            ),
            None,
            None,
        )
        .unwrap();
    })
}
