//! `rocket doctor` — environment & project health diagnostics.
//!
//! This module is the brains behind `rocket doctor`. It runs a battery of
//! independent checks (toolchain versions, project layout, git state, known
//! repo gotchas, disk space, …), scores the overall health 0-100, suggests a
//! concrete `fix_command` for every non-passing check, and can optionally
//! execute the *safe* subset of those fixes.
//!
//! Design notes:
//! * **Single file, std-only.** No new crates are introduced; concurrency uses
//!   `std::thread::scope`, version parsing is hand-rolled. This keeps the crate
//!   buildable in isolation (the wider `tools/` workspace has absolute-path deps
//!   that break a full build).
//! * **Backward compatible.** `RocketDoctor::new(PathBuf)` and
//!   `run_diagnostics() -> DiagnosticReport` keep their old signatures and the
//!   original nine checks still exist with their original messages, so the
//!   pre-existing tests pass unchanged. New data is purely additive
//!   (`CheckResult::fix_command`, `CheckResult::category`, etc.).

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// ANSI helpers (no external color crate dependency).
// ---------------------------------------------------------------------------

mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
}

// ---------------------------------------------------------------------------
// Core enums
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
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

    /// Run every check. Independent checks execute concurrently via
    /// `std::thread::scope`; the result order is deterministic (sorted by the
    /// fixed `check_order` index) so output and tests are stable.
    pub fn run_diagnostics(&self) -> DiagnosticReport {
        // Each closure is `Send` because it only borrows `&self` and returns an
        // owned `CheckResult`. We box them so they share a Vec type.
        type Check<'a> = (usize, Box<dyn Fn() -> CheckResult + Send + Sync + 'a>);

        let checks: Vec<Check> = vec![
            (0, Box::new(|| self.check_git())),
            (1, Box::new(|| self.check_git_status())),
            (2, Box::new(|| self.check_rust())),
            (3, Box::new(|| self.check_cargo())),
            (4, Box::new(|| self.check_clippy())),
            (5, Box::new(|| self.check_rustfmt())),
            (6, Box::new(|| self.check_python())),
            (7, Box::new(|| self.check_node())),
            (8, Box::new(|| self.check_blender())),
            (9, Box::new(|| self.check_disk_space())),
            (10, Box::new(|| self.check_network())),
            (11, Box::new(|| self.check_manifest())),
            (12, Box::new(|| self.check_versions_dir())),
            (13, Box::new(|| self.check_rust_workspaces())),
            (14, Box::new(|| self.check_pwa_node_modules())),
            (15, Box::new(|| self.check_path_dep_gotchas())),
            (16, Box::new(|| self.check_ue4_root())),
            (17, Box::new(|| self.check_ggen())),
            (18, Box::new(|| self.check_anti_llm_cheat_lsp())),
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
            CheckResult::new(
                "PWA node_modules",
                CheckStatus::Warn,
                "pwa-staff/node_modules missing",
                CheckCategory::Project,
            )
            .with_details("Dependencies not installed")
            .with_fix("cd pwa-staff && npm ci")
        }
    }

    /// Flag the known absolute-path dependency gotchas in tools/ Cargo.toml
    /// files (wasm4pm-compat / clap-noun-verb pointing at /Users/sac/...).
    fn check_path_dep_gotchas(&self) -> CheckResult {
        let suspects = [
            "tools/knhk/Cargo.toml",
            "tools/rocket-cmd/Cargo.toml",
        ];
        let mut offenders = Vec::new();
        for rel in suspects {
            let p = self.project_root.join(rel);
            if let Ok(content) = std::fs::read_to_string(&p) {
                for line in content.lines() {
                    let l = line.trim();
                    if l.starts_with('#') {
                        continue;
                    }
                    // Absolute path deps are the breakage: /Users/... or /home/... or C:\
                    let is_abs_path_dep = l.contains("path =")
                        && (l.contains("path = \"/")
                            || l.contains("path = \"C:")
                            || l.contains("/Users/sac"));
                    if is_abs_path_dep {
                        offenders.push(format!("{rel}: {l}"));
                    }
                }
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
        let rocket_json = self.project_root.join(".rocket.json");
        if rocket_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&rocket_json) {
                if content.contains("ue4_root") {
                    return CheckResult::new(
                        "UE4 Root",
                        CheckStatus::Pass,
                        "UE4 root configured in .rocket.json",
                        CheckCategory::Environment,
                    );
                }
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
            CheckResult::new(
                "UE4 Root",
                CheckStatus::Warn,
                "UE4 root not configured",
                CheckCategory::Environment,
            )
            .with_details("Run 'rocket setup' to configure Unreal Engine path.")
            .with_fix("rocket setup  # or export UE4_ROOT=/path/to/UE4")
        }
    }
}

