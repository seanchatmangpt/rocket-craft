use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use crate::config::discover_python3;

const GIT_WRAPPER_DIR: &str = "/tmp/ubt-git-wrapper";

// ── emsdk Python discovery ────────────────────────────────────────────────────

/// Find the emsdk-bundled python3 binary inside the engine's HTML5 emsdk directory.
///
/// Path pattern: `<engine>/Engine/Platforms/HTML5/Build/emsdk/emsdk-*/python/<ver>/bin/python3`
/// This is the Python that HTML5Setup.sh activates. UBT reads `PYTHON` env var first,
/// so pointing it here ensures emcc.py runs under the emsdk interpreter.
pub fn discover_emsdk_python(engine_root: &Path) -> Option<PathBuf> {
    let emsdk_root = engine_root.join("Engine/Platforms/HTML5/Build/emsdk");
    walkdir::WalkDir::new(&emsdk_root)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| {
            let name = e.file_name().to_string_lossy();
            name == "python3"
                && e.path()
                    .parent()
                    .map(|p| p.ends_with("bin"))
                    .unwrap_or(false)
                && e.path().is_file()
        })
        .map(|e| e.into_path())
}

// ── WASM magic bytes ─────────────────────────────────────────────────────────
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

// Real UE4 HTML5 packages are ~50–250 MB.  Anything below this threshold is
// almost certainly a stub or an empty placeholder.
const MIN_REAL_WASM_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

// ── Package verification ──────────────────────────────────────────────────────

/// Verdict for a single file found in the HTML5 archive directory.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WasmVerdict {
    /// Real WASM file: correct magic bytes and above the minimum size threshold.
    Real { size_bytes: u64 },
    /// File has valid WASM magic but is suspiciously small (likely a stub).
    Stub { size_bytes: u64 },
    /// The first four bytes are not the WASM magic sequence.
    NotWasm { first_bytes: Vec<u8> },
    /// File could not be read.
    Unreadable { reason: String },
}

/// Result of verifying one `.wasm` file.
#[derive(Debug, Clone, Serialize)]
pub struct WasmFileReport {
    pub path: PathBuf,
    pub verdict: WasmVerdict,
}

/// Companion files expected alongside a real UE4 HTML5 package.
#[derive(Debug, Clone, Serialize)]
pub struct CompanionReport {
    pub has_js: bool,
    pub has_html: bool,
    pub has_data_or_pak: bool,
}

/// Full report produced by [`Html5PackageVerifier`].
#[derive(Debug, Clone, Serialize)]
pub struct Html5PackageReport {
    pub archive_dir: PathBuf,
    pub wasm_files: Vec<WasmFileReport>,
    pub companions: CompanionReport,
    /// True when at least one `.wasm` with `WasmVerdict::Real` was found AND
    /// all companion files are present.
    pub is_real_package: bool,
    /// True when the UI-input pointer-lock patch has been applied to the HTML files.
    pub ui_input_patched: bool,
    /// UUID of the `game_sessions` row opened for this cook run, if Supabase is configured.
    /// `None` when running offline. Set by [`Html5PackageVerifier::with_cook_session_id`].
    pub cook_session_id: Option<String>,
    /// Structured OCEL events parsed from the UAT cook log.
    /// When non-empty, these replace artifact-derived lifecycle stages in the receipt,
    /// giving pm4py real evidence of which cook stages executed (not just which files exist).
    pub cook_log_events: Vec<CookLogEvent>,
}

impl Html5PackageReport {
    /// Write this report as a JSON receipt next to the archive.
    ///
    /// The receipt file is `<archive_dir>/cook-receipt.json`. It includes the
    /// verdict, summary, wasm sizes, and an RFC 3339 timestamp so the cook
    /// result is inspectable without re-running verify.
    pub fn write_receipt(&self) -> Result<PathBuf> {
        use serde_json::json;
        let out = self.archive_dir.join("cook-receipt.json");
        let wasm_mb = self.wasm_files.iter().find_map(|f| {
            if let WasmVerdict::Real { size_bytes } = f.verdict {
                Some(size_bytes as f64 / 1_048_576.0)
            } else {
                None
            }
        });
        // RFC 3339 timestamp via SystemTime → seconds since UNIX epoch
        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let wasm_files_json: Vec<serde_json::Value> = self.wasm_files.iter().map(|f| {
            let (verdict_str, size_bytes) = match &f.verdict {
                WasmVerdict::Real { size_bytes } => ("real", *size_bytes),
                WasmVerdict::Stub { size_bytes } => ("stub", *size_bytes),
                WasmVerdict::NotWasm { .. } => ("not_wasm", 0),
                WasmVerdict::Unreadable { .. } => ("unreadable", 0),
            };
            json!({ "path": f.path.display().to_string(), "verdict": verdict_str, "size_bytes": size_bytes })
        }).collect();
        // Compute output_hash: BLAKE3 of WASM bytes for Gap 6 cook-to-game cross-check.
        let output_hash: Option<String> = self.wasm_files.iter()
            .find(|f| matches!(f.verdict, WasmVerdict::Real { .. }))
            .and_then(|f| std::fs::read(&f.path).ok())
            .map(|bytes| blake3::hash(&bytes).to_hex().to_string());
        let receipt = json!({
            "verdict": if self.is_real_package { "PASS" } else { "FAIL" },
            "summary": self.summary(),
            "is_real_package": self.is_real_package,
            "output_hash": output_hash,
            "wasm_mb": wasm_mb,
            "wasm_files": wasm_files_json,
            "companions": {
                "has_js": self.companions.has_js,
                "has_html": self.companions.has_html,
                "has_data_or_pak": self.companions.has_data_or_pak,
            },
            "archive_dir": self.archive_dir.display().to_string(),
            "verified_at_unix_secs": timestamp_secs,
            "ui_input_patched": self.ui_input_patched,
        });
        let json_str = serde_json::to_string_pretty(&receipt)
            .context("failed to serialise cook receipt")?;
        std::fs::write(&out, json_str)
            .with_context(|| format!("failed to write cook receipt to {}", out.display()))?;
        Ok(out)
    }

    /// Build a `CookReceipt` for pushing to Supabase.
    ///
    /// Returns the receipt value so the caller can log or inspect it before
    /// the async push.  Uses a BLAKE3 hex digest of the summary string as
    /// the receipt hash (lightweight, deterministic, faster than SHA-256).
    pub fn as_supabase_receipt(&self) -> crate::supabase::CookReceipt {
        use std::collections::HashMap;

        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let wasm_mb = self.wasm_files.iter().find_map(|f| {
            if let WasmVerdict::Real { size_bytes } = f.verdict {
                Some(size_bytes as f64 / 1_048_576.0)
            } else {
                None
            }
        });

        // BLAKE3 receipt hash — faster and safer than SHA-256; same 64-char hex output.
        let hash_input = format!(
            "{}|{}|{}",
            self.summary(),
            timestamp_secs,
            self.archive_dir.display()
        );
        let receipt_hash: String = blake3::hash(hash_input.as_bytes()).to_hex().to_string();

        // output_hash: BLAKE3 of the actual WASM binary bytes (Gap 6 cook-to-game cross-check).
        // If the browser loads a different binary than the one cooked, these hashes diverge.
        let output_hash: Option<String> = self.wasm_files.iter()
            .find(|f| matches!(f.verdict, WasmVerdict::Real { .. }))
            .and_then(|f| std::fs::read(&f.path).ok())
            .map(|bytes| blake3::hash(&bytes).to_hex().to_string());

        // OCEL lifecycle: prefer UAT log-derived events (richer evidence); fall back to
        // artifact-derived stages when no log events are available (offline verify).
        let lifecycle: Vec<String> = if !self.cook_log_events.is_empty() {
            // Log-derived: real cook evidence from UAT log patterns
            let mut lc: Vec<String> = self.cook_log_events.iter()
                .map(|e| e.activity.clone())
                .collect();
            // Append verify-time stages that the log can't observe (they happen after UAT exits)
            if self.ui_input_patched && !lc.contains(&"UiInputPatched".to_string()) {
                lc.push("UiInputPatched".to_string());
            }
            if self.is_real_package && !lc.contains(&"PackageVerified".to_string()) {
                lc.push("PackageVerified".to_string());
            }
            lc
        } else {
            // Artifact-derived fallback (offline verify / no log available)
            let mut lc = vec!["CookStarted".to_string()];
            if self.wasm_files.iter().any(|f| matches!(f.verdict, WasmVerdict::Real { .. })) {
                lc.push("WasmPackaged".to_string());
            }
            if self.companions.has_js { lc.push("JsEmitted".to_string()); }
            if self.companions.has_html { lc.push("HtmlEmitted".to_string()); }
            if self.companions.has_data_or_pak { lc.push("DataPakStaged".to_string()); }
            if self.ui_input_patched { lc.push("UiInputPatched".to_string()); }
            if self.is_real_package { lc.push("PackageVerified".to_string()); }
            lc
        };

        let ocel_event_count = lifecycle.len() as u32;

        let mut payload: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(mb) = wasm_mb { payload.insert("wasm_mb".into(), serde_json::json!(mb)); }
        payload.insert("archive_dir".into(), serde_json::json!(self.archive_dir.display().to_string()));
        payload.insert("summary".into(), serde_json::json!(self.summary()));

        crate::supabase::CookReceipt {
            session_id: self.cook_session_id.clone(),
            verdict: if self.is_real_package { "PASS".into() } else { "FAIL".into() },
            milestone: "HTML5CookVerify".into(),
            ocel_lifecycle: lifecycle,
            ocel_event_count,
            engine_source: "rocket_cli".into(),
            receipt_hash,
            output_hash,
            proven_at: format!("{timestamp_secs}"),
            payload,
        }
    }

