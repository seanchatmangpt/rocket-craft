//! `genie-core` — Core module for the Genie 26 World Manufacturing Platform.
//!
//! This crate defines the structures and data models representing the
//! physical and logical state of manufactured worlds, along with their
//! lifecycle interfaces (parsing, validation, compilation, evolution).

pub mod deployment;
pub mod errors;
pub mod evolution;
pub mod laws;
pub mod layout;
pub mod parser;
pub mod receipt_chain;
pub mod spec;

// Re-exports for convenience
pub use deployment::DeploymentManager;
pub use errors::GenieError;
pub use evolution::WorldEvolver;
pub use laws::{WorldCoherenceGate, WorldCoherenceLaw};
pub use layout::LayoutCompiler;
pub use parser::IntentParser;
pub use receipt_chain::ReceiptChainManager;
pub use spec::WorldSpec;

use std::path::Path;

/// Parses natural language intent into a structured World Specification.
pub fn parse_intent(intent: &str) -> Result<WorldSpec, GenieError> {
    if intent.trim().is_empty() {
        return Err(GenieError::Parse("Intent cannot be empty".to_string()));
    }
    IntentParser::parse(intent)
}

/// Loads a serialized WorldSpec JSON from a file.
pub fn load_spec(path: &Path) -> Result<WorldSpec, GenieError> {
    let content = std::fs::read_to_string(path)?;
    let spec: WorldSpec = serde_json::from_str(&content)?;
    Ok(spec)
}

/// Saves the serialized WorldSpec JSON to a file.
pub fn save_spec(spec: &WorldSpec, path: &Path) -> Result<(), GenieError> {
    let content = serde_json::to_string_pretty(spec)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_intent_empty_returns_error() {
        assert!(parse_intent("").is_err());
        assert!(parse_intent("   ").is_err());
    }

    #[test]
    fn parse_intent_empty_message_contains_cannot_be_empty() {
        let err = parse_intent("").unwrap_err();
        assert!(err.to_string().contains("empty") || err.to_string().contains("Intent"));
    }

    #[test]
    fn parse_intent_nonempty_returns_ok_or_validation_error() {
        // IntentParser may succeed or fail on real text — both are valid outcomes.
        // What must NOT happen: panic or an Io/Serde error.
        match parse_intent("build a jungle arena with two platforms") {
            Ok(spec) => assert!(!spec.engine_version.is_empty()),
            Err(GenieError::Parse(_)) | Err(GenieError::Validation(_)) => {}
            Err(e) => panic!("unexpected error kind: {e}"),
        }
    }

    #[test]
    fn load_spec_missing_file_returns_io_error() {
        let err = load_spec(Path::new("/nonexistent/path/spec.json")).unwrap_err();
        assert!(matches!(err, GenieError::Io(_)));
    }

    #[test]
    fn load_spec_invalid_json_returns_serde_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not json at all").unwrap();
        let err = load_spec(&path).unwrap_err();
        assert!(matches!(err, GenieError::Serde(_)));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("spec.json");
        let spec = WorldSpec::new();
        save_spec(&spec, &path).unwrap();
        let loaded = load_spec(&path).unwrap();
        assert_eq!(loaded.engine_version, "UE4.27-ES3");
        assert!(loaded.places.is_empty());
        assert!(loaded.actors.is_empty());
    }

    #[test]
    fn world_spec_new_sets_engine_version() {
        let spec = WorldSpec::new();
        assert_eq!(spec.engine_version, "UE4.27-ES3");
    }

    #[test]
    fn save_spec_fails_on_unwritable_path() {
        let spec = WorldSpec::new();
        let err = save_spec(&spec, Path::new("/nonexistent/dir/spec.json")).unwrap_err();
        assert!(matches!(err, GenieError::Io(_)));
    }
}
