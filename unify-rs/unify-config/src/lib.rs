pub mod manifest;
pub mod sections;
pub mod loader;
pub mod validate;
pub mod merge;
mod tests;

pub use manifest::UnifyManifest;
pub use sections::{
    WorkspaceConfig, CodegenConfig, LspConfig, LspServerConfig,
    CliConfig, TestConfig, OtelConfig, RdfConfig,
};
pub use loader::{ConfigLoader, ConfigFormat};
pub use validate::{ManifestValidator, ManifestViolation};
pub use merge::ConfigMerge;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialize error: {0}")]
    Serialize(String),
}
