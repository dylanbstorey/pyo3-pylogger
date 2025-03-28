//! Key-Value handling module for Python LogRecord attributes.
//!
//! This module provides functionality to extract and handle custom key-value pairs
//! from Python LogRecord objects, facilitating integration between Python's logging
//! system and Rust's log crate.

use log::kv::Source;
use pyo3::{
    types::{PyAnyMethods, PyDict, PyDictMethods, PyListMethods},
    Bound, PyAny, PyResult,
};
use std::collections::HashMap;

/// A static hashset containing all standard [LogRecord](https://github.com/python/cpython/blob/8a00c9a4d2ce9d373b13f8f0a2265a65f4523293/Lib/logging/__init__.py#L286-L287) attributes defined in the CPython logging module.
///
/// This set is used to differentiate between standard [LogRecord](https://github.com/python/cpython/blob/8a00c9a4d2ce9d373b13f8f0a2265a65f4523293/Lib/logging/__init__.py#L286-L287) attributes and custom key-value pairs
/// that users might add to their log records. The attributes listed here correspond to the default
/// attributes created by Python's [makeRecord](https://github.com/python/cpython/blob/8a00c9a4d2ce9d373b13f8f0a2265a65f4523293/Lib/logging/__init__.py#L1633-L1634) function.
pub static LOG_RECORD_KV_ATTRIBUTES: phf::Set<&'static str> = phf::phf_set! {
    "name",
    "msg",
    "args",
    "levelname",
    "levelno",
    "pathname",
    "filename",
    "module",
    "exc_info",
    "exc_text",
    "stack_info",
    "lineno",
    "funcName",
    "created",
    "msecs",
    "relativeCreated",
    "thread",
    "threadName",
    "processName",
    "process",
    "taskName",
};

/// Extracts custom key-value pairs from a Python LogRecord object.
///
/// This function examines the `__dict__` of a LogRecord(https://github.com/python/cpython/blob/8a00c9a4d2ce9d373b13f8f0a2265a65f4523293/Lib/logging/__init__.py#L286-L287) object and identifies any attributes
/// that are not part of the standard [LogRecord](https://github.com/python/cpython/blob/8a00c9a4d2ce9d373b13f8f0a2265a65f4523293/Lib/logging/__init__.py#L286-L287) attributes. These custom attributes are
/// treated as key-value pairs for structured logging.
///
/// # Arguments
/// * `record` - A reference to a Python LogRecord object
///
/// # Returns
/// * `PyResult<Option<HashMap<String, pyo3::Bound<'a, pyo3::PyAny>>>>` - If custom attributes
///   are found, returns a HashMap containing the key-value pairs. Returns None if no custom
///   attributes are present.
///
/// # Note
/// This function relies on the fact that Python will not implement new attributes on the LogRecord object.
/// If new attributes are added, this function will not be able to filter them out and will return them as key-value pairs.
/// In that future, [LOG_RECORD_KV_ATTRIBUTES] will need to be updated to include the new attributes.
/// This is an unfortunate side effect of using the `__dict__` attribute to extract key-value pairs. However, there are no other ways to handle this given that CPython does not distinguish between user-provided attributes and attributes created by the logging module.
pub fn find_kv_args<'a>(
    record: &Bound<'a, PyAny>,
) -> PyResult<Option<std::collections::HashMap<String, pyo3::Bound<'a, pyo3::PyAny>>>> {
    let dict: Bound<'_, PyDict> = record.getattr("__dict__")?.extract()?;

    // We can abuse the fact that Python dictionaries are ordered by insertion order to reverse iterate over the keys
    // and stop at the first key that is not a predefined key-value pair attribute.
    let mut kv_args: Option<HashMap<String, pyo3::Bound<'_, pyo3::PyAny>>> = None;

    for item in dict.items().iter().rev() {
        let (key, value) =
            item.extract::<(pyo3::Bound<'_, pyo3::PyAny>, pyo3::Bound<'_, pyo3::PyAny>)>()?;

        let key_str = key.to_string();
        if LOG_RECORD_KV_ATTRIBUTES.contains(&key_str) {
            break;
        }
        if kv_args.is_none() {
            kv_args = Some(HashMap::new());
        }

        kv_args.as_mut().unwrap().insert(key_str, value);
    }

    Ok(kv_args)
}

/// A wrapper struct that implements the `log::kv::Source` trait for Python key-value pairs.
///
/// This struct allows Python LogRecord custom attributes to be used with Rust's
/// structured logging system by implementing the necessary trait for key-value handling.
///
/// # Type Parameters
/// * `'a` - The lifetime of the contained Python values
pub struct KVSource<'a>(pub HashMap<String, pyo3::Bound<'a, pyo3::PyAny>>);

impl Source for KVSource<'_> {
    /// Visits each key-value pair in the source, converting Python values to debug representations.
    ///
    /// # Arguments
    /// * `visitor` - The visitor that will process each key-value pair
    ///
    /// # Returns
    /// * `Result<(), log::kv::Error>` - Success if all pairs are visited successfully,
    ///   or an error if visitation fails
    fn visit<'kvs>(
        &'kvs self,
        visitor: &mut dyn log::kv::VisitSource<'kvs>,
    ) -> Result<(), log::kv::Error> {
        for (key, value) in &self.0 {
            let v: log::kv::Value<'_> = log::kv::Value::from_debug(value);

            visitor.visit_pair(log::kv::Key::from_str(key), v)?;
        }
        Ok(())
    }
}
