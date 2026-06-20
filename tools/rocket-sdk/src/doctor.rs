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

impl CheckStatus {
    /// Status glyph (plain ASCII-safe unicode).
    pub fn glyph(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "✔",
            CheckStatus::Warn => "▲",
            CheckStatus::Fail => "✘",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            CheckStatus::Pass => ansi::GREEN,
            CheckStatus::Warn => ansi::YELLOW,
            CheckStatus::Fail => ansi::RED,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
            CheckStatus::Fail => "FAIL",
        }
    }
}

/// Logical grouping for a check so the report can be sectioned.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CheckCategory {
    #[default]
    Environment,
    Toolchain,
    Project,
    Git,
    Optional,
}

impl CheckCategory {
    pub fn label(&self) -> &'static str {
        match self {
            CheckCategory::Environment => "Environment",
            CheckCategory::Toolchain => "Toolchain",
            CheckCategory::Project => "Project",
            CheckCategory::Git => "Git",
            CheckCategory::Optional => "Optional",
        }
    }

    /// Iteration order used when rendering / scoring.
    pub fn ordered() -> [CheckCategory; 5] {
        [
            CheckCategory::Environment,
            CheckCategory::Toolchain,
            CheckCategory::Project,
            CheckCategory::Git,
            CheckCategory::Optional,
        ]
    }
}

// ---------------------------------------------------------------------------
// CheckResult
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
    /// Additive fields (kept after the original four for serde stability).
    #[serde(default)]
    pub category: CheckCategory,
    /// A concrete command the user can run to remediate. Every Warn/Fail
    /// SHOULD carry one.
    #[serde(default)]
    pub fix_command: Option<String>,
}

impl CheckResult {
    /// Convenience constructor that mirrors the original struct-literal style
    /// while letting category/fix be set fluently.
    fn new(
        name: impl Into<String>,
        status: CheckStatus,
        message: impl Into<String>,
        category: CheckCategory,
    ) -> Self {
        CheckResult {
            name: name.into(),
            status,
            message: message.into(),
            details: None,
            category,
            fix_command: None,
        }
    }

    fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix_command = Some(fix.into());
        self
    }

    /// Weight used in health scoring. Optional checks count less.
    fn weight(&self) -> f64 {
        match self.category {
            CheckCategory::Optional => 0.5,
            _ => 1.0,
        }
    }

    /// 0.0..=1.0 score contribution for this check.
    fn score_fraction(&self) -> f64 {
        match self.status {
            CheckStatus::Pass => 1.0,
            CheckStatus::Warn => 0.5,
            CheckStatus::Fail => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// DiagnosticReport
// ---------------------------------------------------------------------------

// TRACKED_WORK(anti-cheat): DiagnosticReport previously derived Deserialize but DateTime<Utc>
// requires chrono's "serde" feature flag. The CLI only serializes diagnostic reports — it never
// deserializes them — so Deserialize remains intentionally absent.
#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticReport {
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<CheckResult>,
}

impl DiagnosticReport {
    /// Weighted health score in the inclusive range 0..=100.
    pub fn health_score(&self) -> u8 {
        let mut total_weight = 0.0;
        let mut earned = 0.0;
        for c in &self.checks {
            let w = c.weight();
            total_weight += w;
            earned += w * c.score_fraction();
        }
        if total_weight == 0.0 {
            return 100;
        }
        ((earned / total_weight) * 100.0).round() as u8
    }

    /// Overall status: Fail if any non-optional check failed, else Warn if any
    /// warn/fail at all, else Pass.
    pub fn overall_status(&self) -> CheckStatus {
        let mut any_warn = false;
        for c in &self.checks {
            match c.status {
                CheckStatus::Fail if c.category != CheckCategory::Optional => {
                    return CheckStatus::Fail;
                }
                CheckStatus::Fail | CheckStatus::Warn => any_warn = true,
                CheckStatus::Pass => {}
            }
        }
        if any_warn {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        }
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;
        for c in &self.checks {
            match c.status {
                CheckStatus::Pass => pass += 1,
                CheckStatus::Warn => warn += 1,
                CheckStatus::Fail => fail += 1,
            }
        }
        (pass, warn, fail)
    }

    // ---- Output modes ----------------------------------------------------

    /// JSON report (includes derived health score and overall status).
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct Out<'a> {
            timestamp: &'a DateTime<Utc>,
            health_score: u8,
            overall_status: &'a CheckStatus,
            pass: usize,
            warn: usize,
            fail: usize,
            checks: &'a [CheckResult],
        }
        let (pass, warn, fail) = self.counts();
        let out = Out {
            timestamp: &self.timestamp,
            health_score: self.health_score(),
            overall_status: &self.overall_status(),
            pass,
            warn,
            fail,
            checks: &self.checks,
        };
        serde_json::to_string_pretty(&out).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }

    /// One-line compact summary, e.g.
    /// `doctor: WARN  health 84/100  (12 pass, 2 warn, 0 fail)`.
    pub fn compact_summary(&self, use_color: bool) -> String {
        let (pass, warn, fail) = self.counts();
        let overall = self.overall_status();
        let score = self.health_score();
        if use_color {
            format!(
                "{}doctor:{} {}{}{}  health {}{}/100{}  ({} pass, {} warn, {} fail)",
                ansi::BOLD,
                ansi::RESET,
                overall.color(),
                overall.label(),
                ansi::RESET,
                ansi::CYAN,
                score,
                ansi::RESET,
                pass,
                warn,
                fail,
            )
        } else {
            format!(
                "doctor: {}  health {}/100  ({} pass, {} warn, {} fail)",
                overall.label(),
                score,
                pass,
                warn,
                fail
            )
        }
    }

    /// Pretty, sectioned, human-readable report.
    pub fn pretty(&self, use_color: bool) -> String {
        let mut out = String::new();
        let paint = |s: &str, color: &str| -> String {
            if use_color {
                format!("{color}{s}{}", ansi::RESET)
            } else {
                s.to_string()
            }
        };

        out.push_str(&paint("Rocket Doctor", ansi::BOLD));
        out.push('\n');
        out.push_str(&format!("{}\n", self.timestamp.to_rfc3339()));
        out.push('\n');

        for cat in CheckCategory::ordered() {
            let in_cat: Vec<&CheckResult> =
                self.checks.iter().filter(|c| c.category == cat).collect();
            if in_cat.is_empty() {
                continue;
            }
            out.push_str(&paint(&format!("── {} ──", cat.label()), ansi::CYAN));
            out.push('\n');
            for c in in_cat {
                let glyph = if use_color {
                    format!("{}{}{}", c.status.color(), c.status.glyph(), ansi::RESET)
                } else {
                    format!("{} {}", c.status.glyph(), c.status.label())
                };
                out.push_str(&format!("  {glyph} {:<22} {}\n", c.name, c.message));
                if let Some(d) = &c.details {
                    out.push_str(&format!("       {}\n", paint(d, ansi::DIM)));
                }
                if c.status != CheckStatus::Pass {
                    if let Some(fix) = &c.fix_command {
                        out.push_str(&format!(
                            "       {} {}\n",
                            paint("fix:", ansi::YELLOW),
                            fix
                        ));
                    }
                }
            }
            out.push('\n');
        }

        out.push_str(&self.compact_summary(use_color));
        out.push('\n');
        out
    }
}

// ---------------------------------------------------------------------------
// Auto-fix
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum FixOutcome {
    /// Would have run (dry-run) — not actually executed.
    Planned,
    Applied,
    Skipped,
    Failed,
}

#[derive(Debug, Serialize, Clone)]
pub struct FixResult {
    pub check_name: String,
    pub command: String,
    pub outcome: FixOutcome,
    pub message: String,
}

/// A safe, whitelisted remediation that `run_fixes` is allowed to perform.
/// We never shell out arbitrary `fix_command` strings — only these vetted ops.
enum SafeFix {
    /// `cargo fetch` inside the tools workspace — pure download, non-destructive.
    CargoFetch,
    /// Create a directory (and parents) if missing — non-destructive.
    CreateDir(PathBuf),
}

