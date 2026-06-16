use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum RocketError {
    #[error("Project '{0}' not found in manifest")]
    ProjectNotFound(String),

    #[error("No target specified for project '{0}' and no default found")]
    NoTargetFound(String),

    #[error("UE4_ROOT not set. Run 'rocket setup' first.")]
    Ue4RootNotSet,

    #[error("Versions directory not found at '{0}'")]
    VersionsDirectoryNotFound(PathBuf),

    #[error("PWA directory not found: {0}")]
    PwaDirectoryNotFound(String),

    #[error("Failed to determine parent directory for '{0}'")]
    ParentDirectoryNotFound(PathBuf),

    #[error("Invalid project manifest: {0}")]
    ManifestError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}
