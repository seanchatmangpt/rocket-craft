//! `genie-core` — Core module for the Genie 26 World Manufacturing Platform.
//!
//! This crate defines the structures and data models representing the
//! physical and logical state of manufactured worlds, along with their
//! lifecycle interfaces (parsing, validation, compilation, evolution).

pub mod spec;
pub mod errors;
pub mod parser;
pub mod laws;
pub mod receipt_chain;
pub mod layout;
pub mod evolution;
pub mod deployment;

// Re-exports for convenience
pub use errors::GenieError;
pub use spec::WorldSpec;
pub use parser::IntentParser;
pub use laws::{WorldCoherenceLaw, WorldCoherenceGate};
pub use receipt_chain::ReceiptChainManager;
pub use layout::LayoutCompiler;
pub use evolution::WorldEvolver;
pub use deployment::DeploymentManager;

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
