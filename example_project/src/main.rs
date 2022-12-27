use pyo3_pylogger;
use pyo3::prelude::*;

fn main() {
    pyo3_pylogger::register();
    env_logger::Builder::from_env(env_logger::Env::default()
    .default_filter_or("trace"))
    .init();

    	// Ask pyo3 to set up embedded Python interpreter
	pyo3::prepare_freethreaded_python();

	Python::with_gil(|py|  {
		// Python code can now `import logging` as usual
		py.run("import logging", None, None).unwrap();

		// Log messages are forwarded to `tracing` and dealt with by the subscriber
		py.run("logging.error('Something bad happened')", None, None).unwrap();

	});
}
