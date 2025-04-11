use std::sync::OnceLock;

pub(crate) static BASE_URL: OnceLock<String> = OnceLock::new();
pub(crate) static AUTH_KEY: OnceLock<Option<String>> = OnceLock::new();
pub(crate) static DRY: OnceLock<bool> = OnceLock::new();