// ---------------------------------------------------------------------------
// Free helper functions
// ---------------------------------------------------------------------------

/// Run `cmd args...` and return trimmed stdout (falling back to stderr, which
/// some tools like `node`/`java` use for version output) if it succeeds.
fn command_version(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        Some(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        }
    }
}

/// Extract the first `MAJOR.MINOR[.PATCH]` triple from a version string.
/// e.g. "rustc 1.82.0 (...)" -> (1, 82, 0); "v20.11.1" -> (20, 11, 1).
pub fn parse_semver_from(s: &str) -> Option<(u64, u64, u64)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            // Try to parse a version starting here.
            if let Some(v) = parse_triple_at(&s[i..]) {
                return Some(v);
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

    #[test]
    fn test_check_git_status_with_repo() {
        let dir = tempdir().unwrap();
        let _repo = git2::Repository::init(dir.path()).unwrap();

        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_git_status();

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

    // ---- New tests -------------------------------------------------------

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver_from("rustc 1.82.0 (f6e511eec 2024)"), Some((1, 82, 0)));
        assert_eq!(parse_semver_from("v20.11.1"), Some((20, 11, 1)));
        assert_eq!(parse_semver_from("Python 3.11.4"), Some((3, 11, 4)));
        assert_eq!(parse_semver_from("cargo 1.79"), Some((1, 79, 0)));
        assert_eq!(parse_semver_from("no version here"), None);
        // Leading non-version digits followed by a real version still parse a triple.
        assert!(parse_semver_from("git version 2.39.2").is_some());
    }

    #[test]
    fn test_run_diagnostics_is_deterministic_order() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let a = doctor.run_diagnostics();
        let b = doctor.run_diagnostics();
        let names_a: Vec<_> = a.checks.iter().map(|c| c.name.clone()).collect();
        let names_b: Vec<_> = b.checks.iter().map(|c| c.name.clone()).collect();
        assert_eq!(names_a, names_b);
        assert!(names_a.contains(&"Git".to_string()));
        assert!(names_a.contains(&"Disk Space".to_string()));
        assert!(names_a.contains(&"Path Dependencies".to_string()));
    }

    #[test]
    fn test_health_score_all_pass_is_100() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![
                CheckResult::new("a", CheckStatus::Pass, "ok", CheckCategory::Environment),
                CheckResult::new("b", CheckStatus::Pass, "ok", CheckCategory::Toolchain),
            ],
        };
        assert_eq!(report.health_score(), 100);
        assert_eq!(report.overall_status(), CheckStatus::Pass);
    }

    #[test]
    fn test_health_score_mixed() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![
                CheckResult::new("a", CheckStatus::Pass, "ok", CheckCategory::Environment),
                CheckResult::new("b", CheckStatus::Fail, "no", CheckCategory::Toolchain),
            ],
        };
        // 1 pass (1.0) + 1 fail (0.0) over weight 2.0 = 50.
        assert_eq!(report.health_score(), 50);
        assert_eq!(report.overall_status(), CheckStatus::Fail);
    }

    #[test]
    fn test_optional_fail_does_not_force_overall_fail() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![
                CheckResult::new("a", CheckStatus::Pass, "ok", CheckCategory::Environment),
                CheckResult::new("opt", CheckStatus::Fail, "no", CheckCategory::Optional),
            ],
        };
        assert_eq!(report.overall_status(), CheckStatus::Warn);
    }

    #[test]
    fn test_overall_warn() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![CheckResult::new(
                "a",
                CheckStatus::Warn,
                "meh",
                CheckCategory::Environment,
            )],
        };
        assert_eq!(report.overall_status(), CheckStatus::Warn);
    }

    #[test]
    fn test_json_output_is_valid() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let report = doctor.run_diagnostics();
        let json = report.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("health_score").is_some());
        assert!(parsed.get("checks").unwrap().is_array());
        assert!(parsed.get("overall_status").is_some());
    }

    #[test]
    fn test_pretty_and_compact_render() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let report = doctor.run_diagnostics();
        let pretty = report.pretty(false);
        assert!(pretty.contains("Rocket Doctor"));
        assert!(pretty.contains("Toolchain"));
        let compact = report.compact_summary(false);
        assert!(compact.contains("doctor:"));
        assert!(compact.contains("health"));
        // Color variant should embed an escape sequence.
        assert!(report.compact_summary(true).contains("\x1b["));
    }

    #[test]
    fn test_non_passing_checks_have_fix() {
        let dir = tempdir().unwrap();
        // Empty dir => manifest missing, versions missing, etc. all non-pass.
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let report = doctor.run_diagnostics();
        for c in &report.checks {
            if c.status != CheckStatus::Pass {
                assert!(
                    c.fix_command.is_some(),
                    "non-passing check '{}' is missing a fix_command",
                    c.name
                );
            }
        }
    }

    #[test]
    fn test_run_fixes_dry_run_is_non_destructive() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let fixes = doctor.run_fixes(true);
        // Dry run should never Apply anything.
        for f in &fixes {
            assert_ne!(f.outcome, FixOutcome::Applied);
        }
        // versions/ must NOT have been created in dry-run.
        assert!(!dir.path().join("versions").exists());
    }

    #[test]
    fn test_run_fixes_creates_versions_dir_when_not_dry() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let _ = doctor.run_fixes(false);
        // The Versions Directory safe-fix should have created the dir.
        assert!(dir.path().join("versions").is_dir());
    }

    #[test]
    fn test_path_dep_gotcha_detection() {
        let dir = tempdir().unwrap();
        let knhk = dir.path().join("tools/knhk");
        fs::create_dir_all(&knhk).unwrap();
        fs::write(
            knhk.join("Cargo.toml"),
            "[dependencies]\nwasm4pm-compat = { path = \"/Users/sac/wasm4pm-compat\" }\n",
        )
        .unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let r = doctor.check_path_dep_gotchas();
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.fix_command.is_some());
        assert!(r.details.unwrap().contains("wasm4pm-compat"));
    }

    #[test]
    fn test_path_dep_gotcha_clean() {
        let dir = tempdir().unwrap();
        let knhk = dir.path().join("tools/knhk");
        fs::create_dir_all(&knhk).unwrap();
        fs::write(
            knhk.join("Cargo.toml"),
            "[dependencies]\nserde = \"1\"\nunrdf = { path = \"../unrdf\" }\n",
        )
        .unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let r = doctor.check_path_dep_gotchas();
        // Relative path deps are fine; should pass.
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn test_pwa_node_modules() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        // No pwa-staff dir at all.
        assert_eq!(doctor.check_pwa_node_modules().status, CheckStatus::Warn);

        let pwa = dir.path().join("pwa-staff");
        fs::create_dir_all(&pwa).unwrap();
        assert_eq!(doctor.check_pwa_node_modules().status, CheckStatus::Warn);

        fs::create_dir_all(pwa.join("node_modules")).unwrap();
        assert_eq!(doctor.check_pwa_node_modules().status, CheckStatus::Pass);
    }

    #[test]
    fn test_rust_workspaces_missing() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let r = doctor.check_rust_workspaces();
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.fix_command.is_some());
    }

    #[test]
    fn test_counts() {
        let report = DiagnosticReport {
            timestamp: Utc::now(),
            checks: vec![
                CheckResult::new("a", CheckStatus::Pass, "", CheckCategory::Environment),
                CheckResult::new("b", CheckStatus::Warn, "", CheckCategory::Environment),
                CheckResult::new("c", CheckStatus::Fail, "", CheckCategory::Environment),
                CheckResult::new("d", CheckStatus::Pass, "", CheckCategory::Environment),
            ],
        };
        assert_eq!(report.counts(), (2, 1, 1));
    }
}
