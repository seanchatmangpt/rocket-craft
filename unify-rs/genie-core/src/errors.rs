use thiserror::Error;

/// Core error type for the Genie 26 World Manufacturing Platform.
#[derive(Debug, Error)]
pub enum GenieError {
    /// Standard I/O operations errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors.
    #[error("JSON Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Errors encountered while parsing natural language or raw structures.
    #[error("Intent parsing error: {0}")]
    Parse(String),

    /// Semantic validation refusals and coherence check failures.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Errors during incremental evolution of the world specification.
    #[error("World evolution error: {0}")]
    Evolution(String),

    /// Operational simulation or deployment failures.
    #[error("Deployment error: {0}")]
    Deployment(String),
}