    /// Push this cook report to Supabase as a `game_receipt` + individual `ocel_events`.
    ///
    /// The individual event rows give pm4py the timestamped evidence needed for
    /// process discovery and conformance checking against the declared cook pipeline.
    ///
    /// Non-fatal when Supabase is unreachable (offline cook). Partial failure on
    /// `push_ocel_events` is logged but does not fail the cook.
    pub async fn push_to_supabase(&self, svc: &crate::supabase::SupabaseService) -> Result<()> {
        let receipt = self.as_supabase_receipt();

        // Build OCEL event rows using ChainedOcelEmitter — single source of truth
        // for the BLAKE3 chain formula, shared with session-seed and browser clients.
        let cook_obj = format!("cook:{}", self.archive_dir.display());
        // proven_at is RFC3339 — parse to epoch ms (not u64 directly).
        let base_ms = chrono::DateTime::parse_from_rfc3339(&receipt.proven_at)
            .map(|dt| dt.timestamp_millis() as u64)
            .unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0)
            });
        let mut emitter = crate::supabase::ChainedOcelEmitter::new(
            self.cook_session_id.clone(),
            cook_obj,
        );
        for (i, activity) in receipt.ocel_lifecycle.iter().enumerate() {
            let timestamp_ms = base_ms + (i as u64 * 1_000);
            emitter.emit(
                activity.as_str(),
                timestamp_ms,
                serde_json::json!({ "stage_index": i }),
            );
        }
        let events = emitter.into_events();

        // Push OCEL events first so the session has evidence before the receipt lands.
        // Non-fatal on failure — the receipt is the authoritative record.
        if let Err(e) = svc.push_ocel_events(&events).await {
            eprintln!("[warn] push_ocel_events failed (non-fatal): {e}");
        }

        // Push receipt through the Nuxt proof gate (falls back to direct REST if Nuxt is down).
        svc.push_cook_receipt(&receipt, None).await?;

        // Close the game_session row now that we have a receipt hash.
        if let Some(sid) = &self.cook_session_id {
            if let Err(e) = svc.close_cook_session(
                sid,
                &receipt.verdict,
                Some(&receipt.receipt_hash),
            ).await {
                eprintln!("[warn] close_cook_session failed (non-fatal): {e}");
            }
        }
        Ok(())
    }

    /// One-liner summary suitable for CLI output.
    pub fn summary(&self) -> String {
        if self.is_real_package {
            let size = self.wasm_files.iter().find_map(|f| {
                if let WasmVerdict::Real { size_bytes } = f.verdict {
                    Some(size_bytes)
                } else {
                    None
                }
            }).unwrap_or(0);
            format!(
                "REAL package — {:.1} MB WASM, js={}, html={}, data={}",
                size as f64 / 1_048_576.0,
                self.companions.has_js,
                self.companions.has_html,
                self.companions.has_data_or_pak,
            )
        } else if self.wasm_files.is_empty() {
            format!("NO .wasm found in {}", self.archive_dir.display())
        } else {
            let stub_count = self.wasm_files.iter().filter(|f| {
                matches!(f.verdict, WasmVerdict::Stub { .. })
            }).count();
            format!(
                "STUB package — {} wasm file(s), {} stub(s)",
                self.wasm_files.len(), stub_count
            )
        }
    }
}

/// Verifies that an HTML5 archive directory contains a real UE4 package.
///
/// Checks:
/// 1. Presence of at least one `.wasm` file with valid magic bytes.
/// 2. Size exceeds [`MIN_REAL_WASM_BYTES`] (10 MB stub detection threshold).
/// 3. Companion files: `.js`, `.html`, and `.data`/`.pak`.
pub struct Html5PackageVerifier {
    pub archive_dir: PathBuf,
    /// Override the minimum size threshold (default: 10 MB).
    pub min_wasm_bytes: u64,
    /// Optional `game_sessions` UUID to attach to OCEL events pushed to Supabase.
    pub cook_session_id: Option<String>,
}

impl Html5PackageVerifier {
    pub fn new(archive_dir: impl Into<PathBuf>) -> Self {
        Self {
            archive_dir: archive_dir.into(),
            min_wasm_bytes: MIN_REAL_WASM_BYTES,
            cook_session_id: None,
        }
    }

    pub fn with_min_wasm_bytes(mut self, bytes: u64) -> Self {
        self.min_wasm_bytes = bytes;
        self
    }

    /// Attach a pre-created `game_sessions` UUID so the resulting report's
    /// OCEL events are linked to a verifiable session in Supabase.
    ///
    /// When set, [`Html5PackageReport::push_to_supabase`] tags all OCEL event
    /// rows with this session_id so `verify_event_chain(session_id)` can prove
    /// the cook pipeline ran its declared lifecycle in lawful order.
    pub fn with_cook_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.cook_session_id = Some(session_id.into());
        self
    }

    /// Walk `archive_dir` recursively and verify all `.wasm` files.
    pub fn verify(&self) -> Result<Html5PackageReport> {
        if !self.archive_dir.exists() {
            bail!(
                "HTML5 archive directory not found: {}",
                self.archive_dir.display()
            );
        }

        let mut wasm_files = Vec::new();
        let mut has_js = false;
        let mut has_html = false;
        let mut has_data_or_pak = false;

        for entry in walkdir::WalkDir::new(&self.archive_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                "wasm" => wasm_files.push(self.check_wasm(path)),
                "js" => has_js = true,
                "html" | "htm" => has_html = true,
                "data" | "pak" => has_data_or_pak = true,
                _ => {}
            }
        }

        let has_real_wasm = wasm_files.iter().any(|f| {
            matches!(f.verdict, WasmVerdict::Real { .. })
        });

        let companions = CompanionReport { has_js, has_html, has_data_or_pak };
        let is_real_package = has_real_wasm && has_js && has_html;
        let ui_input_patched = check_html_ui_patch_applied(&self.archive_dir);

        Ok(Html5PackageReport {
            archive_dir: self.archive_dir.clone(),
            wasm_files,
            companions,
            is_real_package,
            ui_input_patched,
            cook_session_id: self.cook_session_id.clone(),
            cook_log_events: vec![],
        })
    }

    fn check_wasm(&self, path: &Path) -> WasmFileReport {
        let verdict = match std::fs::read(path) {
            Err(e) => WasmVerdict::Unreadable { reason: e.to_string() },
            Ok(bytes) => {
                if bytes.len() < 4 {
                    WasmVerdict::NotWasm { first_bytes: bytes }
                } else if bytes[..4] != WASM_MAGIC {
                    WasmVerdict::NotWasm { first_bytes: bytes[..4].to_vec() }
                } else {
                    let size_bytes = bytes.len() as u64;
                    if size_bytes >= self.min_wasm_bytes {
                        WasmVerdict::Real { size_bytes }
                    } else {
                        WasmVerdict::Stub { size_bytes }
                    }
                }
            }
        };
        WasmFileReport { path: path.to_path_buf(), verdict }
    }
}

