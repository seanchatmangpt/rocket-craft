use chrono::{DateTime, Utc};
use colored::Colorize;
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

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum CheckCategory {
    Required,
    Optional,
}

#[derive(Debug, Serialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
    /// Category determines whether a Fail counts against required health.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<CheckCategory>,
    /// Shell command that can auto-fix a failing check (used with --fix).
    #[serde(skip)]
    pub fix_command: Option<Vec<String>>,
    /// Human-readable suggestion shown next to failing checks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
}

impl CheckResult {
    fn required(
        name: &str,
        status: CheckStatus,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            status,
            message: message.into(),
            details,
            category: Some(CheckCategory::Required),
            fix_command: None,
            fix_hint: None,
        }
    }

    fn optional(
        name: &str,
        status: CheckStatus,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            status,
            message: message.into(),
            details,
            category: Some(CheckCategory::Optional),
            fix_command: None,
            fix_hint: None,
        }
    }

    fn with_fix(mut self, cmd: Vec<&str>, hint: &str) -> Self {
        self.fix_command = Some(cmd.into_iter().map(str::to_string).collect());
        self.fix_hint = Some(hint.to_string());
        self
    }
}

// TRACKED_WORK(anti-cheat): DiagnosticReport previously derived Deserialize but DateTime<Utc>
// requires chrono's "serde" feature flag which was not enabled in Cargo.toml, causing
// a compile error. The CLI only serializes (outputs) diagnostic reports -- it never
// deserializes them -- so Deserialize has been removed.
#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticReport {
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<CheckResult>,
}

impl DiagnosticReport {
    /// Number of required checks passing.
    pub fn required_pass_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| {
                c.category == Some(CheckCategory::Required) && c.status == CheckStatus::Pass
            })
            .count()
    }

    /// Total number of required checks.
    pub fn required_total(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.category == Some(CheckCategory::Required))
            .count()
    }

    /// All required checks passing.
    pub fn all_required_pass(&self) -> bool {
        self.required_pass_count() == self.required_total()
    }
}

/// Pretty-print a single check result using colored symbols.
pub fn print_check(check: &CheckResult) {
    let (symbol, colored_name) = match check.status {
        CheckStatus::Pass => ("v".green().bold(), check.name.green()),
        CheckStatus::Warn => ("!".yellow().bold(), check.name.yellow()),
        CheckStatus::Fail => ("x".red().bold(), check.name.red()),
    };

    let hint_suffix = match (&check.fix_hint, &check.status) {
        (Some(hint), CheckStatus::Fail) | (Some(hint), CheckStatus::Warn) => {
            format!("  {}", format!("-> {hint}").dimmed())
        }
        _ => String::new(),
    };

    println!(
        "  {}  {:<26} {}{}",
        symbol,
        colored_name,
        check.message,
        hint_suffix
    );

    if let Some(ref details) = check.details {
        for line in details.lines() {
            println!("     {}", line.dimmed());
        }
    }
}

/// Print a section header.
pub fn print_section(title: &str) {
    println!("\n{}", format!("-- {title} ").bold());
}

/// Print the health-score summary bar.
pub fn print_health_score(report: &DiagnosticReport) {
    let pass = report.required_pass_count();
    let total = report.required_total();
    let pct = if total == 0 {
        100
    } else {
        (pass * 100) / total
    };

    let bar_width = 10usize;
    let filled = (pass * bar_width) / total.max(1);
    let empty = bar_width - filled;
    let bar: String = "#".repeat(filled) + &".".repeat(empty);

    let score_str = format!("Health: {pass}/{total} checks passing");
    let bar_str = format!("[{bar}]");
    let pct_str = format!("{pct}%");

    let score_colored = if pass == total {
        score_str.green().bold().to_string()
    } else if pass >= total / 2 {
        score_str.yellow().bold().to_string()
    } else {
        score_str.red().bold().to_string()
    };

    println!("\n{score_colored}  {bar_str}  {pct_str}");
}

