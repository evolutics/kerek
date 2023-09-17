use std::sync;

static LEVEL: sync::OnceLock<Level> = sync::OnceLock::new();

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

pub fn level() -> Level {
    *LEVEL.get().unwrap_or(&Level::Debug)
}

pub fn set_level(level: Level) -> anyhow::Result<()> {
    LEVEL
        .set(level)
        .map_err(|_| anyhow::anyhow!("Log level set twice"))
}

#[macro_export]
macro_rules! info {
    ($($argument:tt)*) => {{
        if $crate::log::level() <= $crate::log::Level::Info {
            eprintln!($($argument)*);
        }
    }};
}

pub use info;
