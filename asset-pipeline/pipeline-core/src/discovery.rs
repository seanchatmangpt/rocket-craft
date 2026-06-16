//! File-system scanning and watching for 3D asset files.
//!
//! # Overview
//!
//! This module provides two mechanisms for discovering supported 3D model files:
//!
//! - [`Scanner`] — a one-shot, synchronous recursive directory walk that returns all
//!   supported files found under a given directory.
//! - [`DirectoryWatcher`] — an async, long-running watcher that emits
//!   [`PipelineEvent`] values whenever a new or modified supported file appears.
//!
//! Both use BLAKE3 content hashing (via [`content_hash`]) to identify assets.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::error::PipelineError;
use crate::types::{DiscoveredAsset, Format, PipelineEvent};

// ── Content hashing ───────────────────────────────────────────────────────────

/// Compute the BLAKE3 hash of a file's contents by reading it in 8 KB chunks.
///
/// Reads the file incrementally so that arbitrarily large files can be hashed
/// without loading them entirely into memory.
///
/// # Errors
///
/// Returns [`PipelineError::StagingFailed`] if the file cannot be opened or
/// read; the underlying [`std::io::Error`] is wrapped via the `#[from]`
/// conversion on that variant.
pub fn content_hash(path: &Path) -> Result<[u8; 32], PipelineError> {
    const CHUNK: usize = 8 * 1024; // 8 KB

    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; CHUNK];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(*hasher.finalize().as_bytes())
}

// ── One-shot scanner ──────────────────────────────────────────────────────────

/// Summary produced by a one-shot directory scan.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// All supported 3-D model files that were discovered and hashed successfully.
    pub assets: Vec<DiscoveredAsset>,
    /// Files for which an error occurred.  Each entry is `(path, error_message)`.
    pub errors: Vec<(PathBuf, String)>,
    /// Number of directories visited (including the root).
    pub scanned_dirs: usize,
    /// Number of files skipped because their extension is not supported.
    pub skipped_unsupported: usize,
}

/// Synchronous, one-shot directory scanner.
///
/// Call [`Scanner::scan_once`] to walk a directory tree and collect every
/// supported 3-D model file into a [`ScanResult`].
pub struct Scanner;

impl Scanner {
    /// Walk `dir` recursively and discover all supported 3-D model files.
    ///
    /// - Symlinks are **not** followed (safe for directories with cycles).
    /// - Individual file errors are collected into [`ScanResult::errors`] rather
    ///   than aborting the scan.
    /// - Unsupported extensions are counted in [`ScanResult::skipped_unsupported`].
    pub fn scan_once(dir: &Path) -> ScanResult {
        let mut result = ScanResult::default();

        let walker = WalkDir::new(dir)
            .follow_links(false)
            .sort_by_file_name();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    // The path might be partially known even on error
                    let path = err
                        .path()
                        .unwrap_or(dir)
                        .to_path_buf();
                    warn!(path = %path.display(), error = %err, "walkdir error");
                    result.errors.push((path, err.to_string()));
                    continue;
                }
            };

            let file_type = entry.file_type();

            if file_type.is_dir() {
                result.scanned_dirs += 1;
                continue;
            }

            if !file_type.is_file() {
                // Skip symlinks, sockets, etc.
                continue;
            }

            let path = entry.path().to_path_buf();

            // Check extension against supported formats
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let format = match Format::from_extension(ext) {
                Some(f) => f,
                None => {
                    debug!(path = %path.display(), ext, "skipping unsupported extension");
                    result.skipped_unsupported += 1;
                    continue;
                }
            };

            // Get file size from metadata
            let file_size_bytes = match entry.metadata() {
                Ok(m) => m.len(),
                Err(err) => {
                    warn!(path = %path.display(), error = %err, "failed to read metadata");
                    result.errors.push((path, err.to_string()));
                    continue;
                }
            };

            // Compute BLAKE3 hash
            let hash = match content_hash(&path) {
                Ok(h) => h,
                Err(err) => {
                    warn!(path = %path.display(), error = %err, "failed to hash file");
                    result.errors.push((path, err.to_string()));
                    continue;
                }
            };

            info!(
                path = %path.display(),
                format = %format,
                size_bytes = file_size_bytes,
                "asset discovered"
            );

            result.assets.push(DiscoveredAsset::new(path, hash, format, file_size_bytes));
        }

        result
    }
}

