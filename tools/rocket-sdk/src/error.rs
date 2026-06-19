use std::path::PathBuf;
use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── Display (thiserror) ───────────────────────────────────────────────────

    #[test]
    fn project_not_found_includes_name() {
        let e = RocketError::ProjectNotFound("Brm".into());
        assert!(format!("{e}").contains("Brm"));
    }

    #[test]
    fn no_target_found_includes_project_name() {
        let e = RocketError::NoTargetFound("ShooterGame".into());
        assert!(format!("{e}").contains("ShooterGame"));
    }

    #[test]
    fn ue4_root_not_set_message() {
        let e = RocketError::Ue4RootNotSet;
        assert!(format!("{e}").contains("UE4_ROOT"));
    }

    #[test]
    fn versions_directory_includes_path() {
        let e = RocketError::VersionsDirectoryNotFound(PathBuf::from("/bad/path"));
        assert!(format!("{e}").contains("bad/path") || format!("{e}").contains("bad"));
    }

    #[test]
    fn pwa_directory_includes_name() {
        let e = RocketError::PwaDirectoryNotFound("pwa-staff".into());
        assert!(format!("{e}").contains("pwa-staff"));
    }

    #[test]
    fn manifest_error_includes_detail() {
        let e = RocketError::ManifestError("missing field".into());
        assert!(format!("{e}").contains("missing field"));
    }

    #[test]
    fn generic_error_includes_message() {
        let e = RocketError::Generic("something went wrong".into());
        assert!(format!("{e}").contains("something went wrong"));
    }

    #[test]
    fn parent_directory_not_found_includes_path() {
        let e = RocketError::ParentDirectoryNotFound(PathBuf::from("/some/file.txt"));
        assert!(format!("{e}").contains("file.txt"));
    }

    // ── Debug ────────────────────────────────────────────────────────────────

    #[test]
    fn all_variants_impl_debug() {
        let variants: Vec<RocketError> = vec![
            RocketError::ProjectNotFound("x".into()),
            RocketError::NoTargetFound("x".into()),
            RocketError::Ue4RootNotSet,
            RocketError::VersionsDirectoryNotFound(PathBuf::from(".")),
            RocketError::PwaDirectoryNotFound("x".into()),
            RocketError::ManifestError("x".into()),
            RocketError::Generic("x".into()),
        ];
        for v in &variants {
            assert!(!format!("{v:?}").is_empty());
        }
    }

    // ── From<serde_json::Error> ───────────────────────────────────────────────

    #[test]
    fn json_error_converts_to_rocket_error() {
        let json_err: serde_json::Error = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let e: RocketError = json_err.into();
        assert!(matches!(e, RocketError::Json(_)));
    }
}
