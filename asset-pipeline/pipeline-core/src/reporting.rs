use crate::{PipelineError, StagedAsset};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A record of one asset's journey through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRecord {
    pub source_path: PathBuf,
    pub source_format: String,
    pub content_hash_hex: String,
    pub status: AssetStatus,
    pub content_path: Option<PathBuf>,
    pub error: Option<String>,
    pub processed_at: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Success,
    ValidationFailed,
    ConversionFailed,
    StagingFailed,
    Skipped,
}

/// The pipeline's persistent JSON manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineManifest {
    pub version: String, // "1.0"
    pub runs: Vec<RunRecord>,
}

/// One invocation of the pipeline (either --once or one watch cycle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub watch_dir: PathBuf,
    pub output_dir: PathBuf,
    pub assets: Vec<AssetRecord>,
    pub total_success: usize,
    pub total_failed: usize,
    pub total_skipped: usize,
}

/// Manages reading and writing the JSON manifest file.
pub struct Reporter {
    pub manifest_path: PathBuf,
}

impl Reporter {
    pub fn new(output_dir: &Path) -> Self {
        Self {
            manifest_path: output_dir.join("pipeline-manifest.json"),
        }
    }

    /// Load existing manifest or create a fresh one.
    pub fn load_or_create(&self) -> Result<PipelineManifest, PipelineError> {
        if self.manifest_path.exists() {
            let content = std::fs::read_to_string(&self.manifest_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(PipelineManifest {
                version: "1.0".to_string(),
                runs: Vec::new(),
            })
        }
    }

    /// Start a new run record.
    pub fn begin_run(watch_dir: PathBuf, output_dir: PathBuf) -> RunRecord {
        RunRecord {
            started_at: Utc::now(),
            finished_at: None,
            watch_dir,
            output_dir,
            assets: Vec::new(),
            total_success: 0,
            total_failed: 0,
            total_skipped: 0,
        }
    }

    /// Record a successful staged asset.
    pub fn record_success(run: &mut RunRecord, asset: &StagedAsset, duration_ms: u64) {
        run.assets.push(AssetRecord {
            source_path: asset.path.clone(),
            source_format: "fbx".to_string(), // post-conversion always FBX
            content_hash_hex: hex_hash(&asset.hash),
            status: AssetStatus::Success,
            content_path: Some(asset.content_path.clone()),
            error: None,
            processed_at: Utc::now(),
            duration_ms,
        });
        run.total_success += 1;
    }

    /// Record a failed or skipped asset.
    pub fn record_failure(
        run: &mut RunRecord,
        source_path: PathBuf,
        hash: [u8; 32],
        status: AssetStatus,
        err: &PipelineError,
        duration_ms: u64,
    ) {
        let is_skipped = matches!(status, AssetStatus::Skipped);
        run.assets.push(AssetRecord {
            source_path,
            source_format: String::new(),
            content_hash_hex: hex_hash(&hash),
            status,
            content_path: None,
            error: Some(err.to_string()),
            processed_at: Utc::now(),
            duration_ms,
        });
        if is_skipped {
            run.total_skipped += 1;
        } else {
            run.total_failed += 1;
        }
    }

    /// Finish the run and atomically write the updated manifest.
    pub fn finish_run(
        &self,
        manifest: &mut PipelineManifest,
        mut run: RunRecord,
    ) -> Result<(), PipelineError> {
        run.finished_at = Some(Utc::now());
        manifest.runs.push(run);
        self.write_manifest(manifest)
    }

    /// Atomically write manifest: write to .tmp then rename.
    fn write_manifest(&self, manifest: &PipelineManifest) -> Result<(), PipelineError> {
        let tmp = self.manifest_path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(manifest)?;
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &self.manifest_path)?;
        Ok(())
    }

    /// Print a human-readable summary of the last run.
    pub fn print_summary(run: &RunRecord) {
        let duration = run
            .finished_at
            .map(|end| (end - run.started_at).num_milliseconds())
            .unwrap_or(0);
        tracing::info!("\n\u{2500}\u{2500}\u{2500} Pipeline Run Summary \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
        tracing::info!("  \u{2713} Staged:  {}", run.total_success);
        tracing::info!("  \u{2717} Failed:  {}", run.total_failed);
        tracing::info!("  \u{2298} Skipped: {}", run.total_skipped);
        tracing::info!("  Duration:  {}ms", duration);
        tracing::info!("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n");
    }
}

fn hex_hash(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_staged_asset(tmp: &TempDir) -> StagedAsset {
        StagedAsset {
            path: tmp.path().join("model.obj"),
            hash: [0xabu8; 32],
            content_path: tmp.path().join("content/model.fbx"),
        }
    }

    #[test]
    fn begin_run_creates_record_with_correct_dirs() {
        let watch = PathBuf::from("/watch");
        let output = PathBuf::from("/output");
        let run = Reporter::begin_run(watch.clone(), output.clone());

        assert_eq!(run.watch_dir, watch);
        assert_eq!(run.output_dir, output);
        assert!(run.finished_at.is_none());
        assert_eq!(run.total_success, 0);
    }

    #[test]
    fn finish_run_writes_manifest_json() {
        let tmp = TempDir::new().unwrap();
        let reporter = Reporter::new(tmp.path());

        let mut manifest = reporter.load_or_create().unwrap();
        let run = Reporter::begin_run(PathBuf::from("/watch"), tmp.path().to_path_buf());
        reporter.finish_run(&mut manifest, run).unwrap();

        assert!(reporter.manifest_path.exists());
        let content = std::fs::read_to_string(&reporter.manifest_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["version"], "1.0");
    }

    #[test]
    fn load_or_create_reads_existing_manifest() {
        let tmp = TempDir::new().unwrap();
        let reporter = Reporter::new(tmp.path());

        // Write a manifest
        let mut manifest = reporter.load_or_create().unwrap();
        let run = Reporter::begin_run(PathBuf::from("/watch"), tmp.path().to_path_buf());
        reporter.finish_run(&mut manifest, run).unwrap();

        // Read it back
        let loaded = reporter.load_or_create().unwrap();
        assert_eq!(loaded.version, "1.0");
        assert_eq!(loaded.runs.len(), 1);
    }

    #[test]
    fn hex_hash_produces_64_char_lowercase_hex() {
        let hash = [0xabu8; 32];
        let result = hex_hash(&hash);
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(result, "ab".repeat(32));
    }

    #[test]
    fn record_success_increments_total_success() {
        let tmp = TempDir::new().unwrap();
        let mut run = Reporter::begin_run(PathBuf::from("/watch"), tmp.path().to_path_buf());
        let asset = make_staged_asset(&tmp);
        Reporter::record_success(&mut run, &asset, 100);

        assert_eq!(run.total_success, 1);
        assert_eq!(run.assets.len(), 1);
        assert_eq!(run.assets[0].status, AssetStatus::Success);
    }
}