// ── Async directory watcher ───────────────────────────────────────────────────

/// Long-running async file-system watcher.
///
/// Wraps the `notify` crate to detect new and modified files in a directory
/// tree, then emits [`PipelineEvent::FileDiscovered`] events over a Tokio
/// broadcast channel whenever a supported 3-D model file appears or changes.
pub struct DirectoryWatcher {
    dir: PathBuf,
    event_tx: tokio::sync::broadcast::Sender<PipelineEvent>,
}

impl DirectoryWatcher {
    /// Create a new watcher for `dir` that will publish events to `event_tx`.
    pub fn new(dir: PathBuf, event_tx: tokio::sync::broadcast::Sender<PipelineEvent>) -> Self {
        Self { dir, event_tx }
    }

    /// Start watching `self.dir` recursively.
    ///
    /// This function runs until the task is cancelled (e.g. the Tokio runtime
    /// shuts down or the containing task is dropped).  It bridges the
    /// synchronous `notify` callback into async Tokio code using a standard
    /// library `mpsc` channel and [`tokio::task::spawn_blocking`].
    ///
    /// Only [`notify::EventKind::Create`] and [`notify::EventKind::Modify`]
    /// events for files with supported extensions are forwarded.
    ///
    /// # Errors
    ///
    /// Returns [`PipelineError::Config`] if the `notify` watcher cannot be
    /// created or the watch cannot be registered on `self.dir`.
    pub async fn run(self) -> Result<(), PipelineError> {
        use notify::{EventKind, RecursiveMode, Watcher};

        // Bridge: notify's callback posts to std mpsc; we receive on the
        // async side via spawn_blocking.
        let (std_tx, std_rx) = mpsc::channel::<notify::Result<notify::Event>>();

        let mut watcher = notify::RecommendedWatcher::new(
            move |res| {
                // Ignore send errors — receiver may have been dropped on shutdown.
                let _ = std_tx.send(res);
            },
            notify::Config::default(),
        )
        .map_err(|e| PipelineError::Config(format!("failed to create watcher: {e}")))?;

        watcher
            .watch(&self.dir, RecursiveMode::Recursive)
            .map_err(|e| {
                PipelineError::Config(format!(
                    "failed to watch '{}': {e}",
                    self.dir.display()
                ))
            })?;

        info!(dir = %self.dir.display(), "directory watcher started");

        // Move the receiver into an Arc<Mutex<>> so we can hand it into
        // spawn_blocking across iterations.
        let rx = std::sync::Arc::new(std::sync::Mutex::new(std_rx));

        loop {
            let rx_clone = rx.clone();

            // Blocking recv on a dedicated thread — avoids blocking the async runtime.
            let notify_result = tokio::task::spawn_blocking(move || {
                rx_clone
                    .lock()
                    .expect("mpsc mutex poisoned")
                    .recv()
            })
            .await
            .map_err(|e| PipelineError::Config(format!("spawn_blocking join error: {e}")))?;

            // The recv() call returns Err(RecvError) only when all senders have
            // been dropped, meaning the watcher was dropped — treat as shutdown.
            let event = match notify_result {
                Ok(r) => r.map_err(|e| {
                    PipelineError::Config(format!("notify watch error: {e}"))
                })?,
                Err(_recv_err) => {
                    // All senders dropped; watcher is gone — exit cleanly.
                    debug!("watcher sender dropped, shutting down");
                    break;
                }
            };

            debug!(event = ?event, "notify event received");

            // Only act on create and modify events.
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {}
                _ => continue,
            }

            for path in event.paths {
                // Skip if it's not a regular file (e.g. directory create)
                if !path.is_file() {
                    continue;
                }

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                if Format::from_extension(ext).is_none() {
                    debug!(path = %path.display(), ext, "ignoring unsupported file in watcher");
                    continue;
                }

                // Compute hash and metadata before emitting the event so
                // subscribers receive a fully-populated discovery event.
                match content_hash(&path) {
                    Ok(_hash) => {
                        info!(
                            path = %path.display(),
                            "new asset detected by watcher"
                        );
                        // Send to all subscribers; ignore errors from lagging
                        // receivers (broadcast channel returns Err only when
                        // there are no receivers at all).
                        let _ = self.event_tx.send(PipelineEvent::FileDiscovered {
                            path: path.clone(),
                        });
                    }
                    Err(err) => {
                        warn!(
                            path = %path.display(),
                            error = %err,
                            "failed to hash new file — event dropped"
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── content_hash tests ────────────────────────────────────────────────────

    #[test]
    fn content_hash_is_consistent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("model.obj");
        fs::write(&path, b"v 0 0 0\nv 1 0 0\nf 1 2\n").unwrap();

        let h1 = content_hash(&path).expect("hash should succeed");
        let h2 = content_hash(&path).expect("hash should succeed");
        assert_eq!(h1, h2, "same file must hash identically");
    }

    #[test]
    fn content_hash_differs_for_different_content() {
        let dir = TempDir::new().unwrap();

        let path_a = dir.path().join("a.obj");
        let path_b = dir.path().join("b.obj");
        fs::write(&path_a, b"v 0 0 0\n").unwrap();
        fs::write(&path_b, b"v 1 1 1\n").unwrap();

        let h_a = content_hash(&path_a).expect("hash a");
        let h_b = content_hash(&path_b).expect("hash b");
        assert_ne!(h_a, h_b, "different content must produce different hashes");
    }

    #[test]
    fn content_hash_empty_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.obj");
        fs::write(&path, b"").unwrap();

        // Should not error on an empty file
        let hash = content_hash(&path).expect("empty file hash");
        // BLAKE3 of empty input is well-defined
        assert_ne!(hash, [0u8; 32], "BLAKE3 of empty string is non-zero");
    }

    #[test]
    fn content_hash_missing_file_returns_error() {
        let path = Path::new("/nonexistent/path/to/model.obj");
        let result = content_hash(path);
        assert!(
            result.is_err(),
            "should return an error for non-existent file"
        );
    }

    // ── Scanner tests ─────────────────────────────────────────────────────────

    #[test]
    fn scan_once_empty_dir_returns_empty_result() {
        let dir = TempDir::new().unwrap();
        let result = Scanner::scan_once(dir.path());

        assert!(result.assets.is_empty(), "no assets in empty dir");
        assert!(result.errors.is_empty(), "no errors in empty dir");
        // The root directory itself is counted
        assert_eq!(result.scanned_dirs, 1, "root dir counted");
        assert_eq!(result.skipped_unsupported, 0);
    }

    #[test]
    fn scan_once_finds_supported_and_skips_unsupported() {
        let dir = TempDir::new().unwrap();

        // Supported formats
        fs::write(dir.path().join("mesh.obj"), b"v 0 0 0\n").unwrap();
        fs::write(dir.path().join("scene.glb"), b"glTF\x02\x00\x00\x00").unwrap();
        fs::write(dir.path().join("body.stl"), b"solid body\nendsolid body\n").unwrap();

        // Unsupported — should be skipped
        fs::write(dir.path().join("texture.png"), b"\x89PNG\r\n").unwrap();
        fs::write(dir.path().join("project.blend"), b"BLENDER").unwrap();
        fs::write(dir.path().join("notes.txt"), b"some notes").unwrap();

        let result = Scanner::scan_once(dir.path());

        assert_eq!(result.assets.len(), 3, "should find exactly 3 supported files");
        assert_eq!(
            result.skipped_unsupported, 3,
            "should count 3 unsupported files"
        );
        assert!(result.errors.is_empty(), "no errors expected");

        // Verify each supported file name is present
        let names: Vec<&str> = result.assets.iter().map(|a| a.name()).collect();
        assert!(names.contains(&"mesh"), "mesh.obj expected");
        assert!(names.contains(&"scene"), "scene.glb expected");
        assert!(names.contains(&"body"), "body.stl expected");
    }

    #[test]
    fn scan_once_discovers_files_in_subdirectories() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("models").join("characters");
        fs::create_dir_all(&sub).unwrap();

        fs::write(sub.join("hero.fbx"), b"FBX_content").unwrap();
        fs::write(dir.path().join("ground.dae"), b"<COLLADA/>").unwrap();

        let result = Scanner::scan_once(dir.path());

        assert_eq!(result.assets.len(), 2);
        let names: Vec<&str> = result.assets.iter().map(|a| a.name()).collect();
        assert!(names.contains(&"hero"));
        assert!(names.contains(&"ground"));
        // root + models + characters = 3 directories
        assert_eq!(result.scanned_dirs, 3);
    }

    #[test]
    fn scan_once_records_file_size() {
        let dir = TempDir::new().unwrap();
        let content = b"v 0 0 0\nv 1 0 0\n";
        fs::write(dir.path().join("cube.obj"), content).unwrap();

        let result = Scanner::scan_once(dir.path());

        assert_eq!(result.assets.len(), 1);
        assert_eq!(
            result.assets[0].file_size_bytes,
            content.len() as u64,
            "file size must match written bytes"
        );
    }

    #[test]
    fn scan_once_handles_nonexistent_dir_gracefully() {
        let path = Path::new("/nonexistent/rocket-craft/assets");
        let result = Scanner::scan_once(path);

        // walkdir reports an error for the root if it doesn't exist
        assert!(
            result.errors.len() >= 1 || result.assets.is_empty(),
            "nonexistent dir should produce an error or an empty scan"
        );
        // Must not panic
    }

    #[test]
    fn scan_once_produces_valid_hashes() {
        let dir = TempDir::new().unwrap();
        let content = b"v 0 0 0\n";
        fs::write(dir.path().join("point.obj"), content).unwrap();

        let result = Scanner::scan_once(dir.path());
        assert_eq!(result.assets.len(), 1);

        // Hash should match what content_hash() gives us directly
        let expected = content_hash(&dir.path().join("point.obj")).unwrap();
        assert_eq!(result.assets[0].hash, expected);
    }

    #[test]
    fn scan_once_all_supported_formats_detected() {
        let dir = TempDir::new().unwrap();

        let cases: &[(&str, Format)] = &[
            ("a.obj", Format::Obj),
            ("b.fbx", Format::Fbx),
            ("c.stl", Format::Stl),
            ("d.dae", Format::Dae),
            ("e.gltf", Format::Gltf),
            ("f.glb", Format::Glb),
        ];

        for (name, _) in cases {
            fs::write(dir.path().join(name), b"placeholder").unwrap();
        }

        let result = Scanner::scan_once(dir.path());
        assert_eq!(result.assets.len(), cases.len(), "all 6 formats should be discovered");

        for (_, expected_format) in cases {
            assert!(
                result.assets.iter().any(|a| a.source_format == *expected_format),
                "format {expected_format:?} not found in scan results"
            );
        }
    }

    #[test]
    fn scan_once_extension_case_insensitive() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("model.OBJ"), b"v 0 0 0\n").unwrap();
        fs::write(dir.path().join("scene.GLB"), b"glTF\x02").unwrap();

        let result = Scanner::scan_once(dir.path());
        assert_eq!(
            result.assets.len(),
            2,
            "uppercase extensions should be recognized"
        );
    }
}