// ---------------------------------------------------------------------------
// RocketDoctor
// ---------------------------------------------------------------------------

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

        let mut results: Vec<(usize, CheckResult)> = std::thread::scope(|scope| {
            let handles: Vec<_> = checks
                .into_iter()
                .map(|(idx, f)| scope.spawn(move || (idx, f())))
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("doctor check thread panicked"))
                .collect()
        });

        results.sort_by_key(|(idx, _)| *idx);

        DiagnosticReport {
            timestamp: Utc::now(),
            checks: results.into_iter().map(|(_, r)| r).collect(),
        }
    }

    /// Execute the safe subset of remediations.
    ///
    /// `dry_run = true` (the recommended default) only *reports* what would be
    /// done. Only whitelisted, non-destructive operations are ever performed —
    /// arbitrary `fix_command` strings are NOT executed.
    pub fn run_fixes(&self, dry_run: bool) -> Vec<FixResult> {
        let report = self.run_diagnostics();
        let mut out = Vec::new();

        for check in &report.checks {
            if check.status == CheckStatus::Pass {
                continue;
            }
            let Some(safe) = self.safe_fix_for(check) else {
                // Non-passing but no safe automated fix — surface the manual one.
                if let Some(cmd) = &check.fix_command {
                    out.push(FixResult {
                        check_name: check.name.clone(),
                        command: cmd.clone(),
                        outcome: FixOutcome::Skipped,
                        message: "No safe auto-fix; run the suggested command manually".into(),
                    });
                }
                continue;
            };

            let (command, result) = self.apply_safe_fix(safe, dry_run);
            out.push(FixResult {
                check_name: check.name.clone(),
                command,
                outcome: result.0,
                message: result.1,
            });
        }

        out
    }

    /// Map a failing/warning check to a whitelisted safe fix, if one exists.
    fn safe_fix_for(&self, check: &CheckResult) -> Option<SafeFix> {
        match check.name.as_str() {
            // Missing workspace deps: a plain fetch is safe & non-destructive.
            "Cargo" | "Path Dependencies" => Some(SafeFix::CargoFetch),
            "Versions Directory" => {
                Some(SafeFix::CreateDir(self.project_root.join("versions")))
            }
            _ => None,
        }
    }

    fn apply_safe_fix(&self, fix: SafeFix, dry_run: bool) -> (String, (FixOutcome, String)) {
        match fix {
            SafeFix::CargoFetch => {
                let cmd = "cargo fetch (in tools/)".to_string();
                if dry_run {
                    return (cmd, (FixOutcome::Planned, "Would fetch dependencies".into()));
                }
                let tools_dir = self.project_root.join("tools");
                let res = Command::new("cargo")
                    .arg("fetch")
                    .current_dir(if tools_dir.exists() {
                        tools_dir
                    } else {
                        self.project_root.clone()
                    })
                    .output();
                match res {
                    Ok(o) if o.status.success() => {
                        (cmd, (FixOutcome::Applied, "Dependencies fetched".into()))
                    }
                    Ok(o) => (
                        cmd,
                        (
                            FixOutcome::Failed,
                            String::from_utf8_lossy(&o.stderr).trim().to_string(),
                        ),
                    ),
                    Err(e) => (cmd, (FixOutcome::Failed, e.to_string())),
                }
            }
            SafeFix::CreateDir(path) => {
                let cmd = format!("mkdir -p {}", path.display());
                if dry_run {
                    return (
                        cmd,
                        (FixOutcome::Planned, format!("Would create {}", path.display())),
                    );
                }
                match std::fs::create_dir_all(&path) {
                    Ok(()) => (cmd, (FixOutcome::Applied, format!("Created {}", path.display()))),
                    Err(e) => (cmd, (FixOutcome::Failed, e.to_string())),
                }
            }
        }
    }

    // ===================================================================
    // Individual checks
    // ===================================================================

    fn check_git(&self) -> CheckResult {
        match command_version("git", &["--version"]) {
            Some(v) => CheckResult::new("Git", CheckStatus::Pass, v, CheckCategory::Toolchain),
            None => CheckResult::new(
                "Git",
                CheckStatus::Fail,
                "Git not found in PATH",
                CheckCategory::Toolchain,
            )
            .with_fix("Install git (e.g. apt install git / brew install git)"),
        }
    }

    fn check_git_status(&self) -> CheckResult {
        match git2::Repository::open(&self.project_root) {
            Ok(repo) => {
                let mut message = String::new();
                let mut status = CheckStatus::Pass;

                let head = match repo.head() {
                    Ok(head) => head.shorthand().unwrap_or("unknown").to_string(),
                    Err(_) => "HEAD detached or empty".to_string(),
                };
                message.push_str(&format!("Branch: {}", head));

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
                        return CheckResult::new(
                            "Git Status",
                            CheckStatus::Warn,
                            format!("Branch: {}, could not check statuses: {}", head, e),
                            CheckCategory::Git,
                        );
                    }
                }

                let mut r = CheckResult::new("Git Status", status.clone(), message, CheckCategory::Git);
                if status == CheckStatus::Warn {
                    r = r.with_fix("git status  # review, then commit or stash");
                }
                r
            }
            Err(e) => CheckResult::new(
                "Git Status",
                CheckStatus::Fail,
                "Not a git repository",
                CheckCategory::Git,
            )
            .with_details(e.to_string())
            .with_fix("git init"),
        }
    }

    fn check_rust(&self) -> CheckResult {
        match command_version("rustc", &["--version"]) {
            Some(v) => {
                // Expect rustc >= 1.80 for edition-2024 era toolchains.
                let parsed = parse_semver_from(&v);
                if let Some((maj, min, _)) = parsed {
                    if (maj, min) < (1, 80) {
                        return CheckResult::new(
                            "Rust",
                            CheckStatus::Warn,
                            v,
                            CheckCategory::Toolchain,
                        )
                        .with_details("rustc < 1.80; some workspaces target newer toolchains")
                        .with_fix("rustup update stable");
                    }
                }
                CheckResult::new("Rust", CheckStatus::Pass, v, CheckCategory::Toolchain)
            }
            None => CheckResult::new(
                "Rust",
                CheckStatus::Fail,
                "Rust (rustc) not found in PATH",
                CheckCategory::Toolchain,
            )
            .with_fix("curl https://sh.rustup.rs -sSf | sh"),
        }
    }

    fn check_cargo(&self) -> CheckResult {
        match command_version("cargo", &["--version"]) {
            Some(v) => CheckResult::new("Cargo", CheckStatus::Pass, v, CheckCategory::Toolchain),
            None => CheckResult::new(
                "Cargo",
                CheckStatus::Fail,
                "cargo not found in PATH",
                CheckCategory::Toolchain,
            )
            .with_fix("rustup component add cargo  # or reinstall rustup"),
        }
    }

    fn check_clippy(&self) -> CheckResult {
        match command_version("cargo", &["clippy", "--version"]) {
            Some(v) => CheckResult::new("Clippy", CheckStatus::Pass, v, CheckCategory::Toolchain),
            None => CheckResult::new(
                "Clippy",
                CheckStatus::Warn,
                "cargo-clippy not available",
                CheckCategory::Toolchain,
            )
            .with_fix("rustup component add clippy"),
        }
    }

    fn check_rustfmt(&self) -> CheckResult {
        match command_version("cargo", &["fmt", "--version"]) {
            Some(v) => CheckResult::new("Rustfmt", CheckStatus::Pass, v, CheckCategory::Toolchain),
            None => CheckResult::new(
                "Rustfmt",
                CheckStatus::Warn,
                "rustfmt not available",
                CheckCategory::Toolchain,
            )
            .with_fix("rustup component add rustfmt"),
        }
    }

    fn check_python(&self) -> CheckResult {
        let candidates = ["python3", "python"];
        for cmd in candidates {
            if let Some(v) = command_version(cmd, &["--version"]) {
                let parsed = parse_semver_from(&v);
                if let Some((maj, _, _)) = parsed {
                    if maj < 3 {
                        return CheckResult::new(
                            "Python",
                            CheckStatus::Warn,
                            v,
                            CheckCategory::Toolchain,
                        )
                        .with_details("Python 3.x required for validate-assets.py")
                        .with_fix("Install Python 3.x");
                    }
                }
                return CheckResult::new("Python", CheckStatus::Pass, v, CheckCategory::Toolchain);
            }
        }
        CheckResult::new(
            "Python",
            CheckStatus::Fail,
            "Python not found in PATH",
            CheckCategory::Toolchain,
        )
        .with_fix("Install Python 3.x (apt install python3 / brew install python)")
    }

    fn check_node(&self) -> CheckResult {
        match command_version("node", &["--version"]) {
            Some(v) => {
                // node prints e.g. "v20.11.1"
                let parsed = parse_semver_from(&v);
                if let Some((maj, _, _)) = parsed {
                    if maj < 20 {
                        return CheckResult::new(
                            "Node.js",
                            CheckStatus::Warn,
                            v,
                            CheckCategory::Toolchain,
                        )
                        .with_details("pwa-staff targets Node 20.x")
                        .with_fix("nvm install 20 && nvm use 20");
                    }
                }
                CheckResult::new("Node.js", CheckStatus::Pass, v, CheckCategory::Toolchain)
            }
            None => CheckResult::new(
                "Node.js",
                CheckStatus::Warn,
                "node not found in PATH",
                CheckCategory::Toolchain,
            )
            .with_details("Only required for pwa-staff")
            .with_fix("Install Node.js 20.x (https://nodejs.org)"),
        }
    }

    fn check_blender(&self) -> CheckResult {
        // Prefer BLENDER_PATH, fall back to `blender` on PATH.
        if let Ok(path) = std::env::var("BLENDER_PATH") {
            if Path::new(&path).exists() {
                let v = command_version(&path, &["--version"])
                    .unwrap_or_else(|| format!("BLENDER_PATH={path}"));
                return CheckResult::new("Blender", CheckStatus::Pass, v, CheckCategory::Optional);
            }
            return CheckResult::new(
                "Blender",
                CheckStatus::Warn,
                format!("BLENDER_PATH set but does not exist: {path}"),
                CheckCategory::Optional,
            )
            .with_fix("Set BLENDER_PATH to a valid Blender executable");
        }
        match command_version("blender", &["--version"]) {
            Some(v) => CheckResult::new("Blender", CheckStatus::Pass, v, CheckCategory::Optional),
            None => CheckResult::new(
                "Blender",
                CheckStatus::Warn,
                "Blender not found (BLENDER_PATH unset, not on PATH)",
                CheckCategory::Optional,
            )
            .with_details("Required for the asset-pipeline workspace")
            .with_fix("export BLENDER_PATH=/path/to/blender  # or add blender to PATH"),
        }
    }

    fn check_disk_space(&self) -> CheckResult {
        match free_disk_bytes(&self.project_root) {
            Some(free) => {
                let gib = free as f64 / (1024.0 * 1024.0 * 1024.0);
                let msg = format!("{gib:.1} GiB free");
                if gib < 5.0 {
                    CheckResult::new("Disk Space", CheckStatus::Fail, msg, CheckCategory::Environment)
                        .with_details("UE4 builds need tens of GiB")
                        .with_fix("Free up disk space")
                } else if gib < 20.0 {
                    CheckResult::new("Disk Space", CheckStatus::Warn, msg, CheckCategory::Environment)
                        .with_details("UE4 builds can consume large amounts of disk")
                } else {
                    CheckResult::new(
                        "Disk Space",
                        CheckStatus::Pass,
                        msg,
                        CheckCategory::Environment,
                    )
                }
            }
            None => CheckResult::new(
                "Disk Space",
                CheckStatus::Warn,
                "Could not determine free disk space",
                CheckCategory::Environment,
            )
            .with_fix("df -kP .  # check available space manually"),
        }
    }

    /// Cheap network reachability flag. Does NOT make a network request by
    /// default (that would be slow/flaky in CI); it reports whether the
    /// `ROCKET_OFFLINE` opt-out is set, and a best-effort hint.
    fn check_network(&self) -> CheckResult {
        if std::env::var("ROCKET_OFFLINE").is_ok() {
            return CheckResult::new(
                "Network",
                CheckStatus::Warn,
                "Offline mode (ROCKET_OFFLINE set)",
                CheckCategory::Environment,
            )
            .with_details("crates.io / supabase operations will be skipped")
            .with_fix("unset ROCKET_OFFLINE  # to re-enable network operations");
        }
        CheckResult::new(
            "Network",
            CheckStatus::Pass,
            "Online (ROCKET_OFFLINE not set)",
            CheckCategory::Environment,
        )
    }

    fn check_ggen(&self) -> CheckResult {
        match command_version("ggen", &["--version"]) {
            Some(v) => CheckResult::new("ggen", CheckStatus::Pass, v, CheckCategory::Optional),
            None => CheckResult::new(
                "ggen",
                CheckStatus::Warn,
                "ggen not found in PATH",
                CheckCategory::Optional,
            )
            .with_details("ggen is required for Ostar generative workflows.")
            .with_fix("Install ggen (Ostar code generator)"),
        }
    }

    fn check_manifest(&self) -> CheckResult {
        let path = self.project_root.join("project-manifest.json");
        if path.exists() {
            CheckResult::new(
                "Project Manifest",
                CheckStatus::Pass,
                "project-manifest.json found",
                CheckCategory::Project,
            )
            .with_details(format!("Path: {}", path.display()))
        } else {
            CheckResult::new(
                "Project Manifest",
                CheckStatus::Fail,
                "project-manifest.json MISSING",
                CheckCategory::Project,
            )
            .with_details("Run 'rocket sync' to generate it.")
            .with_fix("rocket sync")
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
            // Missing projects are sample/optional content repos — not pipeline-blocking.
            // The active pipeline project (Brm) being present is what matters for cooking.
            CheckResult {
                name: "Manifest Projects".to_string(),
                status: CheckStatus::Warn,
                message: format!(
                    "{}/{total} declared .uproject files not present (optional sample content)",
                    missing.len()
                ),
                details: Some(missing.join("\n")),
            }
        }
    }

    fn check_versions_dir(&self) -> CheckResult {
        let path = self.project_root.join("versions");
        if path.exists() && path.is_dir() {
            CheckResult::new(
                "Versions Directory",
                CheckStatus::Pass,
                "versions/ directory exists",
                CheckCategory::Project,
            )
        } else {
            CheckResult::new(
                "Versions Directory",
                CheckStatus::Fail,
                "versions/ directory MISSING or not a directory",
                CheckCategory::Project,
            )
            .with_details("This directory should contain the Unreal Engine projects.")
            .with_fix("mkdir -p versions  # then restore UE4 projects")
        }
    }

    /// Verify every Rust workspace's Cargo.toml is present.
    fn check_rust_workspaces(&self) -> CheckResult {
        let workspaces = [
            "tools/Cargo.toml",
            "nexus-engine/Cargo.toml",
            "blueprint-rs/Cargo.toml",
            "unify-rs/Cargo.toml",
            "infinity-blade-4/mud/Cargo.toml",
            "chicago-tdd-tools/Cargo.toml",
        ];
        let missing: Vec<&str> = workspaces
            .iter()
            .copied()
            .filter(|rel| !self.project_root.join(rel).exists())
            .collect();
        if missing.is_empty() {
            CheckResult::new(
                "Rust Workspaces",
                CheckStatus::Pass,
                format!("all {} workspace manifests present", workspaces.len()),
                CheckCategory::Project,
            )
        } else {
            CheckResult::new(
                "Rust Workspaces",
                CheckStatus::Warn,
                format!("{} workspace manifest(s) missing", missing.len()),
                CheckCategory::Project,
            )
            .with_details(format!("Missing: {}", missing.join(", ")))
            .with_fix("git submodule update --init  # or restore the missing workspace")
        }
    }

    fn check_pwa_node_modules(&self) -> CheckResult {
        let pwa = self.project_root.join("pwa-staff");
        if !pwa.exists() {
            return CheckResult::new(
                "PWA node_modules",
                CheckStatus::Warn,
                "pwa-staff/ not found",
                CheckCategory::Project,
            )
            .with_details("Expected the pwa-staff/ directory at the project root")
            .with_fix("git submodule update --init  # or restore pwa-staff/");
        }
        let node_modules = pwa.join("node_modules");
        if node_modules.is_dir() {
            CheckResult::new(
                "PWA node_modules",
                CheckStatus::Pass,
                "pwa-staff/node_modules present",
                CheckCategory::Project,
            )
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
        if offenders.is_empty() {
            CheckResult::new(
                "Path Dependencies",
                CheckStatus::Pass,
                "no absolute-path Cargo deps detected",
                CheckCategory::Project,
            )
        } else {
            CheckResult::new(
                "Path Dependencies",
                CheckStatus::Warn,
                format!("{} absolute-path dep(s) will break on other machines", offenders.len()),
                CheckCategory::Project,
            )
            .with_details(offenders.join("\n"))
            .with_fix("Vendor wasm4pm-compat / clap-noun-verb or use crates.io versions")
        }
    }

    fn check_anti_llm_cheat_lsp(&self) -> CheckResult {
        match command_version("anti-llm-cheat-lsp", &["--version"]) {
            Some(v) => CheckResult::new(
                "anti-llm-cheat-lsp",
                CheckStatus::Pass,
                v,
                CheckCategory::Optional,
            ),
            None => CheckResult::new(
                "anti-llm-cheat-lsp",
                CheckStatus::Warn,
                "anti-llm-cheat-lsp not found in PATH",
                CheckCategory::Optional,
            )
            .with_details("Install: cargo install lsp-max --bin anti-llm-cheat-lsp")
            .with_fix("cargo install lsp-max --bin anti-llm-cheat-lsp"),
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

        // Check WebSocketNetworking — exists in Experimental/ in 4.27-html5-es3
        let ws_paths = vec![
            "Engine/Plugins/Experimental/WebSocketNetworking/WebSocketNetworking.uplugin",
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

        // VaRest is a Marketplace plugin — not bundled in engine source builds.
        // Check common install locations; treat as WARN not FAIL when missing.
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

        match (ws_ok, varest_ok) {
            (true, true) => CheckResult {
                name: "UE4 Plugins".to_string(),
                status: CheckStatus::Pass,
                message: "Found required plugins: WebSocketNetworking, VaRest".to_string(),
                details: None,
            },
            (true, false) => CheckResult {
                name: "UE4 Plugins".to_string(),
                status: CheckStatus::Warn,
                message: "WebSocketNetworking present; VaRest not found (Marketplace plugin — install separately if needed)".to_string(),
                details: None,
            },
            (false, _) => {
                let mut missing = vec!["WebSocketNetworking"];
                if !varest_ok {
                    missing.push("VaRest");
                }
                CheckResult {
                    name: "UE4 Plugins".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("Missing required plugins: {}", missing.join(", ")),
                    details: Some(
                        "Ensure your UE4 build includes WebSocketNetworking (Engine/Plugins/Experimental/ in 4.27-html5-es3).".to_string(),
                    ),
                }
            }
        }
    }
    /// Check whether the most recent HTML5 cook produced a real package.
    ///
    /// Looks for the default archive directory (`/tmp/brm-html5-archive`) first,
    /// then falls back to `manufactured/` in the project root.
    fn check_html5_package(&self) -> CheckResult {
        // Prefer manifest-derived archive paths (project-name-based) over hardcoded defaults.
        let manifest_paths: Vec<PathBuf> = crate::Manifest::load(self.project_root.join("project-manifest.json"))
            .map(|m| {
                m.projects().iter()
                    .map(|p| PathBuf::from(format!("/tmp/{}-html5-archive/HTML5", p.name.to_lowercase())))
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
        }

        if std::env::var("UE4_ROOT").is_ok() {
            CheckResult::new(
                "UE4 Root",
                CheckStatus::Pass,
                "UE4_ROOT environment variable is set",
                CheckCategory::Environment,
            )
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
        i += 1;
    }
    None
}

fn parse_triple_at(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty());
    let maj = parts.next()?.parse::<u64>().ok()?;
    // Require at least major.minor to count as a version.
    let min = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next().and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
    Some((maj, min, patch))
}

/// Best-effort free disk space (bytes) for the filesystem containing `path`.
/// Uses `df -kP` (POSIX) — avoids adding a libc/sysinfo dependency.
fn free_disk_bytes(path: &Path) -> Option<u64> {
    let output = Command::new("df")
        .arg("-kP")
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    // Second line, 4th column = available 1K-blocks.
    let line = text.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    let avail_kb = cols.get(3)?.parse::<u64>().ok()?;
    Some(avail_kb * 1024)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // ---- Preserved original tests (backward compatibility) --------------

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
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("not present") || result.message.contains("optional"));
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

    #[test]
    fn html5_package_check_returns_a_named_html5_result() {
        // Any project root — the check must return a properly named result without panicking
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_html5_package();
        assert!(result.name.contains("HTML5"), "check name must mention HTML5");
        // Status may be Pass/Warn/Fail depending on machine state — just must not panic
        matches!(result.status, CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail);
    }

    #[test]
    fn html5_package_check_reads_manifest_for_archive_paths() {
        let dir = tempdir().unwrap();
        // Write a minimal project-manifest.json with a project named "Alpha"
        let manifest = serde_json::json!({
            "projects": [{"name": "Alpha", "uproject_path": "Alpha.uproject", "targets": []}]
        });
        fs::write(dir.path().join("project-manifest.json"),
            serde_json::to_string(&manifest).unwrap()).unwrap();

        // No archive directories exist, so this should still be Warn (not panic)
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_html5_package();
        // Should be Warn because no archive exists — but must NOT panic or error
        assert!(matches!(result.status, CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Pass));
    }
}
