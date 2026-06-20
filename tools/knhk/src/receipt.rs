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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields_correctly() {
        let r = Receipt::new("path/to/law.wasm".into(), "MyLaw", true, "all good");
        assert_eq!(r.plugin_path, PathBuf::from("path/to/law.wasm"));
        assert_eq!(r.law_name, "MyLaw");
        assert!(r.passed);
        assert_eq!(r.message, "all good");
    }

    #[test]
    fn new_failing_receipt() {
        let r = Receipt::new("law.wasm".into(), "StrictLaw", false, "violation found");
        assert!(!r.passed);
        assert_eq!(r.message, "violation found");
    }

    #[test]
    fn timestamp_is_recent() {
        let before = std::time::SystemTime::now();
        let r = Receipt::new("l.wasm".into(), "L", true, "ok");
        let after = std::time::SystemTime::now();
        assert!(r.timestamp >= before);
        assert!(r.timestamp <= after);
    }
}