pub struct Html5Cook {
    pub engine_root: PathBuf,
    pub project: PathBuf,
    pub archive_dir: PathBuf,
    pub client_config: String,
    /// Minimum free disk space in GB required before a cook is started (default 50 GB).
    pub min_disk_gb: f64,
}

impl Html5Cook {
    pub fn new(
        engine_root: impl Into<PathBuf>,
        project: impl Into<PathBuf>,
        archive_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            engine_root: engine_root.into(),
            project: project.into(),
            archive_dir: archive_dir.into(),
            client_config: "Development".into(),
            min_disk_gb: 50.0,
        }
    }

    /// Override the client config (defaults to `"Development"`).
    /// Use `"Shipping"` for release packages.
    pub fn with_client_config(mut self, config: impl Into<String>) -> Self {
        self.client_config = config.into();
        self
    }

    /// Override the minimum required free disk space (defaults to 50 GB).
    pub fn with_min_disk_gb(mut self, gb: f64) -> Self {
        self.min_disk_gb = gb;
        self
    }

    /// Check that all required environment components exist before starting a cook.
    ///
    /// Returns a list of human-readable failure reasons. An empty Vec means the
    /// environment is ready. Call this before `run()` to surface problems early
    /// without waiting for a long UAT build to fail mid-way.
    pub fn preflight_check(&self) -> Vec<String> {
        let mut failures = Vec::new();

        let run_uat = self.engine_root.join("Engine/Build/BatchFiles/RunUAT.sh");
        if !run_uat.exists() {
            failures.push(format!("RunUAT.sh not found: {}", run_uat.display()));
        }

        if !self.project.exists() {
            failures.push(format!(".uproject not found: {}", self.project.display()));
        }

        let emsdk = self.engine_root.join("Engine/Platforms/HTML5/Build/emsdk");
        if !emsdk.exists() {
            failures.push(format!("HTML5 emsdk not found at {}: run HTML5Setup.sh", emsdk.display()));
        }

        // Python 3 must be discoverable — HTML5 UAT requires it.
        let python_ok = discover_emsdk_python(&self.engine_root)
            .or_else(|| {
                ["python3", "python"]
                    .iter()
                    .find_map(|cmd| {
                        std::process::Command::new(cmd)
                            .arg("--version")
                            .output()
                            .ok()
                            .filter(|o| o.status.success())
                            .map(|_| std::path::PathBuf::from(cmd))
                    })
            })
            .is_some();
        if !python_ok {
            failures.push("Python 3 not found in PATH or emsdk; HTML5 UAT requires it".into());
        }

        // Disk space: UE4 HTML5 cook + intermediate files need ~50 GB free.
        let check_path = if self.engine_root.exists() {
            self.engine_root.as_path()
        } else {
            Path::new("/")
        };
        if let Some(free_gb) = available_disk_gb(check_path) {
            if free_gb < self.min_disk_gb {
                let oclnr_hint = if std::path::Path::new(&format!(
                    "{}/osx-clnr/target/release/oclnr",
                    std::env::var("HOME").unwrap_or_default()
                )).exists() {
                    " — run ~/osx-clnr/target/release/oclnr to free space"
                } else {
                    ""
                };
                failures.push(format!(
                    "insufficient disk space: {:.1} GB free, need ≥{:.0} GB (UE4 HTML5 cook requires ~50 GB){oclnr_hint}",
                    free_gb, self.min_disk_gb
                ));
            }
        }

        failures
    }

    /// Return the UAT command-line arguments that `run()` would execute, without running them.
    ///
    /// Useful for `--dry-run` mode: print the exact command so the user can verify flags
    /// before committing to a 30+ minute build. The returned vec is `[executable, arg1, ...]`.
    pub fn command_args(&self) -> Vec<String> {
        let run_uat = self.engine_root.join("Engine/Build/BatchFiles/RunUAT.sh");
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let cook_log = format!("{home}/ue4-cook-latest.log");
        vec![
            "arch".into(),
            "-x86_64".into(),
            "/bin/bash".into(),
            run_uat.to_string_lossy().into_owned(),
            "BuildCookRun".into(),
            format!("-project={}", self.project.display()),
            "-noP4".into(),
            "-platform=HTML5".into(),
            format!("-clientconfig={}", self.client_config),
            "-cook".into(),
            "-build".into(),
            "-stage".into(),
            "-pak".into(),
            "-package".into(),
            "-archive".into(),
            "-IgnoreCookErrors".into(),
            format!("-archivedirectory={}", self.archive_dir.display()),
            format!("-log={cook_log}"),
        ]
    }

    /// Run the cook and verify the output is a real package.
    /// Returns the package report; call `report.is_real_package` to confirm success.
    pub fn run_and_verify(&self) -> Result<Html5PackageReport> {
        self.run()?;
        Html5PackageVerifier::new(&self.archive_dir)
            .verify()
    }

    pub fn run(&self) -> Result<()> {
        ensure_git_wrapper()?;

        let run_uat = self.engine_root.join("Engine/Build/BatchFiles/RunUAT.sh");
        if !run_uat.exists() {
            bail!("RunUAT.sh not found at {}", run_uat.display());
        }

        // Prefer the emsdk-bundled Python (the same one HTML5Setup.sh activates via
        // `emsdk activate`). UBT reads the PYTHON env var first — pointing it at the
        // emsdk Python ensures emcc.py runs under the correct interpreter.
        // Fall back to system python3 if the emsdk Python is not yet installed.
        let python3 = discover_emsdk_python(&self.engine_root)
            .or_else(|| discover_python3())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Python 3 not found. Install python3 or run 'rocket html5 setup' to install the emsdk Python."
                )
            })?;

        // UHT computes CURRENT_FILE_ID relative to the parent of the project directory.
        // If that parent dir starts with a digit (e.g. versions/4.27.0/), all generated
        // macros start with a digit — invalid C identifier. Create a symlink with a
        // letter-starting name so UHT sees a valid base path.
        let project = ensure_letter_start_symlink(&self.project)?;

        let path_with_wrapper = prepend_to_path(GIT_WRAPPER_DIR);

        // Write cook log to a predictable path so 'rocket html5 log' always finds it.
        // UE4's BuildCookRun also writes ~/ue4-cook<N>.log; the -log flag adds a copy
        // at a known location without disabling the default log.
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let cook_log = format!("{home}/ue4-cook-latest.log");

        let status = Command::new("arch")
            .args(["-x86_64", "/bin/bash"])
            .arg(&run_uat)
            .args([
                "BuildCookRun",
                &format!("-project={}", project.display()),
                "-noP4",
                "-platform=HTML5",
                &format!("-clientconfig={}", self.client_config),
                "-cook",
                "-build",
                "-stage",
                "-pak",
                "-package",
                "-archive",
                "-IgnoreCookErrors",
                &format!("-archivedirectory={}", self.archive_dir.display()),
                &format!("-log={}", cook_log),
            ])
            .env("PYTHON", python3.to_str().unwrap_or("python3"))
            .env("PATH", &path_with_wrapper)
            .env("MONO_THREADS_SUSPEND", "preemptive")
            .env("MONO_GC_PARAMS", "nursery-size=64m")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to spawn RunUAT.sh")?;

        check_exit(status, "RunUAT.sh BuildCookRun")
    }
}

pub struct Html5Setup {
    pub engine_root: PathBuf,
}

impl Html5Setup {
    pub fn new(engine_root: impl Into<PathBuf>) -> Self {
        Self {
            engine_root: engine_root.into(),
        }
    }

    pub fn run(&self) -> Result<()> {
        let setup_sh = self
            .engine_root
            .join("Engine/Platforms/HTML5/HTML5Setup.sh");
        if !setup_sh.exists() {
            bail!("HTML5Setup.sh not found at {}", setup_sh.display());
        }

        ensure_metal_toolchain()?;

        // chmod +x all HTML5 shell scripts (emsdk tarball sometimes loses execute bits)
        let _ = Command::new("find")
            .args([
                self.engine_root
                    .join("Engine/Platforms/HTML5")
                    .to_str()
                    .unwrap(),
                "-name",
                "*.sh",
                "-exec",
                "chmod",
                "+x",
                "{}",
                ";",
            ])
            .status();

        let python3 = discover_python3().ok_or_else(|| {
            anyhow::anyhow!(
                "Python 3 not found. Install python3 or set 'python3_path' in .rocket.json."
            )
        })?;

        let path_with_wrapper = prepend_to_path(GIT_WRAPPER_DIR);

        let status = Command::new("/opt/homebrew/bin/bash")
            .arg(&setup_sh)
            .env("PYTHON", python3.to_str().unwrap_or("python3"))
            .env("PATH", &path_with_wrapper)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to spawn HTML5Setup.sh")?;

        check_exit(status, "HTML5Setup.sh")
    }
}

