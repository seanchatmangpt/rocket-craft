pub mod loader;
pub mod manifest;
pub mod merge;
pub mod sections;
mod tests;
pub mod validate;

pub use loader::{ConfigFormat, ConfigLoader};
pub use manifest::UnifyManifest;
pub use merge::ConfigMerge;
pub use sections::{
    CliConfig, CodegenConfig, LspConfig, LspServerConfig, OtelConfig, RdfConfig, TestConfig,
    WorkspaceConfig,
};
pub use validate::{ManifestValidator, ManifestViolation};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialize error: {0}")]
    Serialize(String),
}
