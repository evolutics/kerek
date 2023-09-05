use std::sync;

static IS_LEVEL_DEBUG_OR_INFO: sync::OnceLock<bool> = sync::OnceLock::new();

pub fn is_level_debug_or_info() -> bool {
    IS_LEVEL_DEBUG_OR_INFO.get().copied().unwrap_or(true)
}

pub fn set_level(is_level_debug_or_info: bool) -> anyhow::Result<()> {
    IS_LEVEL_DEBUG_OR_INFO
        .set(is_level_debug_or_info)
        .map_err(|_| anyhow::anyhow!("Log level set twice"))
}

#[macro_export]
macro_rules! info {
    ($($argument:tt)*) => {{
        if $crate::log::is_level_debug_or_info() {
            eprintln!($($argument)*);
        }
    }};
}

pub use info;
