use log::{logger, Level, MetadataBuilder, Record};
use pyo3::prelude::*;

/// Convenience function to register the rust logger with the Python logging instance.
pub fn register() {
    Python::with_gil(|py| {
        // Extend the `logging` module to interact with log
        setup_logging(py)
    }).unwrap();
}


/// Consume a Python `logging.LogRecord` and emit a Rust `Log` instead.
#[pyfunction]
fn host_log(record: &PyAny) -> PyResult<()> {
    let level = record.getattr("levelno")?;
    let message = record.getattr("getMessage")?.call0()?.to_string();
    let pathname = record.getattr("pathname")?.to_string();
    let lineno = record
        .getattr("lineno")?
        .to_string()
        .parse::<u32>()
        .unwrap();
    let _logger_name = record.getattr("name")?.to_string();

    // error
    let error_metadata = if level.ge(40u8)? {
        MetadataBuilder::new()
            .target("angreal")
            .level(Level::Error)
            .build()
    } else if level.ge(30u8)? {
        MetadataBuilder::new()
            .target("angreal")
            .level(Level::Warn)
            .build()
    } else if level.ge(20u8)? {
        MetadataBuilder::new()
            .target("angreal")
            .level(Level::Info)
            .build()
    } else if level.ge(10u8)? {
        MetadataBuilder::new()
            .target("angreal")
            .level(Level::Debug)
            .build()
    } else {
        MetadataBuilder::new()
            .target("angreal")
            .level(Level::Trace)
            .build()
    };

    logger().log(
        &Record::builder()
            .metadata(error_metadata)
            .args(format_args!("{}", &message))
            .line(Some(lineno))
            .file(Some("app.rs"))
            .module_path(Some(&pathname))
            .build(),
    );

    Ok(())
}

/// Registers the host_log function in rust as the event handler for Python's logging logger
/// This function needs to be called from within a pyo3 context as early as possible to ensure logging messages 
/// arrive to the rust consumer. 
pub fn setup_logging(py: Python) -> PyResult<()> {
    let logging = py.import("logging")?;

    logging.setattr("host_log", wrap_pyfunction!(host_log, logging)?)?;

    py.run(
        r#"
class HostHandler(Handler):
	def __init__(self, level=0):
		super().__init__(level=level)
	
	def emit(self, record):
		host_log(record)

oldBasicConfig = basicConfig
def basicConfig(*pargs, **kwargs):
	if "handlers" not in kwargs:
		kwargs["handlers"] = [HostHandler()]
	return oldBasicConfig(*pargs, **kwargs)
"#,
        Some(logging.dict()),
        None,
    )?;

    let all = logging.index()?;
    all.append("HostHandler")?;

    Ok(())
}


