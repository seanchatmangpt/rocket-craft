use thiserror::Error;
use std::path::PathBuf;

/// All errors that the asset pipeline can produce.
///
/// Variants carry structured data so callers can make programmatic decisions
/// (e.g. skip duplicates vs. abort on conversion failure).
#[derive(Error, Debug)]
pub enum PipelineError {
    /// The file's extension does not map to any supported 3D format.
    #[error("unsupported format for '{path}': extension '{ext}'")]
    UnsupportedFormat { path: PathBuf, ext: String },

    /// The file exceeds the configured size limit.
    #[error("file too large: '{path}' is {size_mb:.1}MB, limit is {limit_mb}MB")]
    FileTooLarge { path: PathBuf, size_mb: f64, limit_mb: u64 },

    /// Two different source files have identical BLAKE3 content hashes.
    #[error("duplicate asset: '{path}' has same content as already-staged '{existing}'")]
    Duplicate { path: PathBuf, existing: PathBuf },

    /// The configured Blender binary could not be found / executed.
    #[error("blender not found at '{blender_bin}': {source}")]
    BlenderNotFound {
        blender_bin: String,
        #[source]
        source: std::io::Error,
    },

    /// Blender ran but exited with a non-zero status or produced error output.
    #[error("blender conversion failed for '{path}': {stderr}")]
    ConversionFailed { path: PathBuf, stderr: String },

    /// An I/O error occurred while moving / copying files into the staging area.
    #[error("staging failed: {0}")]
    StagingFailed(#[from] std::io::Error),

    /// A configuration file is missing, unreadable, or contains invalid TOML.
    #[error("config error: {0}")]
    Config(String),

    /// JSON manifest serialisation or deserialisation failed.
    #[error("manifest serialization: {0}")]
    Manifest(#[from] serde_json::Error),
}

impl PipelineError {
    /// Returns true for errors that should cause the pipeline to skip the asset
    /// rather than abort the whole run (e.g. unsupported format, duplicate).
    pub fn is_skippable(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedFormat { .. }
            | Self::Duplicate { .. }
            | Self::FileTooLarge { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skippable_variants() {
        let err = PipelineError::UnsupportedFormat {
            path: PathBuf::from("/tmp/a.png"),
            ext: "png".into(),
        };
        assert!(err.is_skippable());

        let err = PipelineError::Config("bad toml".into());
        assert!(!err.is_skippable());
    }

    #[test]
    fn error_messages_contain_path() {
        let err = PipelineError::FileTooLarge {
            path: PathBuf::from("/tmp/big.obj"),
            size_mb: 1024.0,
            limit_mb: 500,
        };
        let msg = err.to_string();
        assert_eq!(msg, "file too large: '/tmp/big.obj' is 1024.0MB, limit is 500MB");
    }
}
