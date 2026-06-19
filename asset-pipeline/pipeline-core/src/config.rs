use crate::error::PipelineError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level configuration document.
///
/// Written in TOML; the file must contain a `[pipeline]` section.
///
/// # Example
/// ```toml
/// [pipeline]
/// watch_dir  = "/incoming/models"
/// output_dir = "/ue4/Content/Assets"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub pipeline: PipelineSection,
}

/// Settings that live under the `[pipeline]` TOML key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSection {
    /// Directory to watch (or scan) for incoming 3D model files.
    pub watch_dir: PathBuf,

    /// Root output directory — staged FBX files are written here.
    pub output_dir: PathBuf,

    /// Path to the Blender executable.  Defaults to `"blender"` (assumed on `PATH`).
    #[serde(default = "default_blender")]
    pub blender_bin: String,

    /// Maximum allowed source file size in megabytes.  Larger files are skipped.
    #[serde(default = "default_max_mb")]
    pub max_file_mb: u64,

    /// `tracing` log filter string (e.g. `"info"`, `"pipeline_core=debug"`).
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_blender() -> String {
    "blender".to_string()
}

fn default_max_mb() -> u64 {
    500
}

fn default_log_level() -> String {
    "info".to_string()
}

impl PipelineConfig {
    /// Read and parse a TOML config file from `path`.
    pub fn from_file(path: &Path) -> Result<Self, PipelineError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PipelineError::Config(format!("cannot read {}: {e}", path.display())))?;
        toml::from_str(&content).map_err(|e| PipelineError::Config(e.to_string()))
    }

    /// Serialize the current config back to a TOML string.
    pub fn to_toml_string(&self) -> Result<String, PipelineError> {
        toml::to_string_pretty(self).map_err(|e| PipelineError::Config(e.to_string()))
    }

    /// A ready-to-paste example configuration file with inline comments.
    pub fn example_toml() -> &'static str {
        r#"[pipeline]
watch_dir  = "incoming"
output_dir = "Content/Assets"
blender_bin = "blender"
max_file_mb = 500
log_level   = "info"
"#
    }

    /// Validate that the configuration is internally consistent.
    ///
    /// Checks that `max_file_mb` is non-zero and that `blender_bin` is non-empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pipeline_core::config::{PipelineConfig, PipelineSection};
    /// use std::path::PathBuf;
    ///
    /// let cfg = PipelineConfig {
    ///     pipeline: PipelineSection {
    ///         watch_dir: PathBuf::from("/in"),
    ///         output_dir: PathBuf::from("/out"),
    ///         blender_bin: "blender".to_string(),
    ///         max_file_mb: 100,
    ///         log_level: "info".to_string(),
    ///     }
    /// };
    /// assert!(cfg.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), PipelineError> {
        if self.pipeline.max_file_mb == 0 {
            return Err(PipelineError::Config(
                "max_file_mb must be greater than 0".into(),
            ));
        }
        if self.pipeline.blender_bin.trim().is_empty() {
            return Err(PipelineError::Config(
                "blender_bin must not be empty".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_toml_parses() {
        let cfg: PipelineConfig =
            toml::from_str(PipelineConfig::example_toml()).expect("parse example toml");
        assert_eq!(cfg.pipeline.max_file_mb, 500);
        assert_eq!(cfg.pipeline.blender_bin, "blender");
        assert_eq!(cfg.pipeline.log_level, "info");
    }

    #[test]
    fn defaults_are_applied() {
        let toml_str = r#"
[pipeline]
watch_dir  = "/tmp/in"
output_dir = "/tmp/out"
"#;
        let cfg: PipelineConfig = toml::from_str(toml_str).expect("parse");
        assert_eq!(cfg.pipeline.blender_bin, "blender");
        assert_eq!(cfg.pipeline.max_file_mb, 500);
        assert_eq!(cfg.pipeline.log_level, "info");
    }

    #[test]
    fn validate_rejects_zero_max_mb() {
        let mut cfg: PipelineConfig = toml::from_str(PipelineConfig::example_toml()).unwrap();
        cfg.pipeline.max_file_mb = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_blender_bin() {
        let mut cfg: PipelineConfig = toml::from_str(PipelineConfig::example_toml()).unwrap();
        cfg.pipeline.blender_bin = "   ".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn round_trip_toml() {
        let cfg: PipelineConfig = toml::from_str(PipelineConfig::example_toml()).unwrap();
        let serialised = cfg.to_toml_string().unwrap();
        let cfg2: PipelineConfig = toml::from_str(&serialised).unwrap();
        assert_eq!(cfg.pipeline.max_file_mb, cfg2.pipeline.max_file_mb);
        assert_eq!(cfg.pipeline.blender_bin, cfg2.pipeline.blender_bin);
    }

    #[test]
    fn from_file_reads_and_parses_toml() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("pipeline.toml");
        std::fs::write(&path, PipelineConfig::example_toml()).unwrap();
        let cfg = PipelineConfig::from_file(&path).expect("from_file should succeed");
        assert_eq!(cfg.pipeline.max_file_mb, 500);
        assert_eq!(cfg.pipeline.log_level, "info");
    }

    #[test]
    fn from_file_returns_error_for_missing_file() {
        let result = PipelineConfig::from_file(std::path::Path::new("/nonexistent/pipeline.toml"));
        assert!(result.is_err());
        if let Err(PipelineError::Config(msg)) = result {
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn validate_passes_for_valid_config() {
        let cfg: PipelineConfig = toml::from_str(PipelineConfig::example_toml()).unwrap();
        assert!(cfg.validate().is_ok());
    }
}
