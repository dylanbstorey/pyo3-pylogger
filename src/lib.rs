use std::ffi::CString;

use pyo3::prelude::*;

#[cfg(feature = "kv")]
mod kv;

mod level;

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

    // If log feature is enabled, use log::logger

    #[cfg(feature = "log")]
    {
        let mut metadata_builder = log::MetadataBuilder::new();
        metadata_builder.target(target);

        let level = crate::level::get_level(level.extract()?);

        metadata_builder.level(level.0);

        let mut record_builder = log::Record::builder();

        #[cfg(feature = "kv")]
        {
            let kv_args = kv::find_kv_args(&record)?;

            let kv_source = kv_args.map(kv::KVSource);
            if let Some(kv_source) = kv_source {
                log::logger().log(
                    &record_builder
                        .metadata(metadata_builder.build())
                        .args(format_args!("{}", &message))
                        .line(Some(lineno))
                        .file(Some(&pathname))
                        .module_path(Some(&pathname))
                        .key_values(&kv_source)
                        .build(),
                );
                return Ok(());
            }
        }

        log::logger().log(
            &record_builder
                .metadata(metadata_builder.build())
                .args(format_args!("{}", &message))
                .line(Some(lineno))
                .file(Some(&pathname))
                .module_path(Some(&pathname))
                .build(),
        );
    }

    #[cfg(feature = "tracing")]
    {
        // Create a new Event with the given level
        let level = crate::level::get_level(level.extract()?).0;

        #[cfg(feature = "kv")]
        {
            let kv_args = kv::find_kv_args(&record)?;

            let fields: std::collections::HashMap<String, Bound<'_, PyAny>> =
                kv_args.unwrap_or_default();

            // this is the only way to pass fields to tracing, unfortunatley
            // it's not possible as of mar 30 2025 to pass dynamic fields to tracing
            // see: https://github.com/tokio-rs/tracing/issues/372
            // Convert Python dictionary to a Rust value that can be serialized to JSON
            let mut json_map = serde_json::Map::new();

            // Convert each Python object in the HashMap to a JSON value
            for (key, value) in fields.into_iter() {
                let fallback = format!("{:?}", &value);
                match serde_pyobject::from_pyobject(value) {
                    Ok(json_value) => {
                        json_map.insert(key.clone(), json_value);
                    }
                    Err(e) => {
                        tracing::error!("Error converting Python object to JSON: {:?}", e);
                        // Handle conversion errors (optional)
                        // Supported types: https://github.com/Jij-Inc/serde-pyobject/blob/32c3ac77c2ed09b654f7fbc960c5f273fd1bb85c/src/de.rs#L305
                        json_map.insert(
                            key.clone(),
                            serde_json::Value::String(format!("{:?}", fallback)),
                        );
                    }
                }
            }

            // Create a JSON object from our map and convert to string
            let json_value = serde_json::Value::Object(json_map);
            let fields = serde_json::to_string(&json_value).unwrap_or_default();

            match level {
                tracing::Level::ERROR => {
                    tracing::error!(%target, %pathname, %lineno, python_fields = %fields, "{}", message )
                }
                tracing::Level::WARN => {
                    tracing::warn!(%target, %pathname, %lineno, python_fields = %fields, "{}", message )
                }
                tracing::Level::INFO => {
                    tracing::info!(%target, %pathname, %lineno, python_fields = %fields, "{}", message )
                }
                tracing::Level::DEBUG => {
                    tracing::debug!(%target, %pathname, %lineno, python_fields = %fields, "{}", message )
                }
                tracing::Level::TRACE => {
                    tracing::trace!(%target, %pathname, %lineno, python_fields = %fields, "{}", message )
                }
            }
        }
        #[cfg(not(feature = "kv"))]
        {
            match level {
                tracing::Level::ERROR => {
                    tracing::event!(tracing::Level::ERROR, %target, %pathname, %lineno, "{}", message)
                }
                tracing::Level::WARN => {
                    tracing::event!(tracing::Level::WARN, %target, %pathname, %lineno, "{}", message)
                }
                tracing::Level::INFO => {
                    tracing::event!(tracing::Level::INFO, %target, %pathname, %lineno, "{}", message)
                }
                tracing::Level::DEBUG => {
                    tracing::event!(tracing::Level::DEBUG, %target, %pathname, %lineno, "{}", message)
                }
                tracing::Level::TRACE => {
                    tracing::event!(tracing::Level::TRACE, %target, %pathname, %lineno, "{}", message)
                }
            }
        }
    }

    Ok(())
}

/// Registers the host_log function in rust as the event handler for Python's logging logger
/// This function needs to be called from within a pyo3 context as early as possible to ensure logging messages
/// arrive to the rust consumer.
pub fn setup_logging(py: Python, target: &str) -> PyResult<()> {
    let logging = py.import("logging")?;

    logging.setattr("host_log", wrap_pyfunction!(host_log, &logging)?)?;

    let code = CString::new(format!(
        r#"
class HostHandler(Handler):
	def __init__(self, level=0):
		super().__init__(level=level)

	def emit(self, record: LogRecord):
		host_log(record, "{}")

oldBasicConfig = basicConfig
def basicConfig(*pargs, **kwargs):
    if "handlers" not in kwargs:
        kwargs["handlers"] = [HostHandler()]
    return oldBasicConfig(*pargs, **kwargs)
"#,
        target
    ))?;

    py.run(&code, Some(&logging.dict()), None)?;

    let all = logging.index()?;
    all.append("HostHandler")?;

    Ok(())
}
