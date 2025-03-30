#[cfg(feature = "tracing")]
pub(crate) struct Level(pub tracing::Level);

#[cfg(feature = "log")]
pub(crate) struct Level(pub log::Level);

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