/// On macOS 15+, the Metal compiler ships as a separately downloadable Xcode component.
/// cmake's system compiler probe triggers a popup if it's absent; we gate on it here so
/// the error surfaces as a typed diagnostic before HTML5Setup.sh launches.
#[cfg(target_os = "macos")]
fn ensure_metal_toolchain() -> Result<()> {
    let probe = Command::new("xcrun")
        .args(["--find", "metal"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match probe {
        Ok(s) if s.success() => return Ok(()),
        _ => {}
    }

    eprintln!("Metal compiler not found — downloading Metal toolchain (requires internet)...");
    let status = Command::new("xcodebuild")
        .args(["-downloadComponent", "MetalToolchain"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn xcodebuild -downloadComponent MetalToolchain")?;

    if !status.success() {
        bail!(
            "Metal toolchain download failed (exit {:?}).\n\
             Run manually: xcodebuild -downloadComponent MetalToolchain",
            status.code()
        );
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn ensure_metal_toolchain() -> Result<()> {
    Ok(())
}

/// UHT derives CURRENT_FILE_ID from the PARENT of the .uproject directory.
/// If that parent starts with a digit (e.g. `versions/4.27.0/`), the generated
/// macro prefix is `4_27_0_Source_...` — invalid C identifier. This function
/// creates a sibling symlink with a letter-starting name if needed.
/// .NET Path.GetFullPath() does NOT resolve symlinks, so UHT sees the link name.
fn ensure_letter_start_symlink(project: &Path) -> Result<PathBuf> {
    let project_dir = project
        .parent()
        .ok_or_else(|| anyhow::anyhow!("project has no parent dir"))?;
    let parent = project_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("project dir has no parent"))?;
    let dir_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("project dir name is not valid UTF-8"))?;

    let starts_with_digit = dir_name
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);
    if !starts_with_digit {
        return Ok(project.to_path_buf());
    }

    // Build a letter-starting name: strip leading digits, prefix with project stem
    let project_stem = project
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Project");
    let suffix: String = dir_name.chars().filter(|c| c.is_alphanumeric()).collect();
    let link_name = format!("{project_stem}{suffix}");
    let link_path = parent.join(&link_name);

    if !link_path.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(project_dir, &link_path).with_context(|| {
            format!(
                "create symlink {} → {}",
                link_path.display(),
                project_dir.display()
            )
        })?;
        #[cfg(not(unix))]
        bail!("symlink creation is not supported on this platform — rename the project directory to start with a letter");
    }

    let uproject_name = project
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("project has no file name"))?;
    Ok(link_path.join(uproject_name))
}

/// Check whether the UI-input pointer-lock patch has been applied to any HTML file
/// in the given archive directory (looks for the sentinel comment).
pub fn check_html_ui_patch_applied(archive_dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(archive_dir) else { return false; };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
        .any(|e| {
            std::fs::read_to_string(e.path())
                .map(|s| s.contains("rocket-html5: suppress"))
                .unwrap_or(false)
        })
}

/// Check available disk space on the volume containing `path`.
/// Returns `None` if the check is not supported on this platform or path doesn't exist.
pub fn available_disk_gb(path: &Path) -> Option<f64> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        let c_path = CString::new(path.to_str()?).ok()?;
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
        if ret != 0 { return None; }
        let available = stat.f_bavail as u64 * stat.f_frsize as u64;
        Some(available as f64 / 1_073_741_824.0) // bytes → GB
    }
    #[cfg(not(unix))]
    { let _ = path; None }
}

fn ensure_git_wrapper() -> Result<()> {
    let wrapper = Path::new(GIT_WRAPPER_DIR).join("git");
    if !wrapper.exists() {
        std::fs::create_dir_all(GIT_WRAPPER_DIR).context("create git wrapper dir")?;
        std::fs::write(&wrapper, "#!/bin/sh\nexit 0\n").context("write git wrapper")?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
            .context("chmod git wrapper")?;
    }
    Ok(())
}

fn prepend_to_path(dir: &str) -> String {
    match std::env::var("PATH") {
        Ok(p) => format!("{dir}:{p}"),
        Err(_) => dir.to_string(),
    }
}

fn check_exit(status: ExitStatus, label: &str) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("{label} failed with exit code {:?}", status.code())
    }
}

// ── CookLogParser ─────────────────────────────────────────────────────────────

/// A structured event extracted from a UE4/UAT cook log line.
#[derive(Debug, Clone, Serialize)]
pub struct CookLogEvent {
    /// OCEL 2.0 activity name (e.g. "CookPackageStarted", "PakComplete").
    pub activity: String,
    /// The raw log line that triggered this event.
    pub raw_line: String,
    /// Approximate epoch milliseconds (line offset from cook start × 1000).
    pub timestamp_ms: u64,
    /// Optional structured detail extracted from the log line.
    pub detail: Option<String>,
}

/// Pattern table: `(log substring, OCEL activity)`.
/// Order matters — first match wins. Each activity is emitted at most once
/// (first-seen wins; duplicates from UAT retry logic are dropped by `seen_activities`).
const COOK_PATTERNS: &[(&str, &str)] = &[
    // ── UAT entry / setup ─────────────────────────────────────────────────
    ("BuildCookRun",                       "CookStarted"),
    ("HTML5Setup",                         "HTML5SetupStarted"),
    ("HTML5Setup.sh",                      "HTML5SetupStarted"),   // alternate form
    ("Success!",                           "HTML5SetupComplete"),
    // ── Cook phase ────────────────────────────────────────────────────────
    ("LogCook: Display: Cooking package",  "PackageCooking"),
    ("LogCook: Display: Cook complete",    "CookComplete"),
    ("Total cook time",                    "CookComplete"),         // alternate form
    ("LogCook: Display: Finished cooking", "CookComplete"),
    // ── Shader compilation ────────────────────────────────────────────────
    ("LogShaderCompilers:",                "ShaderCompileStarted"),
    ("ShaderCompileWorker",               "ShaderCompileStarted"),
    ("Shaders compiled",                   "ShadersCompiled"),
    // ── Asset save phase ──────────────────────────────────────────────────
    ("LogSave: Display: Saving package",   "AssetSaveStarted"),
    ("LogSave: Display: Saving cooked",    "AssetSaveStarted"),
    // ── WASM / Emscripten compilation ─────────────────────────────────────
    ("LogHTML5PlatformEditor",             "WasmBuildStarted"),
    ("emcc",                               "EmscriptenInvoked"),
    ("wasm-opt",                           "WasmOptimized"),
    // ── Pak / staging ─────────────────────────────────────────────────────
    ("LogPak: Display: Collecting files",  "PakStarted"),
    ("LogPak: Display: Created pak file",  "PakComplete"),
    ("LogStageAndPackage",                 "StagingStarted"),
    ("Staging complete",                   "StagingComplete"),
    ("Archiving",                          "ArchiveStarted"),
    // ── Package finalisation ──────────────────────────────────────────────
    ("Packaging complete",                 "PackageComplete"),
    ("Package was created",                "PackageCreated"),
    ("BuildCookRun: Completed",            "CookFinished"),
    // ── Errors (last — only if no success pattern matched first) ──────────
    ("CookLog: Error:",                    "CookError"),
    ("Error: Error:",                      "CookError"),
    ("Error:",                             "CookError"),
    ("ERROR:",                             "CookError"),
    ("FAILED:",                            "CookFailed"),
    ("returned exit code",                 "CookFailed"),
    ("exception was thrown",               "CookFailed"),
];

