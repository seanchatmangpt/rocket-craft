use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use crate::config::discover_python3;

const GIT_WRAPPER_DIR: &str = "/tmp/ubt-git-wrapper";

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
}

impl Html5PackageReport {
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
}

impl Html5PackageVerifier {
    pub fn new(archive_dir: impl Into<PathBuf>) -> Self {
        Self {
            archive_dir: archive_dir.into(),
            min_wasm_bytes: MIN_REAL_WASM_BYTES,
        }
    }

    pub fn with_min_wasm_bytes(mut self, bytes: u64) -> Self {
        self.min_wasm_bytes = bytes;
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

        Ok(Html5PackageReport {
            archive_dir: self.archive_dir.clone(),
            wasm_files,
            companions,
            is_real_package,
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
        }
    }

    /// Override the client config (defaults to `"Development"`).
    /// Use `"Shipping"` for release packages.
    pub fn with_client_config(mut self, config: impl Into<String>) -> Self {
        self.client_config = config.into();
        self
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

        // Resolve Python 3 at runtime — never hardcode a path.
        let python3 = discover_python3().ok_or_else(|| {
            anyhow::anyhow!(
                "Python 3 not found. Install python3 or set 'python3_path' in .rocket.json."
            )
        })?;

        // UHT computes CURRENT_FILE_ID relative to the parent of the project directory.
        // If that parent dir starts with a digit (e.g. versions/4.27.0/), all generated
        // macros start with a digit — invalid C identifier. Create a symlink with a
        // letter-starting name so UHT sees a valid base path.
        let project = ensure_letter_start_symlink(&self.project)?;

        let path_with_wrapper = prepend_to_path(GIT_WRAPPER_DIR);

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
}
