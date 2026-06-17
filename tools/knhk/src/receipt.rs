use std::path::PathBuf;

/// Records the outcome of a WASM law validation run.
#[derive(Debug, Clone)]
pub struct Receipt {
    pub plugin_path: PathBuf,
    pub law_name: String,
    pub passed: bool,
    pub message: String,
    pub timestamp: std::time::SystemTime,
}

impl Receipt {
    pub fn new(
        plugin_path: PathBuf,
        law_name: impl Into<String>,
        passed: bool,
        message: impl Into<String>,
    ) -> Self {
        Self {
            plugin_path,
            law_name: law_name.into(),
            passed,
            message: message.into(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}
