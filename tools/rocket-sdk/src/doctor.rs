use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

use crate::config::discover_python3;
use crate::html5::{Html5PackageVerifier, WasmVerdict};

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Serialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
}

// TRACKED_WORK(anti-cheat): DiagnosticReport previously derived Deserialize but DateTime<Utc>
// requires chrono's "serde" feature flag which was not enabled in Cargo.toml, causing
// a compile error. The CLI only serializes (outputs) diagnostic reports — it never
// deserializes them — so Deserialize has been removed.
#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticReport {
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<CheckResult>,
}

pub struct RocketDoctor {
    project_root: PathBuf,
}

impl RocketDoctor {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Resolve UE4 root from `.rocket.json` → `UE4_ROOT` env → `UE_ROOT` env.
    fn resolve_ue4_root(&self) -> Option<PathBuf> {
        let rocket_json = self.project_root.join(".rocket.json");
        if rocket_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&rocket_json) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(root) = v.get("ue4_root").and_then(|r| r.as_str()) {
                        return Some(PathBuf::from(root));
                    }
                }
            }
        }
        std::env::var("UE4_ROOT")
            .or_else(|_| std::env::var("UE_ROOT"))
            .ok()
            .map(PathBuf::from)
    }

    pub fn run_diagnostics(&self) -> DiagnosticReport {
        let checks = vec![
            self.check_git(),
            self.check_git_status(),
            self.check_rust(),
            self.check_python(),
            self.check_manifest(),
            self.check_manifest_projects(),
            self.check_versions_dir(),
            self.check_ue4_root(),
            self.check_ue4_plugins(),
            self.check_html5_toolchain(),
            self.check_ggen(),
            self.check_anti_llm_cheat_lsp(),
            self.check_node(),
            self.check_html5_package(),
            self.check_ue4_build_scripts(),
            self.check_nexus_types(),
            self.check_xcode(),
        ];

        DiagnosticReport {
            timestamp: Utc::now(),
            checks,
        }
    }

    fn check_git(&self) -> CheckResult {
        match Command::new("git").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "Git".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "Git".to_string(),
                status: CheckStatus::Fail,
                message: "Git not found in PATH".to_string(),
                details: None,
            },
        }
    }

    fn check_git_status(&self) -> CheckResult {
        match git2::Repository::open(&self.project_root) {
            Ok(repo) => {
                let mut message = String::new();
                let mut status = CheckStatus::Pass;

                // Branch name
                let head = match repo.head() {
                    Ok(head) => head.shorthand().unwrap_or("unknown").to_string(),
                    Err(_) => "HEAD detached or empty".to_string(),
                };
                message.push_str(&format!("Branch: {}", head));

                // Uncommitted changes
                let mut status_options = git2::StatusOptions::new();
                status_options.include_untracked(true);
                match repo.statuses(Some(&mut status_options)) {
                    Ok(statuses) => {
                        if !statuses.is_empty() {
                            status = CheckStatus::Warn;
                            message.push_str(&format!(", {} uncommitted changes", statuses.len()));
                        } else {
                            message.push_str(", no uncommitted changes");
                        }
                    }
                    Err(e) => {
                        return CheckResult {
                            name: "Git Status".to_string(),
                            status: CheckStatus::Warn,
                            message: format!("Branch: {}, could not check statuses: {}", head, e),
                            details: None,
                        };
                    }
                }

                CheckResult {
                    name: "Git Status".to_string(),
                    status,
                    message,
                    details: None,
                }
            }
            Err(e) => CheckResult {
                name: "Git Status".to_string(),
                status: CheckStatus::Fail,
                message: "Not a git repository".to_string(),
                details: Some(e.to_string()),
            },
        }
    }

    fn check_rust(&self) -> CheckResult {
        match Command::new("rustc").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "Rust".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "Rust".to_string(),
                status: CheckStatus::Fail,
                message: "Rust (rustc) not found in PATH".to_string(),
                details: None,
            },
        }
    }

    fn check_python(&self) -> CheckResult {
        let cmd = if Command::new("python3").arg("--version").output().is_ok() {
            "python3"
        } else {
            "python"
        };

        match Command::new(cmd).arg("--version").output() {
            Ok(output) => CheckResult {
                name: "Python".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "Python".to_string(),
                status: CheckStatus::Fail,
                message: "Python not found in PATH".to_string(),
                details: None,
            },
        }
    }

    fn check_ggen(&self) -> CheckResult {
        match Command::new("ggen").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "ggen".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "ggen".to_string(),
                status: CheckStatus::Warn,
                message: "ggen not found in PATH".to_string(),
                details: Some("ggen is required for Ostar generative workflows.".to_string()),
            },
        }
    }

    fn check_manifest(&self) -> CheckResult {
        let path = self.project_root.join("project-manifest.json");
        if path.exists() {
            CheckResult {
                name: "Project Manifest".to_string(),
                status: CheckStatus::Pass,
                message: "project-manifest.json found".to_string(),
                details: Some(format!("Path: {}", path.display())),
            }
        } else {
            CheckResult {
                name: "Project Manifest".to_string(),
                status: CheckStatus::Fail,
                message: "project-manifest.json MISSING".to_string(),
                details: Some("Run 'rocket sync' to generate it.".to_string()),
            }
        }
    }

    /// Validate that every project declared in `project-manifest.json` has its
    /// `.uproject` file on disk. Returns Warn if the manifest is absent (covered
    /// by `check_manifest`). Reports each missing uproject file in `details`.
    fn check_manifest_projects(&self) -> CheckResult {
        let manifest_path = self.project_root.join("project-manifest.json");
        if !manifest_path.exists() {
            return CheckResult {
                name: "Manifest Projects".to_string(),
                status: CheckStatus::Warn,
                message: "Skipped: project-manifest.json not found".to_string(),
                details: None,
            };
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                return CheckResult {
                    name: "Manifest Projects".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("Cannot read project-manifest.json: {e}"),
                    details: None,
                };
            }
        };

        let manifest: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                return CheckResult {
                    name: "Manifest Projects".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("project-manifest.json is invalid JSON: {e}"),
                    details: None,
                };
            }
        };

        let projects = match manifest.get("projects").and_then(|p| p.as_array()) {
            Some(arr) => arr.clone(),
            None => {
                return CheckResult {
                    name: "Manifest Projects".to_string(),
                    status: CheckStatus::Warn,
                    message: "No 'projects' array in manifest".to_string(),
                    details: None,
                };
            }
        };

        let mut missing = Vec::new();
        let mut total = 0usize;

        for proj in &projects {
            if let Some(uproject_path) = proj.get("uproject_path").and_then(|p| p.as_str()) {
                total += 1;
                let full_path = if std::path::Path::new(uproject_path).is_absolute() {
                    PathBuf::from(uproject_path)
                } else {
                    self.project_root.join(uproject_path)
                };
                if !full_path.exists() {
                    let name = proj
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(uproject_path);
                    missing.push(format!("{name} ({uproject_path})"));
                }
            }
        }

        if missing.is_empty() {
            CheckResult {
                name: "Manifest Projects".to_string(),
                status: CheckStatus::Pass,
                message: format!("All {total} declared .uproject files present on disk"),
                details: None,
            }
        } else {
            CheckResult {
                name: "Manifest Projects".to_string(),
                status: CheckStatus::Fail,
                message: format!(
                    "{}/{total} .uproject files MISSING",
                    missing.len()
                ),
                details: Some(missing.join("\n")),
            }
        }
    }

    fn check_versions_dir(&self) -> CheckResult {
        let path = self.project_root.join("versions");
        if path.exists() && path.is_dir() {
            CheckResult {
                name: "Versions Directory".to_string(),
                status: CheckStatus::Pass,
                message: "versions/ directory exists".to_string(),
                details: None,
            }
        } else {
            CheckResult {
                name: "Versions Directory".to_string(),
                status: CheckStatus::Fail,
                message: "versions/ directory MISSING or not a directory".to_string(),
                details: Some(
                    "This directory should contain the Unreal Engine projects.".to_string(),
                ),
            }
        }
    }

    fn check_anti_llm_cheat_lsp(&self) -> CheckResult {
        match Command::new("anti-llm-cheat-lsp").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "anti-llm-cheat-lsp".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "anti-llm-cheat-lsp".to_string(),
                status: CheckStatus::Warn,
                message: "anti-llm-cheat-lsp not found in PATH".to_string(),
                details: Some(
                    "Install: cargo install lsp-max --bin anti-llm-cheat-lsp".to_string(),
                ),
            },
        }
    }

    fn check_ue4_root(&self) -> CheckResult {
        // Parse the JSON properly — string search gives false positives
        // (e.g. "ue4_root" appearing in a comment or description value).
        let rocket_json = self.project_root.join(".rocket.json");
        if rocket_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&rocket_json) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(root) = v.get("ue4_root").and_then(|r| r.as_str()) {
                        let root_path = PathBuf::from(root);
                        if root_path.exists() {
                            return CheckResult {
                                name: "UE4 Root".to_string(),
                                status: CheckStatus::Pass,
                                message: format!("UE4 root: {root}"),
                                details: None,
                            };
                        } else {
                            return CheckResult {
                                name: "UE4 Root".to_string(),
                                status: CheckStatus::Fail,
                                message: format!("UE4 root configured but path missing: {root}"),
                                details: Some(
                                    "Run 'rocket setup' to reconfigure.".to_string(),
                                ),
                            };
                        }
                    }
                }
            }
        }

        if let Ok(root) = std::env::var("UE4_ROOT") {
            let root_path = PathBuf::from(&root);
            if root_path.exists() {
                CheckResult {
                    name: "UE4 Root".to_string(),
                    status: CheckStatus::Pass,
                    message: format!("UE4_ROOT={root}"),
                    details: None,
                }
            } else {
                CheckResult {
                    name: "UE4 Root".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("UE4_ROOT set but path missing: {root}"),
                    details: None,
                }
            }
        } else {
            CheckResult {
                name: "UE4 Root".to_string(),
                status: CheckStatus::Warn,
                message: "UE4 root not configured".to_string(),
                details: Some("Run 'rocket setup' to configure Unreal Engine path.".to_string()),
            }
        }
    }

    fn check_html5_toolchain(&self) -> CheckResult {
        // 1. Verify Python 3 is available for UAT/UHT scripts.
        let python_ok = discover_python3().map(|path| format!("Python 3 at {}", path.display()));

        // 2. Verify emscripten — check engine-bundled emsdk first, then PATH.
        let emsdk_found = self.find_ue4_emsdk();
        let emcc_on_path = Command::new("emcc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        match (python_ok, emsdk_found || emcc_on_path) {
            (Some(py), true) => CheckResult {
                name: "HTML5 Toolchain".to_string(),
                status: CheckStatus::Pass,
                message: format!("{py}; emscripten present"),
                details: None,
            },
            (Some(py), false) => CheckResult {
                name: "HTML5 Toolchain".to_string(),
                status: CheckStatus::Warn,
                message: format!("{py}; emscripten NOT found"),
                details: Some(
                    "Run HTML5Setup.sh in the engine to build emsdk, or run 'rocket html5 setup'."
                        .to_string(),
                ),
            },
            (None, _) => CheckResult {
                name: "HTML5 Toolchain".to_string(),
                status: CheckStatus::Fail,
                message: "Python 3 not found — required for UAT scripts".to_string(),
                details: Some(
                    "Install python3 or set 'python3_path' in .rocket.json".to_string(),
                ),
            },
        }
    }

    /// Check if the engine's bundled emsdk is present (built by HTML5Setup.sh).
    fn find_ue4_emsdk(&self) -> bool {
        self.resolve_ue4_root()
            .map(|r| r.join("Engine/Platforms/HTML5/Build/emsdk").exists())
            .unwrap_or(false)
    }

    fn check_ue4_plugins(&self) -> CheckResult {
        let root_path = match self.resolve_ue4_root() {
            Some(p) => p,
            None => {
                return CheckResult {
                    name: "UE4 Plugins".to_string(),
                    status: CheckStatus::Warn,
                    message: "Skipped: UE4 root not configured, cannot verify plugins".to_string(),
                    details: None,
                };
            }
        };

        // Check WebSocketNetworking
        let ws_paths = vec![
            "Engine/Plugins/Runtime/Networking/WebSocketNetworking/WebSocketNetworking.uplugin",
            "Engine/Plugins/Networking/WebSocketNetworking/WebSocketNetworking.uplugin",
            "Engine/Plugins/WebSocketNetworking/WebSocketNetworking.uplugin",
        ];
        let mut ws_ok = false;
        for rel in &ws_paths {
            if root_path.join(rel).exists() {
                ws_ok = true;
                break;
            }
        }

        // Check VaRest
        let varest_paths = vec![
            "Engine/Plugins/Marketplace/VaRest/VaRest.uplugin",
            "Engine/Plugins/VaRest/VaRest.uplugin",
        ];
        let mut varest_ok = false;
        for rel in &varest_paths {
            if root_path.join(rel).exists() {
                varest_ok = true;
                break;
            }
        }

        if ws_ok && varest_ok {
            CheckResult {
                name: "UE4 Plugins".to_string(),
                status: CheckStatus::Pass,
                message: "Found required plugins: WebSocketNetworking, VaRest".to_string(),
                details: None,
            }
        } else {
            let mut missing = Vec::new();
            if !ws_ok {
                missing.push("WebSocketNetworking");
            }
            if !varest_ok {
                missing.push("VaRest");
            }
            CheckResult {
                name: "UE4 Plugins".to_string(),
                status: CheckStatus::Fail,
                message: format!("Missing plugins: {}", missing.join(", ")),
                details: Some(
                    "Ensure your UE4.24 build includes WebSocketNetworking and VaRest plugins."
                        .to_string(),
                ),
            }
        }
    }
    /// Check whether the most recent HTML5 cook produced a real package.
    ///
    /// Looks for the default archive directory (`/tmp/brm-html5-archive`) first,
    /// then falls back to `manufactured/` in the project root.
    fn check_html5_package(&self) -> CheckResult {
        let candidates = [
            PathBuf::from("/tmp/brm-html5-archive/HTML5"),
            PathBuf::from("/tmp/brm-html5-archive"),
            self.project_root.join("manufactured"),
            self.project_root.join("pwa-staff/manufactured"),
        ];

        let archive_dir = candidates.iter().find(|d| d.exists());

        match archive_dir {
            None => CheckResult {
                name: "HTML5 Package".to_string(),
                status: CheckStatus::Warn,
                message: "No HTML5 archive directory found".to_string(),
                details: Some(
                    "Run 'rocket html5 cook --project Brm' to produce a package.".to_string(),
                ),
            },
            Some(dir) => {
                match Html5PackageVerifier::new(dir).verify() {
                    Err(e) => CheckResult {
                        name: "HTML5 Package".to_string(),
                        status: CheckStatus::Fail,
                        message: format!("Verification error: {e}"),
                        details: None,
                    },
                    Ok(report) => {
                        let summary = report.summary();
                        if report.is_real_package {
                            CheckResult {
                                name: "HTML5 Package".to_string(),
                                status: CheckStatus::Pass,
                                message: summary,
                                details: Some(format!("Archive: {}", dir.display())),
                            }
                        } else {
                            // Distinguish between stub/no-wasm and companion-missing
                            let has_real_wasm = report.wasm_files.iter().any(|f| {
                                matches!(f.verdict, WasmVerdict::Real { .. })
                            });
                            let status = if has_real_wasm {
                                CheckStatus::Warn // WASM is real but companions missing
                            } else {
                                CheckStatus::Fail // stub or no wasm
                            };
                            CheckResult {
                                name: "HTML5 Package".to_string(),
                                status,
                                message: summary,
                                details: Some(format!("Archive: {}", dir.display())),
                            }
                        }
                    }
                }
            }
        }
    }

    /// Verify that the critical UE4 build scripts are present and executable.
    ///
    /// Checks RunUAT.sh (required for cook+package) and the Mac/Linux Build.sh
    /// scripts. Also validates the HTML5-specific setup script is present when
    /// an emsdk is configured. Reports Warn rather than Fail when UE4_ROOT is
    /// not configured at all (the `check_ue4_root` check already covers that).
    fn check_ue4_build_scripts(&self) -> CheckResult {
        let root = match self.resolve_ue4_root() {
            None => {
                return CheckResult {
                    name: "UE4 Build Scripts".to_string(),
                    status: CheckStatus::Warn,
                    message: "Skipped: UE4 root not configured".to_string(),
                    details: None,
                };
            }
            Some(r) if !r.exists() => {
                return CheckResult {
                    name: "UE4 Build Scripts".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("UE4 root path missing: {}", r.display()),
                    details: None,
                };
            }
            Some(r) => r,
        };

        // Critical scripts that must exist for `rocket build` and `rocket html5 cook`.
        let required = [
            "Engine/Build/BatchFiles/RunUAT.sh",
            "Engine/Build/BatchFiles/Mac/Build.sh",
        ];
        let optional = [
            "Engine/Platforms/HTML5/HTML5Setup.sh",
        ];

        let mut missing_required: Vec<&str> = Vec::new();
        let mut missing_optional: Vec<&str> = Vec::new();

        for rel in &required {
            if !root.join(rel).exists() {
                missing_required.push(rel);
            }
        }
        for rel in &optional {
            if !root.join(rel).exists() {
                missing_optional.push(rel);
            }
        }

        if !missing_required.is_empty() {
            return CheckResult {
                name: "UE4 Build Scripts".to_string(),
                status: CheckStatus::Fail,
                message: format!("Missing critical scripts: {}", missing_required.join(", ")),
                details: Some(format!(
                    "UE4 root: {} — ensure you have a full engine build with BatchFiles",
                    root.display()
                )),
            };
        }

        if !missing_optional.is_empty() {
            CheckResult {
                name: "UE4 Build Scripts".to_string(),
                status: CheckStatus::Warn,
                message: format!(
                    "RunUAT.sh present; HTML5 setup scripts missing: {}",
                    missing_optional.join(", ")
                ),
                details: Some(
                    "Run HTML5Setup.sh from the SpeculativeCoder/UnrealEngine fork to enable HTML5 packaging".to_string(),
                ),
            }
        } else {
            CheckResult {
                name: "UE4 Build Scripts".to_string(),
                status: CheckStatus::Pass,
                message: format!(
                    "RunUAT.sh, Build.sh, HTML5Setup.sh present at {}",
                    root.display()
                ),
                details: None,
            }
        }
    }

    /// Quick compile-check of `nexus-types` — the zero-dependency root of the
    /// nexus-engine workspace. A failure here means the foundational shared types
    /// are broken, which would cascade to every other nexus crate.
    /// Check that Node.js ≥20 and npm are available — required for `rocket pwa`.
    fn check_node(&self) -> CheckResult {
        let node_output = Command::new("node").arg("--version").output();
        let npm_output = Command::new("npm").arg("--version").output();

        let node_version = node_output.ok().and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

        let npm_ok = npm_output
            .map(|o| o.status.success())
            .unwrap_or(false);

        match (node_version, npm_ok) {
            (Some(v), true) => {
                // Warn if Node < 20 (pwa-staff requires Node 20.x)
                let major: Option<u32> = v
                    .trim_start_matches('v')
                    .split('.')
                    .next()
                    .and_then(|s| s.parse().ok());
                if major.map(|m| m >= 20).unwrap_or(false) {
                    CheckResult {
                        name: "Node.js".to_string(),
                        status: CheckStatus::Pass,
                        message: format!("Node.js {v} with npm"),
                        details: None,
                    }
                } else {
                    CheckResult {
                        name: "Node.js".to_string(),
                        status: CheckStatus::Warn,
                        message: format!("Node.js {v} found but pwa-staff requires ≥20"),
                        details: Some("Upgrade via nvm: `nvm install 20 && nvm use 20`".into()),
                    }
                }
            }
            (Some(v), false) => CheckResult {
                name: "Node.js".to_string(),
                status: CheckStatus::Warn,
                message: format!("Node.js {v} found but npm not found"),
                details: Some("Install npm: `npm install -g npm`".into()),
            },
            (None, _) => CheckResult {
                name: "Node.js".to_string(),
                status: CheckStatus::Warn,
                message: "Node.js not found — required for `rocket pwa build`".to_string(),
                details: Some("Install via nvm: `nvm install 20 && nvm use 20`".into()),
            },
        }
    }

    /// Quick compile-check of `nexus-types` — the zero-dependency root of the
    /// nexus-engine workspace. A failure here means the foundational shared types
    /// are broken, which would cascade to every other nexus crate.
    fn check_nexus_types(&self) -> CheckResult {
        let nexus_root = self.project_root.join("nexus-engine");
        if !nexus_root.exists() {
            return CheckResult {
                name: "nexus-types (compile check)".to_string(),
                status: CheckStatus::Warn,
                message: "nexus-engine directory not found; skipping compile check".to_string(),
                details: None,
            };
        }
        let output = Command::new("cargo")
            .args(["check", "-p", "nexus-types", "--quiet"])
            .current_dir(&nexus_root)
            .output();
        match output {
            Ok(o) if o.status.success() => CheckResult {
                name: "nexus-types (compile check)".to_string(),
                status: CheckStatus::Pass,
                message: "nexus-types compiles cleanly".to_string(),
                details: None,
            },
            Ok(o) => CheckResult {
                name: "nexus-types (compile check)".to_string(),
                status: CheckStatus::Fail,
                message: "nexus-types compile check failed".to_string(),
                details: Some(String::from_utf8_lossy(&o.stderr).chars().take(800).collect()),
            },
            Err(e) => CheckResult {
                name: "nexus-types (compile check)".to_string(),
                status: CheckStatus::Fail,
                message: format!("could not invoke cargo: {e}"),
                details: None,
            },
        }
    }
    /// Check that Xcode command-line tools are installed (macOS only).
    ///
    /// UE4's `Build.sh` and the Mono/C++ toolchain invoked by UAT require
    /// `xcrun` and at minimum the Xcode CLT package. Without them, `Build.sh`
    /// silently exits with a non-zero code and no human-readable error.
    fn check_xcode(&self) -> CheckResult {
        #[cfg(not(target_os = "macos"))]
        return CheckResult {
            name: "Xcode CLT".to_string(),
            status: CheckStatus::Pass,
            message: "Not macOS — skipped".to_string(),
            details: None,
        };

        #[cfg(target_os = "macos")]
        {
            // `xcode-select -p` prints the active developer directory; exits non-zero when
            // CLT are absent or the path is missing.
            let xcode_select = Command::new("xcode-select").arg("-p").output();
            match xcode_select {
                Ok(out) if out.status.success() => {
                    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    // xcrun --find clang is the minimal probe that the compiler toolchain works.
                    let clang_ok = Command::new("xcrun")
                        .args(["--find", "clang"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if clang_ok {
                        CheckResult {
                            name: "Xcode CLT".to_string(),
                            status: CheckStatus::Pass,
                            message: format!("Developer tools active at {path}"),
                            details: None,
                        }
                    } else {
                        CheckResult {
                            name: "Xcode CLT".to_string(),
                            status: CheckStatus::Warn,
                            message: format!(
                                "xcode-select path set ({path}) but clang not found via xcrun"
                            ),
                            details: Some(
                                "Run: xcode-select --install".to_string(),
                            ),
                        }
                    }
                }
                _ => CheckResult {
                    name: "Xcode CLT".to_string(),
                    status: CheckStatus::Fail,
                    message: "Xcode command-line tools not installed".to_string(),
                    details: Some(
                        "Run: xcode-select --install  (required for UE4 Build.sh)".to_string(),
                    ),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rocket_doctor_new() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        assert_eq!(doctor.project_root, dir.path().to_path_buf());
    }

    #[test]
    fn test_check_manifest_missing() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest();
        assert_eq!(result.status, CheckStatus::Fail);
        assert_eq!(result.message, "project-manifest.json MISSING");
    }

    #[test]
    fn test_check_manifest_exists() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("project-manifest.json");
        fs::write(&manifest_path, "{}").unwrap();

        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest();
        assert_eq!(result.status, CheckStatus::Pass);
        assert_eq!(result.message, "project-manifest.json found");
    }

    #[test]
    fn test_check_git_status_no_repo() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_git_status();
        assert_eq!(result.status, CheckStatus::Fail);
        assert_eq!(result.message, "Not a git repository");
    }

    // ── check_ue4_root (new behaviour) ───────────────────────────────────────

    #[test]
    fn check_ue4_root_warns_when_unconfigured() {
        let dir = tempdir().unwrap();
        // No .rocket.json, no UE4_ROOT env
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        // Temporarily clear UE4_ROOT so the test is deterministic
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let result = doctor.check_ue4_root();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("not configured"));
    }

    #[test]
    fn check_ue4_root_fails_when_path_configured_but_missing() {
        let dir = tempdir().unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(&rocket_json, r#"{"ue4_root": "/nonexistent/ue4"}"#).unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_root();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("missing"));
    }

    #[test]
    fn check_ue4_root_passes_when_path_exists() {
        let dir = tempdir().unwrap();
        let fake_ue4 = dir.path().join("ue4");
        fs::create_dir_all(&fake_ue4).unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(
            &rocket_json,
            format!(r#"{{"ue4_root": "{}"}}"#, fake_ue4.display()),
        ).unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_root();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Pass);
    }

    // ── check_html5_toolchain ─────────────────────────────────────────────────

    #[test]
    fn check_html5_toolchain_returns_a_result() {
        // Just verify it doesn't panic and returns a named result
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_html5_toolchain();
        assert_eq!(result.name, "HTML5 Toolchain");
        // On any dev machine with python3 this should be Pass or Warn (never panic)
        assert!(
            result.status == CheckStatus::Pass
                || result.status == CheckStatus::Warn
                || result.status == CheckStatus::Fail
        );
    }

    // ── check_ue4_build_scripts ───────────────────────────────────────────────

    #[test]
    fn build_scripts_warn_when_ue4_root_not_configured() {
        let dir = tempdir().unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Warn);
        assert_eq!(result.name, "UE4 Build Scripts");
    }

    #[test]
    fn build_scripts_fail_when_ue4_root_missing_from_disk() {
        let dir = tempdir().unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(&rocket_json, r#"{"ue4_root": "/nonexistent/ue4-path"}"#).unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("missing"));
    }

    #[test]
    fn build_scripts_fail_when_run_uat_absent() {
        let dir = tempdir().unwrap();
        let fake_ue4 = dir.path().join("ue4");
        // Create the root but NOT the scripts
        fs::create_dir_all(&fake_ue4).unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(&rocket_json, format!(r#"{{"ue4_root": "{}"}}"#, fake_ue4.display())).unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("RunUAT.sh"));
    }

    #[test]
    fn build_scripts_warn_when_run_uat_present_but_html5_setup_absent() {
        let dir = tempdir().unwrap();
        let fake_ue4 = dir.path().join("ue4");
        // Create RunUAT.sh and Mac/Build.sh but NOT HTML5Setup.sh
        fs::create_dir_all(fake_ue4.join("Engine/Build/BatchFiles/Mac")).unwrap();
        fs::write(fake_ue4.join("Engine/Build/BatchFiles/RunUAT.sh"), b"#!/bin/sh").unwrap();
        fs::write(fake_ue4.join("Engine/Build/BatchFiles/Mac/Build.sh"), b"#!/bin/sh").unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(&rocket_json, format!(r#"{{"ue4_root": "{}"}}"#, fake_ue4.display())).unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("RunUAT.sh present"));
    }

    #[test]
    fn build_scripts_pass_when_all_scripts_present() {
        let dir = tempdir().unwrap();
        let fake_ue4 = dir.path().join("ue4");
        fs::create_dir_all(fake_ue4.join("Engine/Build/BatchFiles/Mac")).unwrap();
        fs::create_dir_all(fake_ue4.join("Engine/Platforms/HTML5")).unwrap();
        fs::write(fake_ue4.join("Engine/Build/BatchFiles/RunUAT.sh"), b"#!/bin/sh").unwrap();
        fs::write(fake_ue4.join("Engine/Build/BatchFiles/Mac/Build.sh"), b"#!/bin/sh").unwrap();
        fs::write(fake_ue4.join("Engine/Platforms/HTML5/HTML5Setup.sh"), b"#!/bin/sh").unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(&rocket_json, format!(r#"{{"ue4_root": "{}"}}"#, fake_ue4.display())).unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev { std::env::set_var("UE4_ROOT", v); }
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("RunUAT.sh"));
        assert!(result.message.contains("HTML5Setup.sh"));
    }

    // ── check_manifest_projects ───────────────────────────────────────────────

    #[test]
    fn manifest_projects_warns_when_manifest_absent() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Warn);
    }

    #[test]
    fn manifest_projects_warns_on_missing_projects_key() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("project-manifest.json"), r#"{"version": 1}"#).unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("No 'projects'"));
    }

    #[test]
    fn manifest_projects_fails_on_invalid_json() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("project-manifest.json"), b"not json").unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("invalid JSON"));
    }

    #[test]
    fn manifest_projects_passes_when_all_uprojects_exist() {
        let dir = tempdir().unwrap();
        let uproject = dir.path().join("Game.uproject");
        fs::write(&uproject, b"{}").unwrap();
        let manifest = serde_json::json!({
            "projects": [{"name": "Game", "uproject_path": uproject.to_str().unwrap()}]
        });
        fs::write(dir.path().join("project-manifest.json"), manifest.to_string()).unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("1 declared"));
    }

    #[test]
    fn manifest_projects_fails_when_uproject_missing() {
        let dir = tempdir().unwrap();
        let manifest = serde_json::json!({
            "projects": [{"name": "Ghost", "uproject_path": "/nonexistent/Ghost.uproject"}]
        });
        fs::write(dir.path().join("project-manifest.json"), manifest.to_string()).unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("MISSING"));
        assert!(result.details.as_deref().unwrap_or("").contains("Ghost"));
    }

    #[test]
    fn manifest_projects_reports_partial_missing() {
        let dir = tempdir().unwrap();
        let present = dir.path().join("Present.uproject");
        fs::write(&present, b"{}").unwrap();
        let manifest = serde_json::json!({
            "projects": [
                {"name": "Present", "uproject_path": present.to_str().unwrap()},
                {"name": "Missing", "uproject_path": "/nonexistent/Missing.uproject"}
            ]
        });
        fs::write(dir.path().join("project-manifest.json"), manifest.to_string()).unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("1/2"));
    }

    #[test]
    fn test_check_git_status_with_repo() {
        let dir = tempdir().unwrap();
        let _repo = git2::Repository::init(dir.path()).unwrap();

        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_git_status();

        // Initial repo might have no HEAD yet
        assert_eq!(result.status, CheckStatus::Pass);
        assert_eq!(
            result.message,
            "Branch: HEAD detached or empty, no uncommitted changes"
        );

        // Add a file
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        let result = doctor.check_git_status();
        assert_eq!(result.status, CheckStatus::Warn);
        assert_eq!(
            result.message,
            "Branch: HEAD detached or empty, 1 uncommitted changes"
        );
    }

    // ── check_node ────────────────────────────────────────────────────────────

    #[test]
    fn node_check_returns_a_result() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_node();
        assert_eq!(result.name, "Node.js");
        // Accept any status — the check should not panic regardless of env
        matches!(result.status, CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail);
    }

    #[test]
    fn node_check_pass_or_warn_status_has_nonempty_message() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_node();
        assert!(!result.message.is_empty());
    }

    // ── check_xcode ───────────────────────────────────────────────────────────

    #[test]
    fn xcode_check_returns_named_result() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_xcode();
        assert_eq!(result.name, "Xcode CLT");
        assert!(!result.message.is_empty());
    }

    #[test]
    fn xcode_check_status_is_valid_variant() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_xcode();
        // Should be Pass on this mac (Xcode is installed) or Fail/Warn without CLT
        matches!(result.status, CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail);
    }
}
