/// A wrapper type for logging levels that supports both `tracing` and `log` features.
#[cfg(feature = "tracing")]
pub(crate) struct Level(pub tracing::Level);

/// A wrapper type for logging levels that supports both `tracing` and `log` features.
#[cfg(feature = "log")]
pub(crate) struct Level(pub log::Level);

/// Converts a numeric level value to the appropriate logging Level.
///
/// # Arguments
///
/// * `level` - A u8 value representing the logging level:
///   * 40+ = Error
///   * 30-39 = Warn
///   * 20-29 = Info
///   * 10-19 = Debug
///   * 0-9 = Trace
///
/// # Returns
///
/// Returns a `Level` wrapper containing either a `tracing::Level` or `log::Level`
/// depending on which feature is enabled.
pub(crate) fn get_level(level: u8) -> Level {
    #[cfg(feature = "log")]
    {
        if level.ge(&40u8) {
            Level(log::Level::Error)
        } else if level.ge(&30u8) {
            Level(log::Level::Warn)
        } else if level.ge(&20u8) {
            Level(log::Level::Info)
        } else if level.ge(&10u8) {
            Level(log::Level::Debug)
        } else {
            Level(log::Level::Trace)
        }
    }
    #[cfg(feature = "tracing")]
    {
        if level.ge(&40u8) {
            Level(tracing::Level::ERROR)
        } else if level.ge(&30u8) {
            Level(tracing::Level::WARN)
        } else if level.ge(&20u8) {
            Level(tracing::Level::INFO)
        } else if level.ge(&10u8) {
            Level(tracing::Level::DEBUG)
        } else {
            Level(tracing::Level::TRACE)
        }
    }
}
