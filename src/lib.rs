use log::{logger, Level, MetadataBuilder, Record};
use pyo3::prelude::*;

/// Convenience function to register the rust logger with the Python logging instance.
pub fn register(target: &str) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // Extend the `logging` module to interact with log
        setup_logging(py, target)
    })
    .unwrap();
}

/// Consume a Python `logging.LogRecord` and emit a Rust `Log` instead.
#[pyfunction]
fn host_log(record: Bound<'_, PyAny>, rust_target: &str) -> PyResult<()> {
    let level = record.getattr("levelno")?;
    let message = record.getattr("getMessage")?.call0()?.to_string();
    let pathname = record.getattr("pathname")?.to_string();
    let lineno = record
        .getattr("lineno")?
        .to_string()
        .parse::<u32>()
        .unwrap();

    let logger_name = record.getattr("name")?.to_string();

    let full_target: Option<String> = if logger_name.trim().is_empty() || logger_name == "root" {
        None
    } else {
        // Libraries (ex: tracing_subscriber::filter::Directive) expect rust-style targets like foo::bar,
        // and may not deal well with "." as a module separator:
        let logger_name = logger_name.replace('.', "::");
        Some(format!("{rust_target}::{logger_name}"))
    };
    let target = full_target.as_deref().unwrap_or(rust_target);

    let mut metadata_builder = MetadataBuilder::new();
    metadata_builder.target(target);
    if level.ge(40u8)? {
        metadata_builder.level(Level::Error)
    } else if level.ge(30u8)? {
        metadata_builder.level(Level::Warn)
    } else if level.ge(20u8)? {
        metadata_builder.level(Level::Info)
    } else if level.ge(10u8)? {
        metadata_builder.level(Level::Debug)
    } else {
        metadata_builder.level(Level::Trace)
    };

    logger().log(
        &Record::builder()
            .metadata(metadata_builder.build())
            .args(format_args!("{}", &message))
            .line(Some(lineno))
            .file(Some(&pathname))
            .module_path(Some(&pathname))
            .build(),
    );

    Ok(())
}

/// Registers the host_log function in rust as the event handler for Python's logging logger
/// This function needs to be called from within a pyo3 context as early as possible to ensure logging messages
/// arrive to the rust consumer.
pub fn setup_logging(py: Python, target: &str) -> PyResult<()> {
    let logging = py.import_bound("logging")?;

    logging.setattr("host_log", wrap_pyfunction!(host_log, &logging)?)?;

    py.run_bound(
        format!(
            r#"
class HostHandler(Handler):
	def __init__(self, level=0):
		super().__init__(level=level)

	def emit(self, record):
		host_log(record,"{}")

oldBasicConfig = basicConfig
def basicConfig(*pargs, **kwargs):
	if "handlers" not in kwargs:
		kwargs["handlers"] = [HostHandler()]
	return oldBasicConfig(*pargs, **kwargs)
"#,
            target
        )
        .as_str(),
        Some(&logging.dict()),
        None,
    )?;

    let all = logging.index()?;
    all.append("HostHandler")?;

    Ok(())
}