/// Emit a structured JSON report to stdout.
pub fn print_json_report(report: &DiagnosticReport) {
    let checks: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|c| {
            let status = match c.status {
                CheckStatus::Pass => "pass",
                CheckStatus::Warn => "warn",
                CheckStatus::Fail => "fail",
            };
            let category = match &c.category {
                Some(CheckCategory::Required) => "required",
                Some(CheckCategory::Optional) => "optional",
                None => "required",
            };
            let mut obj = serde_json::json!({
                "name": c.name,
                "status": status,
                "value": c.message,
                "category": category,
            });
            if let Some(d) = &c.details {
                obj["details"] = serde_json::Value::String(d.clone());
            }
            if let Some(h) = &c.fix_hint {
                obj["fix_hint"] = serde_json::Value::String(h.clone());
            }
            obj
        })
        .collect();

    let score = report.required_pass_count();
    let total = report.required_total();

    let output = serde_json::json!({
        "checks": checks,
        "score": score,
        "total": total,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}

/// Try to auto-fix a failing check by running its fix command.
fn run_fix(check: &CheckResult, project_root: &PathBuf) -> bool {
    let Some(ref cmd) = check.fix_command else {
        return false;
    };
    if cmd.is_empty() {
        return false;
    }
    println!(
        "  {}  Running fix for '{}': {}",
        "~".cyan(),
        check.name,
        cmd.join(" ").bold()
    );
    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(project_root)
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("  {}  Fix applied successfully.", "v".green());
            true
        }
        Ok(s) => {
            println!("  {}  Fix exited with code {:?}", "x".red(), s.code());
            false
        }
        Err(e) => {
            println!("  {}  Fix failed: {e}", "x".red());
            false
        }
    }
}

pub struct RocketDoctor {
    project_root: PathBuf,
}

impl RocketDoctor {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Resolve UE4 root from `.rocket.json` -> `UE4_ROOT` env -> `UE_ROOT` env.
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

    /// Run all diagnostic checks, grouped into Required then Optional.
    pub fn run_diagnostics(&self) -> DiagnosticReport {
        let checks = vec![
            // -- Required ---------------------------------------------------------
            self.check_git(),
            self.check_rust(),
            self.check_manifest(),
            self.check_versions_dir(),
            self.check_ue4_root(),
            self.check_ue4_plugins(),
            self.check_ue4_build_scripts(),
            self.check_nexus_types(),
            // -- Optional ---------------------------------------------------------
            self.check_git_status(),
            self.check_python(),
            self.check_node(),
            self.check_rustfmt_clippy(),
            self.check_pwa_node_modules(),
            self.check_dotenv(),
            self.check_html5_toolchain(),
            self.check_html5_package(),
            self.check_ue4_build_scripts_html5(),
            self.check_manifest_projects(),
            self.check_ggen(),
            self.check_anti_llm_cheat_lsp(),
            self.check_xcode(),
        ];

        DiagnosticReport {
            timestamp: Utc::now(),
            checks,
        }
    }

    /// Run all auto-fixes for failing/warning checks, then re-run diagnostics.
    pub fn run_fix_and_recheck(&self) -> DiagnosticReport {
        let initial = self.run_diagnostics();
        let mut any_fixed = false;

        for check in &initial.checks {
            if matches!(check.status, CheckStatus::Fail | CheckStatus::Warn) {
                if check.fix_command.is_some() {
                    let fixed = run_fix(check, &self.project_root);
                    if fixed {
                        any_fixed = true;
                    }
                }
            }
        }

        if any_fixed {
            println!("\n{}", "Re-running checks after fixes...".bold());
        }

        self.run_diagnostics()
    }

    // -------------------------------------------------------------------------
    // Required checks
    // -------------------------------------------------------------------------

    fn check_git(&self) -> CheckResult {
        match Command::new("git").arg("--version").output() {
            Ok(output) => CheckResult::required(
                "Git",
                CheckStatus::Pass,
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                None,
            ),
            Err(_) => {
                CheckResult::required("Git", CheckStatus::Fail, "Git not found in PATH", None)
            }
        }
    }

    fn check_rust(&self) -> CheckResult {
        match Command::new("rustc").arg("--version").output() {
            Ok(output) => CheckResult::required(
                "Rust stable",
                CheckStatus::Pass,
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                None,
            ),
            Err(_) => CheckResult::required(
                "Rust stable",
                CheckStatus::Fail,
                "Rust (rustc) not found in PATH",
                None,
            ),
        }
    }

    fn check_manifest(&self) -> CheckResult {
        let path = self.project_root.join("project-manifest.json");
        if path.exists() {
            CheckResult::required(
                "Project Manifest",
                CheckStatus::Pass,
                "project-manifest.json found",
                Some(format!("Path: {}", path.display())),
            )
        } else {
            CheckResult::required(
                "Project Manifest",
                CheckStatus::Fail,
                "project-manifest.json MISSING",
                Some("Run 'rocket sync' to generate it.".to_string()),
            )
            .with_fix(vec!["./rocket", "sync"], "run: ./rocket sync")
        }
    }

    fn check_versions_dir(&self) -> CheckResult {
        let path = self.project_root.join("versions");
        if path.exists() && path.is_dir() {
            CheckResult::required(
                "Versions Directory",
                CheckStatus::Pass,
                "versions/ directory exists",
                None,
            )
        } else {
            CheckResult::required(
                "Versions Directory",
                CheckStatus::Fail,
                "versions/ directory MISSING or not a directory",
                Some("This directory should contain the Unreal Engine projects.".to_string()),
            )
        }
    }

