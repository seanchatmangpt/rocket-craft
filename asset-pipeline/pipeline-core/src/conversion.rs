use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use serde::Deserialize;

use crate::{ConvertedAsset, PipelineError, ValidatedAsset};

/// Result envelope that Blender's Python script prints as the last stdout line.
#[derive(Debug, Deserialize)]
struct BlenderResult {
    ok: bool,
    output_path: Option<String>,
    error: Option<String>,
}

/// Orchestrates headless Blender to convert 3D source files to UE4 FBX.
#[derive(Debug, Clone)]
pub struct BlenderConverter {
    /// Path to the Blender binary (absolute or found via PATH).
    pub blender_path: PathBuf,
    /// Absolute path to the blender_convert.py script.
    pub script_path: PathBuf,
    /// Subprocess timeout in seconds (default: 300 = 5 minutes).
    pub timeout_secs: u64,
}

impl BlenderConverter {
    /// Discover Blender using:
    /// 1. `BLENDER_PATH` environment variable
    /// 2. Well-known OS paths: macOS .app bundle, /usr/bin/blender, /snap/bin/blender
    /// 3. `blender` in PATH (fallback)
    pub fn discover(script_path: PathBuf) -> Result<Self, PipelineError> {
        let candidates = {
            let mut v: Vec<PathBuf> = Vec::new();
            if let Ok(p) = std::env::var("BLENDER_PATH") {
                v.push(PathBuf::from(p));
            }
            // Well-known paths
            v.push(PathBuf::from("/Applications/Blender.app/Contents/MacOS/blender")); // macOS
            v.push(PathBuf::from("/usr/bin/blender"));
            v.push(PathBuf::from("/usr/local/bin/blender"));
            v.push(PathBuf::from("/snap/bin/blender"));
            v.push(PathBuf::from("blender")); // fallback: must be in PATH
            v
        };

        for candidate in candidates {
            // Try to probe: run `blender --version` synchronously
            match std::process::Command::new(&candidate)
                .arg("--version")
                .output()
            {
                Ok(out) if out.status.success() => {
                    let version = String::from_utf8_lossy(&out.stdout);
                    info!(
                        "Found Blender at {}: {}",
                        candidate.display(),
                        version.lines().next().unwrap_or("")
                    );
                    return Ok(Self {
                        blender_path: candidate,
                        script_path,
                        timeout_secs: 300,
                    });
                }
                Ok(_) => {
                    debug!(
                        "Blender at {} exited non-zero for --version",
                        candidate.display()
                    );
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    debug!("Blender not found at {}", candidate.display());
                }
                Err(e) => {
                    warn!("Error probing {}: {e}", candidate.display());
                }
            }
        }

        Err(PipelineError::BlenderNotFound {
            blender_bin: "blender".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found in any candidate path",
            ),
        })
    }

    /// Convert a ValidatedAsset to FBX using headless Blender.
    ///
    /// If the source is already FBX (`Format::Fbx`), skips Blender and
    /// copies the file to `work_dir` instead.
    ///
    /// `work_dir`: a temp directory for intermediate .fbx files before staging.
    pub async fn convert(
        &self,
        asset: ValidatedAsset,
        work_dir: &Path,
    ) -> Result<ConvertedAsset, (PipelineError, ValidatedAsset)> {
        // Fast path: already FBX — just copy
        if asset.source_format.is_fbx() {
            let fbx_path = work_dir.join(format!("{}.fbx", asset.name()));
            debug!(
                "Asset {} is already FBX — copying to {}",
                asset.path.display(),
                fbx_path.display()
            );
            if let Err(e) = std::fs::copy(&asset.path, &fbx_path) {
                return Err((PipelineError::StagingFailed(e), asset));
            }
            return Ok(asset.into_converted(fbx_path));
        }

        let fbx_path = work_dir.join(format!("{}.fbx", asset.name()));

        info!(
            "Converting {} ({}) → {}",
            asset.path.display(),
            asset.source_format,
            fbx_path.display()
        );

        let mut child = match Command::new(&self.blender_path)
            .args(["--background", "--python"])
            .arg(&self.script_path)
            .arg("--")
            .arg("--input")
            .arg(&asset.path)
            .arg("--output")
            .arg(&fbx_path)
            .arg("--format")
            .arg(asset.source_format.extension())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Failed to spawn Blender at {}: {e}",
                    self.blender_path.display()
                );
                return Err((
                    PipelineError::BlenderNotFound {
                        blender_bin: self.blender_path.to_string_lossy().to_string(),
                        source: e,
                    },
                    asset,
                ));
            }
        };

        // `wait_with_output` takes ownership of `child`.  We instead extract the
        // stdout/stderr pipes manually and use `child.wait()` (takes `&mut self`)
        // so that `child` remains accessible for `kill()` on timeout.
        use tokio::io::AsyncReadExt as _;
        let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
        let mut stderr_pipe = child.stderr.take().expect("stderr was piped");

        let timeout_dur = Duration::from_secs(self.timeout_secs);

        let wait_result = tokio::select! {
            status = child.wait() => Some(status),
            _ = tokio::time::sleep(timeout_dur) => None,
        };

        if wait_result.is_none() {
            // Timed out — child is still alive, kill it.
            let _ = child.kill().await;
            return Err((
                PipelineError::ConversionFailed {
                    path: asset.path.clone(),
                    stderr: format!("timed out after {}s", self.timeout_secs),
                },
                asset,
            ));
        }

        if let Err(e) = wait_result.unwrap() {
            return Err((
                PipelineError::ConversionFailed {
                    path: asset.path.clone(),
                    stderr: format!("process error: {e}"),
                },
                asset,
            ));
        }

        // Drain the pipes now that the process has exited.
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut stdout_bytes).await;
        let _ = stderr_pipe.read_to_end(&mut stderr_bytes).await;

        let stdout = String::from_utf8_lossy(&stdout_bytes);
        let stderr = String::from_utf8_lossy(&stderr_bytes);

        debug!("Blender stdout:\n{stdout}");
        if !stderr.is_empty() {
            debug!("Blender stderr:\n{stderr}");
        }

        // Parse the last non-empty stdout line as JSON
        let last_line = stdout.lines().filter(|l| !l.trim().is_empty()).last();

        let blender_result: Option<BlenderResult> =
            last_line.and_then(|l| serde_json::from_str(l).ok());

        match blender_result {
            Some(BlenderResult {
                ok: true,
                output_path: Some(ref out_path),
                ..
            }) => {
                let fbx = PathBuf::from(out_path);
                if fbx.exists() {
                    info!(
                        "Conversion succeeded: {} → {}",
                        asset.path.display(),
                        fbx.display()
                    );
                    Ok(asset.into_converted(fbx))
                } else {
                    error!("Blender reported success but output file is missing: {out_path}");
                    Err((
                        PipelineError::ConversionFailed {
                            path: asset.path.clone(),
                            stderr: format!("output file missing: {out_path}"),
                        },
                        asset,
                    ))
                }
            }
            Some(BlenderResult {
                ok: false,
                error: Some(ref err),
                ..
            }) => {
                error!("Blender reported failure for {}: {err}", asset.path.display());
                Err((
                    PipelineError::ConversionFailed {
                        path: asset.path.clone(),
                        stderr: err.clone(),
                    },
                    asset,
                ))
            }
            _ => {
                // No parseable JSON result — capture whatever stderr we got
                let stderr_snippet = stderr.chars().take(500).collect::<String>();
                error!(
                    "Blender conversion for {} produced no parseable result. stderr: {stderr_snippet}",
                    asset.path.display()
                );
                Err((
                    PipelineError::ConversionFailed {
                        path: asset.path.clone(),
                        stderr: stderr_snippet,
                    },
                    asset,
                ))
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Format;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper: create a ValidatedAsset pointing at a real temp file.
    fn make_validated_asset(tmp: &TempDir, filename: &str, format: Format) -> ValidatedAsset {
        let path = tmp.path().join(filename);
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"dummy content")
            .unwrap();
        ValidatedAsset {
            path,
            hash: [0u8; 32],
            source_format: format,
            file_size_bytes: 13,
        }
    }

    #[test]
    fn discover_fails_gracefully_when_blender_absent() {
        // Point BLENDER_PATH at a path that definitely doesn't exist
        // and clear any real BLENDER_PATH from the environment.
        // We use a temp-unique name to avoid colliding with a real install.
        let original = std::env::var("BLENDER_PATH").ok();

        std::env::set_var(
            "BLENDER_PATH",
            "/tmp/__nonexistent_blender_binary_for_test__",
        );

        let result = BlenderConverter::discover(PathBuf::from("/dev/null"));

        // Restore environment
        match original {
            Some(v) => std::env::set_var("BLENDER_PATH", v),
            None => std::env::remove_var("BLENDER_PATH"),
        }

        // On CI / dev machines where blender is not installed this will be Err.
        // We only assert the error variant — not that it's always Err, because
        // some CI runners have blender in PATH.
        if let Err(e) = result {
            assert!(
                matches!(e, PipelineError::BlenderNotFound { .. }),
                "expected BlenderNotFound, got: {e:?}"
            );
        }
    }

    #[tokio::test]
    async fn fbx_fast_path_copies_without_spawning_blender() {
        let work_tmp = TempDir::new().unwrap();
        let src_tmp = TempDir::new().unwrap();

        let asset = make_validated_asset(&src_tmp, "rocket.fbx", Format::Fbx);
        let original_path = asset.path.clone();

        // Use a non-existent blender binary — the fast path must never spawn it.
        let converter = BlenderConverter {
            blender_path: PathBuf::from("/nonexistent/blender"),
            script_path: PathBuf::from("/nonexistent/blender_convert.py"),
            timeout_secs: 10,
        };

        let result = converter.convert(asset, work_tmp.path()).await;

        match result {
            Ok(converted) => {
                assert_eq!(
                    converted.fbx_path,
                    work_tmp.path().join("rocket.fbx"),
                    "fbx_path should be in work_dir"
                );
                assert!(
                    converted.fbx_path.exists(),
                    "copied FBX must exist on disk"
                );
                assert_eq!(
                    converted.path, original_path,
                    "original source path preserved"
                );
                assert_eq!(
                    converted.source_format,
                    Format::Fbx,
                    "format preserved"
                );
            }
            Err((e, _)) => panic!("FBX fast-path should not fail: {e}"),
        }
    }

    #[tokio::test]
    async fn non_fbx_without_blender_returns_conversion_error() {
        let work_tmp = TempDir::new().unwrap();
        let src_tmp = TempDir::new().unwrap();

        let asset = make_validated_asset(&src_tmp, "cube.obj", Format::Obj);

        let converter = BlenderConverter {
            blender_path: PathBuf::from("/nonexistent/blender"),
            script_path: PathBuf::from("/nonexistent/blender_convert.py"),
            timeout_secs: 10,
        };

        let result = converter.convert(asset, work_tmp.path()).await;

        assert!(result.is_err(), "should fail without a real Blender binary");
        let (err, _asset) = result.unwrap_err();
        assert!(
            matches!(err, PipelineError::BlenderNotFound { .. }),
            "expected BlenderNotFound, got: {err:?}"
        );
    }
}