/// Parse a UAT/UE4 cook log and extract structured OCEL lifecycle events.
///
/// Scans the log file sequentially. Each line is matched against `COOK_PATTERNS`
/// and when a match is found a `CookLogEvent` is emitted. Duplicate activities are
/// deduplicated so the caller gets a clean ordered lifecycle list (first-seen wins).
///
/// Timestamps are synthetic: `cook_start_ms + line_number * 100ms` — real timestamps
/// require log-line timestamp parsing which UAT does not always emit.
pub fn parse_cook_log(log_path: &Path, cook_start_ms: u64) -> Vec<CookLogEvent> {
    use std::io::{BufRead, BufReader};
    use std::collections::HashSet;
    let Ok(file) = std::fs::File::open(log_path) else { return vec![]; };
    let reader = BufReader::new(file);
    let mut seen: HashSet<String> = HashSet::new();
    let mut events: Vec<CookLogEvent> = Vec::new();
    let mut line_no: u64 = 0;

    for line in reader.lines().filter_map(|l| l.ok()) {
        line_no += 1;
        for (pattern, activity) in COOK_PATTERNS {
            if line.contains(pattern) && seen.insert((*activity).to_string()) {
                let detail = extract_cook_detail(&line, activity);
                events.push(CookLogEvent {
                    activity: activity.to_string(),
                    raw_line: line.clone(),
                    timestamp_ms: cook_start_ms + line_no * 100,
                    detail,
                });
                break;
            }
        }
    }
    events
}