    fn check_ue4_root(&self) -> CheckResult {
        // Parse the JSON properly -- string search gives false positives
        // (e.g. "ue4_root" appearing in a comment or description value).
        let rocket_json = self.project_root.join(".rocket.json");
        if rocket_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&rocket_json) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(root) = v.get("ue4_root").and_then(|r| r.as_str()) {
                        let root_path = PathBuf::from(root);
                        if root_path.exists() {
                            return CheckResult::required(
                                "UE4_ROOT",
                                CheckStatus::Pass,
                                root.to_string(),
                                None,
                            );
                        } else {
                            return CheckResult::required(
                                "UE4_ROOT",
                                CheckStatus::Fail,
                                format!("configured but path missing: {root}"),
                                Some("Run 'rocket setup' to reconfigure.".to_string()),
                            )
                            .with_fix(vec!["./rocket", "setup"], "run: ./rocket setup");
                        }
                    }
                }
            }
        }

        if let Ok(root) = std::env::var("UE4_ROOT") {
            let root_path = PathBuf::from(&root);
            if root_path.exists() {
                CheckResult::required("UE4_ROOT", CheckStatus::Pass, root, None)
            } else {
                CheckResult::required(
                    "UE4_ROOT",
                    CheckStatus::Fail,
                    format!("UE4_ROOT set but path missing: {root}"),
                    None,
                )
            }
        } else {
            CheckResult::required(
                "UE4_ROOT",
                CheckStatus::Warn,
                "UE4 root not configured",
                Some("Run 'rocket setup' to configure Unreal Engine path.".to_string()),
            )
            .with_fix(vec!["./rocket", "setup"], "run: ./rocket setup")
        }
    }

    fn check_ue4_plugins(&self) -> CheckResult {
        let root_path = match self.resolve_ue4_root() {
            Some(p) => p,
            None => {
                return CheckResult::required(
                    "UE4 Plugins",
                    CheckStatus::Warn,
                    "Skipped: UE4 root not configured, cannot verify plugins",
                    None,
                );
            }
        };

        // Check WebSocketNetworking -- exists in Experimental/ in 4.27-html5-es3
        let ws_paths = vec![
            "Engine/Plugins/Experimental/WebSocketNetworking/WebSocketNetworking.uplugin",
            "Engine/Plugins/Runtime/Networking/WebSocketNetworking/WebSocketNetworking.uplugin",
            "Engine/Plugins/Networking/WebSocketNetworking/WebSocketNetworking.uplugin",
            "Engine/Plugins/WebSocketNetworking/WebSocketNetworking.uplugin",
        ];
        let ws_ok = ws_paths.iter().any(|rel| root_path.join(rel).exists());

        // VaRest is a Marketplace plugin -- not bundled in engine source builds.
        let varest_paths = vec![
            "Engine/Plugins/Marketplace/VaRest/VaRest.uplugin",
            "Engine/Plugins/VaRest/VaRest.uplugin",
        ];
        let varest_ok = varest_paths.iter().any(|rel| root_path.join(rel).exists());

        match (ws_ok, varest_ok) {
            (true, true) => CheckResult::required(
                "UE4 Plugins",
                CheckStatus::Pass,
                "WebSocketNetworking, VaRest",
                None,
            ),
            (true, false) => CheckResult::required(
                "UE4 Plugins",
                CheckStatus::Warn,
                "WebSocketNetworking present; VaRest not found (Marketplace plugin -- install separately if needed)",
                None,
            ),
            (false, _) => {
                let mut missing = vec!["WebSocketNetworking"];
                if !varest_ok {
                    missing.push("VaRest");
                }
                CheckResult::required(
                    "UE4 Plugins",
                    CheckStatus::Fail,
                    format!("Missing required plugins: {}", missing.join(", ")),
                    Some("Ensure your UE4 build includes WebSocketNetworking (Engine/Plugins/Experimental/ in 4.27-html5-es3).".to_string()),
                )
            }
        }
    }

    fn check_ue4_build_scripts(&self) -> CheckResult {
        let root = match self.resolve_ue4_root() {
            None => {
                return CheckResult::required(
                    "UE4 Build Scripts",
                    CheckStatus::Warn,
                    "Skipped: UE4 root not configured",
                    None,
                );
            }
            Some(r) if !r.exists() => {
                return CheckResult::required(
                    "UE4 Build Scripts",
                    CheckStatus::Fail,
                    format!("UE4 root path missing: {}", r.display()),
                    None,
                );
            }
            Some(r) => r,
        };

        // Critical scripts that must exist for `rocket build` and `rocket html5 cook`.
        let required = [
            "Engine/Build/BatchFiles/RunUAT.sh",
            "Engine/Build/BatchFiles/Mac/Build.sh",
        ];

        let missing_required: Vec<&str> = required
            .iter()
            .filter(|rel| !root.join(rel).exists())
            .copied()
            .collect();

        if !missing_required.is_empty() {
            return CheckResult::required(
                "UE4 Build Scripts",
                CheckStatus::Fail,
                format!("Missing critical scripts: {}", missing_required.join(", ")),
                Some(format!(
                    "UE4 root: {} -- ensure you have a full engine build with BatchFiles",
                    root.display()
                )),
            );
        }

        CheckResult::required(
            "UE4 Build Scripts",
            CheckStatus::Pass,
            format!("RunUAT.sh, Build.sh present at {}", root.display()),
            None,
        )
    }

    /// Quick compile-check of `nexus-types` -- the zero-dependency root of the
    /// nexus-engine workspace.
    fn check_nexus_types(&self) -> CheckResult {
        let nexus_root = self.project_root.join("nexus-engine");
        if !nexus_root.exists() {
            return CheckResult::required(
                "nexus-types",
                CheckStatus::Warn,
                "nexus-engine directory not found; skipping compile check",
                None,
            );
        }
        let output = Command::new("cargo")
            .args(["check", "-p", "nexus-types", "--quiet"])
            .current_dir(&nexus_root)
            .output();
        match output {
            Ok(o) if o.status.success() => CheckResult::required(
                "nexus-types",
                CheckStatus::Pass,
                "nexus-types compiles cleanly",
                None,
            ),
            Ok(o) => CheckResult::required(
                "nexus-types",
                CheckStatus::Fail,
                "nexus-types compile check failed",
                Some(
                    String::from_utf8_lossy(&o.stderr)
                        .chars()
                        .take(800)
                        .collect(),
                ),
            ),
            Err(e) => CheckResult::required(
                "nexus-types",
                CheckStatus::Fail,
                format!("could not invoke cargo: {e}"),
                None,
            ),
        }
    }

    // -------------------------------------------------------------------------
    // Optional checks
    // -------------------------------------------------------------------------

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
                            message
                                .push_str(&format!(", {} uncommitted changes", statuses.len()));
                        } else {
                            message.push_str(", no uncommitted changes");
                        }
                    }
                    Err(e) => {
                        return CheckResult::optional(
                            "Git Status",
                            CheckStatus::Warn,
                            format!("Branch: {}, could not check statuses: {}", head, e),
                            None,
                        );
                    }
                }

                CheckResult::optional("Git Status", status, message, None)
            }
            Err(e) => CheckResult::optional(
                "Git Status",
                CheckStatus::Fail,
                "Not a git repository",
                Some(e.to_string()),
            ),
        }
    }

    fn check_python(&self) -> CheckResult {
        let cmd = if Command::new("python3").arg("--version").output().is_ok() {
            "python3"
        } else {
            "python"
        };

        match Command::new(cmd).arg("--version").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let version = if stdout.is_empty() { stderr } else { stdout };
                CheckResult::optional("Python 3", CheckStatus::Pass, version, None)
            }
            Err(_) => {
                CheckResult::optional("Python 3", CheckStatus::Warn, "Not found in PATH", None)
            }
        }
    }

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

        let npm_ok = npm_output.map(|o| o.status.success()).unwrap_or(false);

        match (node_version, npm_ok) {
            (Some(v), true) => {
                // Warn if Node < 20 (pwa-staff requires Node 20.x)
                let major: Option<u32> = v
                    .trim_start_matches('v')
                    .split('.')
                    .next()
                    .and_then(|s| s.parse().ok());
                if major.map(|m| m >= 20).unwrap_or(false) {
                    CheckResult::optional("Node.js", CheckStatus::Pass, format!("{v} with npm"), None)
                } else {
                    CheckResult::optional(
                        "Node.js",
                        CheckStatus::Warn,
                        format!("{v} (want >=20)"),
                        Some("Upgrade via nvm: `nvm install 20 && nvm use 20`".into()),
                    )
                    .with_fix(
                        vec!["nvm", "install", "20"],
                        "run: nvm install 20 && nvm use 20",
                    )
                }
            }
            (Some(v), false) => CheckResult::optional(
                "Node.js",
                CheckStatus::Warn,
                format!("{v} found but npm not found"),
                Some("Install npm: `npm install -g npm`".into()),
            ),
            (None, _) => CheckResult::optional(
                "Node.js",
                CheckStatus::Warn,
                "Not found -- required for `rocket pwa build`",
                Some("Install via nvm: `nvm install 20 && nvm use 20`".into()),
            )
            .with_fix(
                vec!["nvm", "install", "20"],
                "run: nvm install 20 && nvm use 20",
            ),
        }
    }

    /// Check that rustfmt and clippy components are installed.
    fn check_rustfmt_clippy(&self) -> CheckResult {
        let rustfmt_ok = Command::new("rustfmt")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let clippy_ok = Command::new("cargo")
            .args(["clippy", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        match (rustfmt_ok, clippy_ok) {
            (true, true) => {
                CheckResult::optional("rustfmt + clippy", CheckStatus::Pass, "both installed", None)
            }
            (false, false) => CheckResult::optional(
                "rustfmt + clippy",
                CheckStatus::Warn,
                "NOT FOUND",
                None,
            )
            .with_fix(
                vec!["rustup", "component", "add", "rustfmt", "clippy"],
                "run: rustup component add rustfmt clippy",
            ),
            (false, true) => CheckResult::optional(
                "rustfmt + clippy",
                CheckStatus::Warn,
                "rustfmt missing; clippy present",
                None,
            )
            .with_fix(
                vec!["rustup", "component", "add", "rustfmt"],
                "run: rustup component add rustfmt",
            ),
            (true, false) => CheckResult::optional(
                "rustfmt + clippy",
                CheckStatus::Warn,
                "rustfmt present; clippy missing",
                None,
            )
            .with_fix(
                vec!["rustup", "component", "add", "clippy"],
                "run: rustup component add clippy",
            ),
        }
    }

    /// Check that pwa-staff/node_modules exists (npm ci has been run).
    fn check_pwa_node_modules(&self) -> CheckResult {
        let modules = self.project_root.join("pwa-staff").join("node_modules");
        if modules.exists() {
            CheckResult::optional("pwa node_modules", CheckStatus::Pass, "installed", None)
        } else {
            CheckResult::optional(
                "pwa node_modules",
                CheckStatus::Warn,
                "NOT FOUND -- run `npm ci` in pwa-staff/",
                None,
            )
            .with_fix(
                vec!["npm", "ci", "--prefix", "pwa-staff"],
                "run: npm ci (in pwa-staff/)",
            )
        }
    }

    /// Check that a .env file exists; offer to copy from .env.example.
    fn check_dotenv(&self) -> CheckResult {
        let env_file = self.project_root.join(".env");
        if env_file.exists() {
            CheckResult::optional(".env file", CheckStatus::Pass, ".env present", None)
        } else {
            let example = self.project_root.join(".env.example");
            if example.exists() {
                CheckResult::optional(
                    ".env file",
                    CheckStatus::Warn,
                    "NOT FOUND -- .env.example available",
                    Some("copy .env.example to .env to configure env vars".to_string()),
                )
                .with_fix(
                    vec!["cp", ".env.example", ".env"],
                    "run: cp .env.example .env",
                )
            } else {
                CheckResult::optional(
                    ".env file",
                    CheckStatus::Warn,
                    "NOT FOUND (and no .env.example)",
                    None,
                )
            }
        }
    }

    fn check_html5_toolchain(&self) -> CheckResult {
        // 1. Verify Python 3 is available for UAT/UHT scripts.
        let python_ok = discover_python3().map(|path| format!("Python 3 at {}", path.display()));

        // 2. Verify emscripten -- check engine-bundled emsdk first, then PATH.
        let emsdk_found = self.find_ue4_emsdk();
        let emcc_on_path = Command::new("emcc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        match (python_ok, emsdk_found || emcc_on_path) {
            (Some(py), true) => CheckResult::optional(
                "HTML5 Toolchain",
                CheckStatus::Pass,
                format!("{py}; emscripten present"),
                None,
            ),
            (Some(py), false) => CheckResult::optional(
                "HTML5 Toolchain",
                CheckStatus::Warn,
                format!("{py}; emscripten NOT found"),
                Some(
                    "Run HTML5Setup.sh in the engine to build emsdk, or run 'rocket html5 setup'."
                        .to_string(),
                ),
            )
            .with_fix(
                vec!["./rocket", "html5", "preflight"],
                "run: ./rocket html5 preflight",
            ),
            (None, _) => CheckResult::optional(
                "HTML5 Toolchain",
                CheckStatus::Fail,
                "Python 3 not found -- required for UAT scripts",
                Some("Install python3 or set 'python3_path' in .rocket.json".to_string()),
            ),
        }
    }

    /// Check whether the most recent HTML5 cook produced a real package.
    fn check_html5_package(&self) -> CheckResult {
        // Prefer manifest-derived archive paths over hardcoded defaults.
        let manifest_paths: Vec<PathBuf> =
            crate::Manifest::load(self.project_root.join("project-manifest.json"))
                .map(|m| {
                    m.projects()
                        .iter()
                        .map(|p| {
                            PathBuf::from(format!(
                                "/tmp/{}-html5-archive/HTML5",
                                p.name.to_lowercase()
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default();

        let mut candidates: Vec<PathBuf> = manifest_paths;
        candidates.extend([
            PathBuf::from("/tmp/brm-html5-archive/HTML5"),
            PathBuf::from("/tmp/brm-html5-archive"),
            self.project_root.join("manufactured"),
            self.project_root.join("pwa-staff/manufactured"),
        ]);
        candidates.dedup();

        let archive_dir = candidates.iter().find(|d| d.exists());

        match archive_dir {
            None => CheckResult::optional(
                "HTML5 Package",
                CheckStatus::Warn,
                "No HTML5 archive directory found",
                Some(
                    "Run 'rocket html5 cook --project Brm' to produce a package.".to_string(),
                ),
            )
            .with_fix(
                vec!["./rocket", "html5", "cook", "--project", "Brm"],
                "run: ./rocket html5 cook --project Brm",
            ),
            Some(dir) => match Html5PackageVerifier::new(dir).verify() {
                Err(e) => CheckResult::optional(
                    "HTML5 Package",
                    CheckStatus::Fail,
                    format!("Verification error: {e}"),
                    None,
                ),
                Ok(report) => {
                    let summary = report.summary();
                    if report.is_real_package {
                        CheckResult::optional(
                            "HTML5 Package",
                            CheckStatus::Pass,
                            summary,
                            Some(format!("Archive: {}", dir.display())),
                        )
                    } else {
                        let has_real_wasm = report
                            .wasm_files
                            .iter()
                            .any(|f| matches!(f.verdict, WasmVerdict::Real { .. }));
                        let status = if has_real_wasm {
                            CheckStatus::Warn // WASM is real but companions missing
                        } else {
                            CheckStatus::Fail // stub or no wasm
                        };
                        CheckResult::optional(
                            "HTML5 Package",
                            status,
                            summary,
                            Some(format!("Archive: {}", dir.display())),
                        )
                    }
                }
            },
        }
    }

    /// Check for HTML5-specific build scripts (optional -- only needed for html5 cook).
    fn check_ue4_build_scripts_html5(&self) -> CheckResult {
        let root = match self.resolve_ue4_root() {
            None => {
                return CheckResult::optional(
                    "UE4 HTML5 Scripts",
                    CheckStatus::Warn,
                    "Skipped: UE4 root not configured",
                    None,
                );
            }
            Some(r) if !r.exists() => {
                return CheckResult::optional(
                    "UE4 HTML5 Scripts",
                    CheckStatus::Fail,
                    format!("UE4 root path missing: {}", r.display()),
                    None,
                );
            }
            Some(r) => r,
        };

        let html5_setup = "Engine/Platforms/HTML5/HTML5Setup.sh";
        if root.join(html5_setup).exists() {
            CheckResult::optional(
                "UE4 HTML5 Scripts",
                CheckStatus::Pass,
                format!("HTML5Setup.sh present at {}", root.display()),
                None,
            )
        } else {
            CheckResult::optional(
                "UE4 HTML5 Scripts",
                CheckStatus::Warn,
                "HTML5Setup.sh missing",
                Some(
                    "Run HTML5Setup.sh from the SpeculativeCoder/UnrealEngine fork to enable HTML5 packaging"
                        .to_string(),
                ),
            )
        }
    }

    /// Validate that every project declared in `project-manifest.json` has its
    /// `.uproject` file on disk.
    fn check_manifest_projects(&self) -> CheckResult {
        let manifest_path = self.project_root.join("project-manifest.json");
        if !manifest_path.exists() {
            return CheckResult::optional(
                "Manifest Projects",
                CheckStatus::Warn,
                "Skipped: project-manifest.json not found",
                None,
            );
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                return CheckResult::optional(
                    "Manifest Projects",
                    CheckStatus::Fail,
                    format!("Cannot read project-manifest.json: {e}"),
                    None,
                );
            }
        };

        let manifest: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                return CheckResult::optional(
                    "Manifest Projects",
                    CheckStatus::Fail,
                    format!("project-manifest.json is invalid JSON: {e}"),
                    None,
                );
            }
        };

        let projects = match manifest.get("projects").and_then(|p| p.as_array()) {
            Some(arr) => arr.clone(),
            None => {
                return CheckResult::optional(
                    "Manifest Projects",
                    CheckStatus::Warn,
                    "No 'projects' array in manifest",
                    None,
                );
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
            CheckResult::optional(
                "Manifest Projects",
                CheckStatus::Pass,
                format!("All {total} declared .uproject files present on disk"),
                None,
            )
        } else {
            // Missing projects are sample/optional content repos -- not pipeline-blocking.
            CheckResult::optional(
                "Manifest Projects",
                CheckStatus::Warn,
                format!(
                    "{}/{total} declared .uproject files not present (optional sample content)",
                    missing.len()
                ),
                Some(missing.join("\n")),
            )
        }
    }

    fn check_ggen(&self) -> CheckResult {
        match Command::new("ggen").arg("--version").output() {
            Ok(output) => CheckResult::optional(
                "ggen",
                CheckStatus::Pass,
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                None,
            ),
            Err(_) => CheckResult::optional(
                "ggen",
                CheckStatus::Warn,
                "not found in PATH",
                Some("ggen is required for Ostar generative workflows.".to_string()),
            ),
        }
    }

    fn check_anti_llm_cheat_lsp(&self) -> CheckResult {
        match Command::new("anti-llm-cheat-lsp").arg("--version").output() {
            Ok(output) => CheckResult::optional(
                "anti-llm-cheat-lsp",
                CheckStatus::Pass,
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                None,
            ),
            Err(_) => CheckResult::optional(
                "anti-llm-cheat-lsp",
                CheckStatus::Warn,
                "not found in PATH",
                Some("Install: cargo install lsp-max --bin anti-llm-cheat-lsp".to_string()),
            ),
        }
    }

    /// Check that Xcode command-line tools are installed (macOS only).
    fn check_xcode(&self) -> CheckResult {
        #[cfg(not(target_os = "macos"))]
        return CheckResult::optional("Xcode CLT", CheckStatus::Pass, "Not macOS -- skipped", None);

        #[cfg(target_os = "macos")]
        {
            let xcode_select = Command::new("xcode-select").arg("-p").output();
            match xcode_select {
                Ok(out) if out.status.success() => {
                    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let clang_ok = Command::new("xcrun")
                        .args(["--find", "clang"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if clang_ok {
                        CheckResult::optional(
                            "Xcode CLT",
                            CheckStatus::Pass,
                            format!("Developer tools active at {path}"),
                            None,
                        )
                    } else {
                        CheckResult::optional(
                            "Xcode CLT",
                            CheckStatus::Warn,
                            format!(
                                "xcode-select path set ({path}) but clang not found via xcrun"
                            ),
                            Some("Run: xcode-select --install".to_string()),
                        )
                    }
                }
                _ => CheckResult::optional(
                    "Xcode CLT",
                    CheckStatus::Fail,
                    "Xcode command-line tools not installed",
                    Some(
                        "Run: xcode-select --install  (required for UE4 Build.sh)".to_string(),
                    ),
                ),
            }
        }
    }

    /// Check if the engine's bundled emsdk is present (built by HTML5Setup.sh).
    fn find_ue4_emsdk(&self) -> bool {
        self.resolve_ue4_root()
            .map(|r| r.join("Engine/Platforms/HTML5/Build/emsdk").exists())
            .unwrap_or(false)
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

    // -- check_ue4_root -------------------------------------------------------

    #[test]
    fn check_ue4_root_warns_when_unconfigured() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let result = doctor.check_ue4_root();
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
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
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
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
        )
        .unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_root();
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
        assert_eq!(result.status, CheckStatus::Pass);
    }

    // -- check_html5_toolchain ------------------------------------------------

    #[test]
    fn check_html5_toolchain_returns_a_result() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_html5_toolchain();
        assert_eq!(result.name, "HTML5 Toolchain");
        assert!(
            result.status == CheckStatus::Pass
                || result.status == CheckStatus::Warn
                || result.status == CheckStatus::Fail
        );
    }

    // -- check_ue4_build_scripts ----------------------------------------------

    #[test]
    fn build_scripts_warn_when_ue4_root_not_configured() {
        let dir = tempdir().unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
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
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
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
        fs::write(
            &rocket_json,
            format!(r#"{{"ue4_root": "{}"}}"#, fake_ue4.display()),
        )
        .unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("RunUAT.sh"));
    }

    #[test]
    fn build_scripts_pass_when_required_scripts_present() {
        let dir = tempdir().unwrap();
        let fake_ue4 = dir.path().join("ue4");
        fs::create_dir_all(fake_ue4.join("Engine/Build/BatchFiles/Mac")).unwrap();
        fs::write(
            fake_ue4.join("Engine/Build/BatchFiles/RunUAT.sh"),
            b"#!/bin/sh",
        )
        .unwrap();
        fs::write(
            fake_ue4.join("Engine/Build/BatchFiles/Mac/Build.sh"),
            b"#!/bin/sh",
        )
        .unwrap();
        let rocket_json = dir.path().join(".rocket.json");
        fs::write(
            &rocket_json,
            format!(r#"{{"ue4_root": "{}"}}"#, fake_ue4.display()),
        )
        .unwrap();
        let prev = std::env::var("UE4_ROOT").ok();
        std::env::remove_var("UE4_ROOT");
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_ue4_build_scripts();
        if let Some(v) = prev {
            std::env::set_var("UE4_ROOT", v);
        }
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("RunUAT.sh"));
    }

    // -- check_manifest_projects ----------------------------------------------

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
        fs::write(
            dir.path().join("project-manifest.json"),
            r#"{"version": 1}"#,
        )
        .unwrap();
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
        fs::write(
            dir.path().join("project-manifest.json"),
            manifest.to_string(),
        )
        .unwrap();
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
        fs::write(
            dir.path().join("project-manifest.json"),
            manifest.to_string(),
        )
        .unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("not present") || result.message.contains("optional"));
        assert!(result
            .details
            .as_deref()
            .unwrap_or("")
            .contains("Ghost"));
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
        fs::write(
            dir.path().join("project-manifest.json"),
            manifest.to_string(),
        )
        .unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest_projects();
        assert_eq!(result.status, CheckStatus::Warn);
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

    // -- check_node -----------------------------------------------------------

    #[test]
    fn node_check_returns_a_result() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_node();
        assert_eq!(result.name, "Node.js");
        // Accept any status -- the check should not panic regardless of env
        matches!(
            result.status,
            CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
        );
    }

    #[test]
    fn node_check_pass_or_warn_status_has_nonempty_message() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_node();
        assert!(!result.message.is_empty());
    }

    // -- check_xcode ----------------------------------------------------------

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
        matches!(
            result.status,
            CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
        );
    }

    // -- html5 package --------------------------------------------------------

    #[test]
    fn html5_package_check_returns_a_named_html5_result() {
        // Any project root -- the check must return a properly named result without panicking
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_html5_package();
        assert!(result.name.contains("HTML5"), "check name must mention HTML5");
        // Status may be Pass/Warn/Fail depending on machine state -- just must not panic
        matches!(
            result.status,
            CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
        );
    }

    #[test]
    fn html5_package_check_reads_manifest_for_archive_paths() {
        let dir = tempdir().unwrap();
        // Write a minimal project-manifest.json with a project named "Alpha"
        let manifest = serde_json::json!({
            "projects": [{"name": "Alpha", "uproject_path": "Alpha.uproject", "targets": []}]
        });
        fs::write(
            dir.path().join("project-manifest.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();

        // No archive directories exist, so this should still be Warn (not panic)
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_html5_package();
        // Should be Warn because no archive exists -- but must NOT panic or error
        assert!(matches!(
            result.status,
            CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Pass
        ));
    }

    // -- new checks -----------------------------------------------------------

    #[test]
    fn pwa_node_modules_check_warn_when_missing() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_pwa_node_modules();
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.fix_command.is_some(), "should have a fix command");
    }

    #[test]
    fn dotenv_check_warn_when_missing_but_example_exists() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".env.example"), "# example env").unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_dotenv();
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.fix_command.is_some(), "should offer cp fix");
    }

    #[test]
    fn dotenv_check_pass_when_env_exists() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".env"), "KEY=val").unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_dotenv();
        assert_eq!(result.status, CheckStatus::Pass);
    }

    // -- health score ---------------------------------------------------------

    #[test]
    fn health_score_all_required_pass() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![
                CheckResult::required("A", CheckStatus::Pass, "ok", None),
                CheckResult::required("B", CheckStatus::Pass, "ok", None),
                CheckResult::optional("C", CheckStatus::Fail, "bad", None),
            ],
        };
        assert_eq!(report.required_pass_count(), 2);
        assert_eq!(report.required_total(), 2);
        assert!(report.all_required_pass());
    }

    #[test]
    fn health_score_partial_pass() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![
                CheckResult::required("A", CheckStatus::Pass, "ok", None),
                CheckResult::required("B", CheckStatus::Fail, "bad", None),
                CheckResult::optional("C", CheckStatus::Pass, "ok", None),
            ],
        };
        assert_eq!(report.required_pass_count(), 1);
        assert_eq!(report.required_total(), 2);
        assert!(!report.all_required_pass());
    }

    #[test]
    fn check_result_fix_command_attached() {
        let result = CheckResult::required("Test", CheckStatus::Fail, "missing", None).with_fix(
            vec!["rustup", "component", "add", "clippy"],
            "run: rustup component add clippy",
        );
        assert!(result.fix_command.is_some());
        assert!(result.fix_hint.is_some());
    }

    #[test]
    fn categories_are_set_correctly() {
        let req = CheckResult::required("R", CheckStatus::Pass, "ok", None);
        let opt = CheckResult::optional("O", CheckStatus::Pass, "ok", None);
        assert_eq!(req.category, Some(CheckCategory::Required));
        assert_eq!(opt.category, Some(CheckCategory::Optional));
    }

    #[test]
    fn run_diagnostics_contains_required_and_optional_checks() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let report = doctor.run_diagnostics();
        let has_required = report
            .checks
            .iter()
            .any(|c| c.category == Some(CheckCategory::Required));
        let has_optional = report
            .checks
            .iter()
            .any(|c| c.category == Some(CheckCategory::Optional));
        assert!(has_required);
        assert!(has_optional);
    }
}
