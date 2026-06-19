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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn parse_error_displays_message() {
        let e = GenieError::Parse("bad intent".into());
        assert!(e.to_string().contains("bad intent"));
    }

    #[test]
    fn validation_error_displays_message() {
        let e = GenieError::Validation("coherence violation".into());
        assert!(e.to_string().contains("coherence violation"));
    }

    #[test]
    fn evolution_error_displays_message() {
        let e = GenieError::Evolution("unknown id".into());
        assert!(e.to_string().contains("unknown id"));
    }

    #[test]
    fn deployment_error_displays_message() {
        let e = GenieError::Deployment("UE4 build failed".into());
        assert!(e.to_string().contains("UE4 build failed"));
    }

    #[test]
    fn serde_from_conversion() {
        let json_err: serde_json::Error = serde_json::from_str::<serde_json::Value>("{{").unwrap_err();
        let e: GenieError = json_err.into();
        assert!(matches!(e, GenieError::Serde(_)));
        assert!(e.source().is_some());
    }
}