/// Extract a meaningful detail string from certain log line types.
fn extract_cook_detail(line: &str, activity: &str) -> Option<String> {
    match activity {
        "PackageCooking" => {
            // "LogCook: Display: Cooking package: /Game/Maps/TestMap"
            line.split("Cooking package:").nth(1)
                .map(|s| s.trim().to_string())
        }
        "CookComplete" | "CookFinished" => {
            // "Total cook time was 1847.3s"
            line.split("Total cook time").nth(1)
                .map(|s| format!("Total cook time{}", s.trim()))
        }
        "PakComplete" => {
            line.split("Created pak file").nth(1)
                .map(|s| s.trim().to_string())
        }
        "CookError" | "CookFailed" => {
            Some(line.trim().to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── Html5PackageVerifier tests ────────────────────────────────────────────

    fn write_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn verifier_fails_on_missing_archive_dir() {
        let result = Html5PackageVerifier::new("/nonexistent/archive").verify();
        assert!(result.is_err());
    }

    #[test]
    fn verifier_no_files_gives_empty_report() {
        let dir = TempDir::new().unwrap();
        let report = Html5PackageVerifier::new(dir.path()).verify().unwrap();
        assert!(report.wasm_files.is_empty());
        assert!(!report.is_real_package);
        assert!(report.summary().contains("NO .wasm"));
    }

    #[test]
    fn verifier_detects_stub_wasm_below_threshold() {
        let dir = TempDir::new().unwrap();
        // Valid WASM magic, but only 8 bytes total
        let stub: Vec<u8> = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00].to_vec();
        write_file(dir.path(), "game.wasm", &stub);
        write_file(dir.path(), "game.js", b"// stub");
        write_file(dir.path(), "index.html", b"<html/>");

        let report = Html5PackageVerifier::new(dir.path())
            .with_min_wasm_bytes(1024)  // lower threshold so the test is predictable
            .verify().unwrap();

        assert!(!report.is_real_package);
        assert_eq!(report.wasm_files.len(), 1);
        assert!(matches!(report.wasm_files[0].verdict, WasmVerdict::Stub { .. }));
        assert!(report.summary().contains("STUB"));
    }

    #[test]
    fn verifier_rejects_non_wasm_magic() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "fake.wasm", b"not a real wasm file at all");
        let report = Html5PackageVerifier::new(dir.path()).verify().unwrap();
        assert!(matches!(
            report.wasm_files[0].verdict,
            WasmVerdict::NotWasm { .. }
        ));
        assert!(!report.is_real_package);
    }

    #[test]
    fn verifier_detects_real_package() {
        let dir = TempDir::new().unwrap();
        // Build a fake "real" WASM: magic bytes + enough padding to exceed threshold
        let threshold = 512_u64; // small threshold for test speed
        let mut real_wasm = vec![0x00u8, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        real_wasm.extend(vec![0u8; threshold as usize]);
        write_file(dir.path(), "game.wasm", &real_wasm);
        write_file(dir.path(), "game.js", b"(function(){})();");
        write_file(dir.path(), "index.html", b"<html><body></body></html>");
        write_file(dir.path(), "game.data", b"some data");

        let report = Html5PackageVerifier::new(dir.path())
            .with_min_wasm_bytes(threshold)
            .verify().unwrap();

        assert!(report.is_real_package);
        assert!(matches!(report.wasm_files[0].verdict, WasmVerdict::Real { .. }));
        assert!(report.companions.has_js);
        assert!(report.companions.has_html);
        assert!(report.companions.has_data_or_pak);
        assert!(report.summary().contains("REAL"));
    }

    #[test]
    fn verifier_real_requires_js_and_html() {
        let dir = TempDir::new().unwrap();
        let threshold = 100_u64;
        let mut wasm = vec![0x00u8, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        wasm.extend(vec![0u8; threshold as usize]);
        write_file(dir.path(), "game.wasm", &wasm);
        // Missing .js and .html

        let report = Html5PackageVerifier::new(dir.path())
            .with_min_wasm_bytes(threshold)
            .verify().unwrap();

        // WASM is real-sized but companions are missing → not a complete package
        assert!(!report.is_real_package);
    }

    #[test]
    fn wasm_verdict_real_size_is_correct() {
        let dir = TempDir::new().unwrap();
        let threshold = 50_u64;
        let mut wasm = vec![0x00u8, 0x61, 0x73, 0x6d];
        wasm.extend(vec![0u8; 100]); // 104 bytes total
        write_file(dir.path(), "g.wasm", &wasm);
        write_file(dir.path(), "g.js", b"");
        write_file(dir.path(), "g.html", b"");

        let report = Html5PackageVerifier::new(dir.path())
            .with_min_wasm_bytes(threshold)
            .verify().unwrap();

        if let WasmVerdict::Real { size_bytes } = report.wasm_files[0].verdict {
            assert_eq!(size_bytes, 104);
        } else {
            panic!("expected Real verdict, got {:?}", report.wasm_files[0].verdict);
        }
    }

    #[test]
    fn pak_file_counts_as_data_companion() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "game.pak", b"pak data");
        let report = Html5PackageVerifier::new(dir.path()).verify().unwrap();
        assert!(report.companions.has_data_or_pak);
    }

    fn make_uproject(base: &Path, dir_name: &str, project_name: &str) -> PathBuf {
        let proj_dir = base.join(dir_name);
        std::fs::create_dir_all(&proj_dir).unwrap();
        let uproject = proj_dir.join(format!("{project_name}.uproject"));
        std::fs::write(&uproject, "{}").unwrap();
        uproject
    }

    #[test]
    fn letter_start_dir_unchanged() {
        let dir = TempDir::new().unwrap();
        let uproject = make_uproject(dir.path(), "Brm427", "Brm");
        let result = ensure_letter_start_symlink(&uproject).unwrap();
        // Already starts with a letter — returned path should equal input
        assert_eq!(result, uproject);
    }

    #[test]
    fn digit_start_dir_gets_symlink() {
        let dir = TempDir::new().unwrap();
        let uproject = make_uproject(dir.path(), "4.27.0", "Brm");
        let result = ensure_letter_start_symlink(&uproject).unwrap();
        // Result must start with a letter
        let link_name = result
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            link_name.starts_with(|c: char| c.is_ascii_alphabetic()),
            "symlink name '{link_name}' should start with a letter"
        );
        // The .uproject file must be reachable through the symlink
        assert!(
            result.exists(),
            "uproject must be accessible via symlink at {}",
            result.display()
        );
    }

    #[test]
    fn symlink_idempotent() {
        let dir = TempDir::new().unwrap();
        let uproject = make_uproject(dir.path(), "4.27.0", "Brm");
        let r1 = ensure_letter_start_symlink(&uproject).unwrap();
        let r2 = ensure_letter_start_symlink(&uproject).unwrap();
        assert_eq!(r1, r2, "calling twice must return the same path");
    }

    #[test]
    fn git_wrapper_is_created() {
        let wrapper = Path::new(GIT_WRAPPER_DIR).join("git");
        ensure_git_wrapper().unwrap();
        assert!(
            wrapper.exists(),
            "git wrapper must exist at {}",
            wrapper.display()
        );
        let content = std::fs::read_to_string(&wrapper).unwrap();
        assert!(content.contains("exit 0"));
    }

    #[test]
    fn prepend_to_path_contains_dir() {
        let result = prepend_to_path("/my/custom/dir");
        assert!(
            result.starts_with("/my/custom/dir"),
            "prepended dir must be first in PATH"
        );
        assert!(result.contains(':'), "must contain a PATH separator");
    }

    #[test]
    fn write_receipt_creates_file_in_archive_dir() {
        let dir = TempDir::new().unwrap();
        // Plant a stub .wasm and a .js so the report isn't empty
        let wasm = dir.path().join("Game.wasm");
        let mut magic = WASM_MAGIC.to_vec();
        magic.extend(vec![0u8; MIN_REAL_WASM_BYTES as usize]); // make it "real"
        std::fs::write(&wasm, &magic).unwrap();
        std::fs::write(dir.path().join("Game.js"), "").unwrap();
        std::fs::write(dir.path().join("Game.html"), "").unwrap();

        let verifier = Html5PackageVerifier::new(dir.path());
        let report = verifier.verify().unwrap();
        assert!(report.is_real_package, "should be real with valid wasm + companions");

        let receipt_path = report.write_receipt().unwrap();
        assert!(receipt_path.exists());
        let content = std::fs::read_to_string(&receipt_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
        assert_eq!(v["verdict"], "PASS");
        assert!(v["wasm_mb"].as_f64().unwrap() > 0.0);
        // New fields added to receipt
        assert!(v["verified_at_unix_secs"].as_u64().unwrap() > 0, "timestamp must be > 0");
        assert!(v["wasm_files"].is_array(), "wasm_files array must be present");
        let wasm_arr = v["wasm_files"].as_array().unwrap();
        assert!(!wasm_arr.is_empty(), "wasm_files must not be empty");
        assert_eq!(wasm_arr[0]["verdict"], "real");
    }

    #[test]
    fn write_receipt_verdict_fail_for_stub() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("stub.wasm"), WASM_MAGIC).unwrap(); // 4 bytes = stub
        let report = Html5PackageVerifier::new(dir.path()).verify().unwrap();
        assert!(!report.is_real_package);
        let receipt_path = report.write_receipt().unwrap();
        let content = std::fs::read_to_string(&receipt_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["verdict"], "FAIL");
        // Timestamp and wasm_files are always present regardless of verdict
        assert!(v["verified_at_unix_secs"].as_u64().is_some(), "timestamp field must exist on FAIL receipt");
    }

    #[test]
    fn discover_emsdk_python_finds_python3_in_nested_structure() {
        let dir = TempDir::new().unwrap();
        // Mirror real path: <engine>/Engine/Platforms/HTML5/Build/emsdk/emsdk-5.0.7/python/3.13.3_64bit/bin/python3
        let bin_dir = dir.path()
            .join("Engine/Platforms/HTML5/Build/emsdk/emsdk-5.0.7/python/3.13.3_64bit/bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let python3 = bin_dir.join("python3");
        std::fs::write(&python3, "#!/bin/sh\nexec python3 \"$@\"").unwrap();
        // Make it executable so the file exists check passes
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&python3, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let found = discover_emsdk_python(dir.path());
        assert!(found.is_some(), "should find python3 in emsdk structure");
        let found = found.unwrap();
        assert!(found.ends_with("bin/python3"), "path should end with bin/python3, got: {}", found.display());
    }

    // ── Html5Cook::preflight_check ───────────────────────────────────────────

    #[test]
    fn preflight_check_reports_missing_runuat() {
        let dir = TempDir::new().unwrap();
        let uproject = dir.path().join("Game.uproject");
        std::fs::write(&uproject, "{}").unwrap();

        let cook = Html5Cook::new(dir.path(), &uproject, dir.path().join("archive"));
        let failures = cook.preflight_check();
        assert!(
            failures.iter().any(|f| f.contains("RunUAT.sh")),
            "missing RunUAT.sh must be reported: {failures:?}"
        );
    }

    #[test]
    fn preflight_check_reports_missing_uproject() {
        let dir = TempDir::new().unwrap();
        let cook = Html5Cook::new(dir.path(), dir.path().join("Ghost.uproject"), dir.path().join("archive"));
        let failures = cook.preflight_check();
        assert!(
            failures.iter().any(|f| f.contains("uproject")),
            "missing .uproject must be reported: {failures:?}"
        );
    }

    #[test]
    fn preflight_check_reports_missing_emsdk() {
        let dir = TempDir::new().unwrap();
        // Provide RunUAT.sh and .uproject so those checks pass
        let batch = dir.path().join("Engine/Build/BatchFiles");
        std::fs::create_dir_all(&batch).unwrap();
        std::fs::write(batch.join("RunUAT.sh"), "#!/bin/sh\n").unwrap();
        let uproject = dir.path().join("Game.uproject");
        std::fs::write(&uproject, "{}").unwrap();

        let cook = Html5Cook::new(dir.path(), &uproject, dir.path().join("archive"));
        let failures = cook.preflight_check();
        assert!(
            failures.iter().any(|f| f.contains("emsdk") || f.contains("HTML5Setup")),
            "missing emsdk must be reported: {failures:?}"
        );
    }

    #[test]
    fn preflight_check_passes_when_all_present() {
        let dir = TempDir::new().unwrap();
        // Simulate a minimal valid engine layout
        let batch = dir.path().join("Engine/Build/BatchFiles");
        std::fs::create_dir_all(&batch).unwrap();
        std::fs::write(batch.join("RunUAT.sh"), "#!/bin/sh\n").unwrap();
        let emsdk_dir = dir.path().join("Engine/Platforms/HTML5/Build/emsdk");
        std::fs::create_dir_all(&emsdk_dir).unwrap();
        // Put a fake python3 into the emsdk so discover_emsdk_python finds one
        let fake_python = emsdk_dir.join("python3");
        std::fs::write(&fake_python, "#!/bin/sh\necho Python 3.11.0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake_python, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let uproject = dir.path().join("Game.uproject");
        std::fs::write(&uproject, "{}").unwrap();

        let cook = Html5Cook::new(dir.path(), &uproject, dir.path().join("archive"));
        // Python check depends on PATH; we allow failures here as long as the first two checks pass.
        let failures = cook.preflight_check();
        let has_runuat_error = failures.iter().any(|f| f.contains("RunUAT.sh"));
        let has_uproject_error = failures.iter().any(|f| f.contains("uproject"));
        let has_emsdk_error = failures.iter().any(|f| f.contains("emsdk"));
        assert!(!has_runuat_error, "RunUAT.sh should not be reported missing");
        assert!(!has_uproject_error, ".uproject should not be reported missing");
        assert!(!has_emsdk_error, "emsdk should not be reported missing");
    }

    #[test]
    // ── Html5Cook::command_args ──────────────────────────────────────────────

    #[test]
    fn command_args_contains_builcookrun_and_platform() {
        let cook = Html5Cook::new("/fake/engine", "/fake/Game.uproject", "/tmp/archive");
        let args = cook.command_args();
        assert!(args.iter().any(|a| a == "BuildCookRun"), "must include BuildCookRun verb");
        assert!(args.iter().any(|a| a == "-platform=HTML5"), "must include -platform=HTML5");
    }

    #[test]
    fn command_args_includes_configured_client_config() {
        let cook = Html5Cook::new("/engine", "/g.uproject", "/archive")
            .with_client_config("Shipping");
        let args = cook.command_args();
        assert!(args.iter().any(|a| a == "-clientconfig=Shipping"), "must include -clientconfig=Shipping");
    }

    #[test]
    fn command_args_includes_archive_dir() {
        let cook = Html5Cook::new("/engine", "/g.uproject", "/my/archive/dir");
        let args = cook.command_args();
        assert!(args.iter().any(|a| a.contains("/my/archive/dir")), "archive dir must appear in args");
    }

    #[test]
    fn command_args_includes_runuat_path() {
        let cook = Html5Cook::new("/my/engine", "/g.uproject", "/archive");
        let args = cook.command_args();
        assert!(args.iter().any(|a| a.contains("RunUAT.sh")), "RunUAT.sh path must appear in args");
        assert!(args.iter().any(|a| a.contains("/my/engine")), "engine root must appear in RunUAT path");
    }

    fn discover_emsdk_python_returns_none_when_emsdk_absent() {
        let dir = TempDir::new().unwrap();
        let found = discover_emsdk_python(dir.path());
        assert!(found.is_none(), "should return None when no emsdk directory exists");
    }

    // ── ensure_letter_start_symlink ──────────────────────────────────────────

    #[test]
    fn letter_start_dir_is_returned_unchanged() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("MyProject");
        std::fs::create_dir_all(&project_dir).unwrap();
        let uproject = project_dir.join("MyProject.uproject");
        std::fs::write(&uproject, "{}").unwrap();

        let result = ensure_letter_start_symlink(&uproject).unwrap();
        assert_eq!(result, uproject, "letter-starting dir must pass through unchanged");
    }

    #[cfg(unix)]
    #[test]
    fn digit_start_dir_creates_symlink_with_letter_prefix() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("4.27.0");
        std::fs::create_dir_all(&project_dir).unwrap();
        let uproject = project_dir.join("Brm.uproject");
        std::fs::write(&uproject, "{}").unwrap();

        let result = ensure_letter_start_symlink(&uproject).unwrap();

        // The returned path must start with a letter
        let link_stem = result.parent().unwrap().file_name().unwrap().to_str().unwrap();
        assert!(
            link_stem.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false),
            "symlink directory name must start with a letter, got: {}",
            link_stem
        );
        // The symlink target directory must exist
        assert!(result.parent().unwrap().exists(), "symlink must exist on disk");
    }

    // ── check_html_ui_patch_applied ───────────────────────────────────────────

    #[test]
    fn ui_patch_detected_when_sentinel_present() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Game.html"),
            "<html><body><!-- rocket-html5: suppress pointer lock --></body></html>",
        ).unwrap();
        assert!(check_html_ui_patch_applied(dir.path()), "sentinel must be detected");
    }

    #[test]
    fn ui_patch_not_detected_on_unpatched_html() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Game.html"), "<html><body></body></html>").unwrap();
        assert!(!check_html_ui_patch_applied(dir.path()), "unpached html must return false");
    }

    #[test]
    fn ui_patch_not_detected_on_empty_dir() {
        let dir = TempDir::new().unwrap();
        assert!(!check_html_ui_patch_applied(dir.path()));
    }

    #[test]
    fn verifier_report_includes_ui_input_patched_field() {
        let dir = TempDir::new().unwrap();
        // patched html
        std::fs::write(
            dir.path().join("Game.html"),
            "<html><!-- rocket-html5: suppress --></html>",
        ).unwrap();
        let report = Html5PackageVerifier::new(dir.path()).verify().unwrap();
        assert!(report.ui_input_patched, "report must reflect patched html");
    }

    // ── available_disk_gb ─────────────────────────────────────────────────────

    #[test]
    #[cfg(unix)]
    fn available_disk_gb_returns_positive_for_root() {
        let gb = available_disk_gb(Path::new("/"));
        assert!(gb.is_some(), "should return Some on unix for /");
        assert!(gb.unwrap() > 0.0, "free space must be positive");
    }

    #[test]
    fn available_disk_gb_returns_none_for_missing_path() {
        let gb = available_disk_gb(Path::new("/nonexistent/path/that/does/not/exist"));
        // On unix, statvfs fails for nonexistent paths
        #[cfg(unix)]
        assert!(gb.is_none(), "nonexistent path must return None on unix");
    }

    #[test]
    fn preflight_check_reports_disk_shortage_when_threshold_is_enormous() {
        let dir = TempDir::new().unwrap();
        // Set min_disk_gb to an absurdly large number to guarantee the check fires
        let cook = Html5Cook::new(dir.path(), dir.path().join("G.uproject"), dir.path().join("a"))
            .with_min_disk_gb(999_999.0);
        let failures = cook.preflight_check();
        assert!(
            failures.iter().any(|f| f.contains("disk") || f.contains("GB")),
            "disk check must fire with absurd threshold: {failures:?}"
        );
    }

    #[test]
    fn with_min_disk_gb_builder_sets_field() {
        let cook = Html5Cook::new("/e", "/p.uproject", "/a").with_min_disk_gb(100.0);
        assert_eq!(cook.min_disk_gb, 100.0);
    }

    // ── as_supabase_receipt tests ─────────────────────────────────────────────

    fn make_real_report(dir: &TempDir) -> Html5PackageReport {
        let mut wasm = [0u8; 4]; wasm.copy_from_slice(&WASM_MAGIC);
        let mut content = wasm.to_vec();
        content.extend(vec![0u8; 20 * 1024 * 1024]); // 20 MB real wasm
        write_file(dir.path(), "Brm.wasm", &content);
        write_file(dir.path(), "Brm.js", b"Module={};");
        write_file(dir.path(), "Brm.html", b"<html><body><canvas></canvas></body></html>");
        write_file(dir.path(), "Brm.pak", b"UE4pak");
        Html5PackageVerifier::new(dir.path()).verify().unwrap()
    }

    #[test]
    fn as_supabase_receipt_pass_for_real_package() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        assert_eq!(receipt.verdict, "PASS");
        assert_eq!(receipt.engine_source, "rocket_cli");
        assert_eq!(receipt.milestone, "HTML5CookVerify");
    }

    #[test]
    fn as_supabase_receipt_lifecycle_starts_with_cook_started() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        assert_eq!(receipt.ocel_lifecycle[0], "CookStarted");
        assert!(receipt.ocel_lifecycle.contains(&"WasmPackaged".to_string()));
        assert!(receipt.ocel_lifecycle.contains(&"PackageVerified".to_string()));
    }

    #[test]
    fn as_supabase_receipt_event_count_matches_lifecycle_len() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        assert_eq!(receipt.ocel_event_count as usize, receipt.ocel_lifecycle.len());
    }

    #[test]
    fn as_supabase_receipt_fail_for_stub_wasm() {
        let dir = TempDir::new().unwrap();
        // 4-byte stub — below MIN_REAL_WASM_BYTES
        write_file(dir.path(), "Brm.wasm", &WASM_MAGIC);
        write_file(dir.path(), "Brm.js", b"Module={};");
        write_file(dir.path(), "Brm.html", b"<html></html>");
        let report = Html5PackageVerifier::new(dir.path()).verify().unwrap();
        let receipt = report.as_supabase_receipt();
        assert_eq!(receipt.verdict, "FAIL");
        assert!(!receipt.ocel_lifecycle.contains(&"WasmPackaged".to_string()));
    }

    #[test]
    fn as_supabase_receipt_hash_is_stable_hex_string() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        // Hash is a 64-char lowercase BLAKE3 hex string.
        assert_eq!(receipt.receipt_hash.len(), 64);
        assert!(receipt.receipt_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── push_to_supabase OCEL event construction ──────────────────────────────

    fn ocel_events_from_lifecycle(
        lifecycle: &[String],
        base_ms: u64,
        archive_dir: &std::path::Path,
    ) -> Vec<crate::supabase::OcelEventRow> {
        let mut prev_hash: Option<String> = None;
        lifecycle.iter().enumerate().map(|(i, activity)| {
            let timestamp_ms = base_ms + (i as u64 * 1_000);
            let chain_input = format!(
                "{}{}{}",
                prev_hash.as_deref().unwrap_or("genesis"),
                activity,
                timestamp_ms
            );
            let event_hash = format!("{:016x}", chain_input.bytes().fold(
                0xcbf29ce484222325u64,
                |acc, b| acc.wrapping_mul(0x100000001b3).wrapping_add(b as u64),
            ));
            let row = crate::supabase::OcelEventRow {
                session_id: None,
                activity: activity.clone(),
                timestamp_ms,
                object_refs: vec![format!("cook:{}", archive_dir.display())],
                attributes: serde_json::json!({ "stage_index": i }),
                prev_hash: prev_hash.clone(),
                event_hash: event_hash.clone(),
                seq: i as u32,
            };
            prev_hash = Some(event_hash);
            row
        }).collect()
    }

    #[test]
    fn ocel_events_seq_matches_lifecycle_index() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        let events = ocel_events_from_lifecycle(&receipt.ocel_lifecycle, 1_000_000, dir.path());
        for (i, evt) in events.iter().enumerate() {
            assert_eq!(evt.seq, i as u32, "seq mismatch at index {i}");
            assert_eq!(evt.activity, receipt.ocel_lifecycle[i]);
        }
    }

    #[test]
    fn ocel_events_timestamp_spaced_one_second_apart() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        let base_ms = 1_720_000_000_000u64;
        let events = ocel_events_from_lifecycle(&receipt.ocel_lifecycle, base_ms, dir.path());
        for (i, evt) in events.iter().enumerate() {
            assert_eq!(evt.timestamp_ms, base_ms + i as u64 * 1_000);
        }
    }

    #[test]
    fn ocel_event_hash_chain_links_each_event_to_previous() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        let events = ocel_events_from_lifecycle(&receipt.ocel_lifecycle, 0, dir.path());
        assert!(events[0].prev_hash.is_none(), "genesis event has no prev_hash");
        for i in 1..events.len() {
            assert_eq!(
                events[i].prev_hash.as_deref(),
                Some(events[i - 1].event_hash.as_str()),
                "event {i} prev_hash must equal event {} event_hash", i - 1
            );
        }
    }

    #[test]
    fn ocel_event_count_matches_lifecycle_on_full_real_package() {
        let dir = TempDir::new().unwrap();
        let report = make_real_report(&dir);
        let receipt = report.as_supabase_receipt();
        let events = ocel_events_from_lifecycle(&receipt.ocel_lifecycle, 0, dir.path());
        assert_eq!(events.len(), receipt.ocel_lifecycle.len());
        assert_eq!(events.len(), receipt.ocel_event_count as usize);
    }

    // ── CookLogParser tests ───────────────────────────────────────────────────

    fn write_cook_log(dir: &Path, lines: &[&str]) -> PathBuf {
        let path = dir.join("ue4-cook-latest.log");
        std::fs::write(&path, lines.join("\n")).unwrap();
        path
    }

    #[test]
    fn parse_cook_log_empty_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[]);
        let events = parse_cook_log(&log, 0);
        assert!(events.is_empty());
    }

    #[test]
    fn parse_cook_log_detects_cook_started() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &["Running BuildCookRun for Brm project..."]);
        let events = parse_cook_log(&log, 0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].activity, "CookStarted");
    }

    #[test]
    fn parse_cook_log_detects_pak_complete_with_detail() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "LogPak: Display: Created pak file /tmp/brm-html5-archive/HTML5/Brm-HTML5.pak",
        ]);
        let events = parse_cook_log(&log, 0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].activity, "PakComplete");
        assert!(events[0].detail.as_deref().unwrap_or("").contains("Brm-HTML5.pak"));
    }

    #[test]
    fn parse_cook_log_deduplicates_same_activity() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "Running BuildCookRun",
            "Running BuildCookRun again (should be deduped)",
        ]);
        let events = parse_cook_log(&log, 0);
        assert_eq!(events.len(), 1, "CookStarted must appear only once");
    }

    #[test]
    fn parse_cook_log_timestamps_increase_monotonically() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "Running BuildCookRun",
            "LogCook: Display: Cooking package: /Game/Maps/TestMap",
            "LogPak: Display: Created pak file /out/game.pak",
            "Total cook time was 42s",
        ]);
        let events = parse_cook_log(&log, 1_000_000);
        assert!(events.len() >= 2);
        for w in events.windows(2) {
            assert!(w[1].timestamp_ms > w[0].timestamp_ms, "timestamps must increase");
        }
    }

    #[test]
    fn parse_cook_log_detects_cook_error() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "Error: Failed to cook package /Game/Meshes/Foo",
        ]);
        let events = parse_cook_log(&log, 0);
        assert_eq!(events[0].activity, "CookError");
        assert!(events[0].detail.as_deref().unwrap_or("").contains("Failed to cook"));
    }

    #[test]
    fn parse_cook_log_extracts_package_cooking_detail() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "LogCook: Display: Cooking package: /Game/Maps/MainLevel",
        ]);
        let events = parse_cook_log(&log, 0);
        assert_eq!(events[0].activity, "PackageCooking");
        assert_eq!(events[0].detail.as_deref(), Some("/Game/Maps/MainLevel"));
    }

    #[test]
    fn parse_cook_log_missing_file_returns_empty() {
        let events = parse_cook_log(std::path::Path::new("/nonexistent/cook.log"), 0);
        assert!(events.is_empty());
    }

    #[test]
    fn parse_cook_log_detects_new_patterns() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "Running BuildCookRun",
            "LogShaderCompilers: Compiling shaders",
            "LogPak: Display: Collecting files to pak",
            "Staging complete, archive ready",
            "Package was created successfully",
            "BuildCookRun: Completed successfully",
        ]);
        let events = parse_cook_log(&log, 0);
        let activities: Vec<&str> = events.iter().map(|e| e.activity.as_str()).collect();
        assert!(activities.contains(&"CookStarted"), "CookStarted missing");
        assert!(activities.contains(&"ShaderCompileStarted"), "ShaderCompileStarted missing");
        assert!(activities.contains(&"PakStarted"), "PakStarted missing");
        assert!(activities.contains(&"StagingComplete"), "StagingComplete missing");
        assert!(activities.contains(&"PackageCreated"), "PackageCreated missing");
        assert!(activities.contains(&"CookFinished"), "CookFinished missing");
    }

    #[test]
    fn parse_cook_log_failed_pattern() {
        let dir = TempDir::new().unwrap();
        let log = write_cook_log(dir.path(), &[
            "Running BuildCookRun",
            "ERROR: Exception was thrown during cook",
        ]);
        let events = parse_cook_log(&log, 0);
        let activities: Vec<&str> = events.iter().map(|e| e.activity.as_str()).collect();
        assert!(activities.contains(&"CookStarted"));
        // CookError matches "Error:" and "ERROR:" — verify it fires
        assert!(activities.iter().any(|a| *a == "CookError" || *a == "CookFailed"));
    }

    // ── output_hash tests ──────────────────────────────────────────────────────

    #[test]
    fn write_receipt_includes_output_hash_for_real_wasm() {
        let dir = TempDir::new().unwrap();
        let wasm = dir.path().join("Brm.wasm");
        // Write a valid WASM magic header
        std::fs::write(&wasm, b"\0asm\x01\0\0\0test-payload").unwrap();

        let report = Html5PackageReport {
            archive_dir: dir.path().to_owned(),
            wasm_files: vec![WasmFileReport { path: wasm.clone(), verdict: WasmVerdict::Real { size_bytes: 16 } }],
            companions: CompanionReport { has_js: false, has_html: false, has_data_or_pak: false },
            is_real_package: true,
            ui_input_patched: false,
            cook_session_id: None,
            cook_log_events: vec![],
        };

        let path = report.write_receipt().unwrap();
        let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        let hash = json["output_hash"].as_str().expect("output_hash must be present");
        assert_eq!(hash.len(), 64, "BLAKE3 hex must be 64 chars");
        // Verify it's actually the BLAKE3 of the file bytes
        let expected = blake3::hash(b"\0asm\x01\0\0\0test-payload").to_hex().to_string();
        assert_eq!(hash, expected, "output_hash must be BLAKE3 of WASM bytes");
    }

    #[test]
    fn write_receipt_output_hash_null_for_stub_wasm() {
        let dir = TempDir::new().unwrap();
        let wasm = dir.path().join("Stub.wasm");
        std::fs::write(&wasm, b"stub").unwrap();

        let report = Html5PackageReport {
            archive_dir: dir.path().to_owned(),
            wasm_files: vec![WasmFileReport { path: wasm, verdict: WasmVerdict::Stub { size_bytes: 4 } }],
            companions: CompanionReport { has_js: false, has_html: false, has_data_or_pak: false },
            is_real_package: false,
            ui_input_patched: false,
            cook_session_id: None,
            cook_log_events: vec![],
        };

        let path = report.write_receipt().unwrap();
        let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        // Stub WASM → output_hash must be null (no real binary)
        assert!(json["output_hash"].is_null(), "output_hash must be null for stub WASM");
    }

    #[test]
    fn as_supabase_receipt_output_hash_matches_write_receipt() {
        let dir = TempDir::new().unwrap();
        let wasm = dir.path().join("Brm.wasm");
        std::fs::write(&wasm, b"\0asm\x01\0\0\0consistency-check").unwrap();

        let report = Html5PackageReport {
            archive_dir: dir.path().to_owned(),
            wasm_files: vec![WasmFileReport { path: wasm, verdict: WasmVerdict::Real { size_bytes: 24 } }],
            companions: CompanionReport { has_js: true, has_html: true, has_data_or_pak: false },
            is_real_package: true,
            ui_input_patched: false,
            cook_session_id: None,
            cook_log_events: vec![],
        };

        let disk_path = report.write_receipt().unwrap();
        let disk_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&disk_path).unwrap()).unwrap();
        let supa_receipt = report.as_supabase_receipt();

        let disk_hash = disk_json["output_hash"].as_str().unwrap();
        let supa_hash = supa_receipt.output_hash.as_deref().unwrap();
        assert_eq!(disk_hash, supa_hash,
            "output_hash must be identical in write_receipt() and as_supabase_receipt()");
    }
}
