//! Rich project health dashboard for the rocket-craft monorepo.
//!
//! Run all checks in parallel with `tokio::task::spawn_blocking` and collect
//! results into a `StatusReport`.  The caller decides how to render it
//! (coloured text, JSON, quiet mode).

use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Health {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub name: String,
    pub health: Health,
    pub detail: String,
    /// Wall-clock seconds taken by this check (None for instant checks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_secs: Option<f64>,
}

impl CheckItem {
    fn pass(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), health: Health::Pass, detail: detail.into(), elapsed_secs: None }
    }
    fn warn(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), health: Health::Warn, detail: detail.into(), elapsed_secs: None }
    }
    fn fail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), health: Health::Fail, detail: detail.into(), elapsed_secs: None }
    }
    fn with_elapsed(mut self, secs: f64) -> Self {
        self.elapsed_secs = Some(secs);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionResult {
    pub title: String,
    pub items: Vec<CheckItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusReport {
    pub timestamp: DateTime<Utc>,
    pub sections: Vec<SectionResult>,
}

impl StatusReport {
    pub fn total_checks(&self) -> usize {
        self.sections.iter().map(|s| s.items.len()).sum()
    }
    pub fn passing(&self) -> usize {
        self.sections.iter().flat_map(|s| &s.items).filter(|i| i.health == Health::Pass).count()
    }
    pub fn failing(&self) -> usize {
        self.sections.iter().flat_map(|s| &s.items).filter(|i| i.health == Health::Fail).count()
    }
    pub fn warning(&self) -> usize {
        self.sections.iter().flat_map(|s| &s.items).filter(|i| i.health == Health::Warn).count()
    }
}

// ── Check helpers ─────────────────────────────────────────────────────────────

fn run_cmd_version(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn cargo_check_workspace(dir: &Path) -> CheckItem {
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    if !dir.exists() {
        return CheckItem::warn(name, "directory not found — skipped");
    }

    let start = Instant::now();
    let result = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .current_dir(dir)
        .output();
    let elapsed = start.elapsed().as_secs_f64();

    match result {
        Ok(o) if o.status.success() => {
            CheckItem::pass(&name, "cargo check").with_elapsed(elapsed)
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let first_line = stderr.lines().next().unwrap_or("compile error").to_string();
            CheckItem::fail(&name, first_line).with_elapsed(elapsed)
        }
        Err(e) => CheckItem::fail(name, format!("could not run cargo: {e}")),
    }
}

// ── Section: Git ──────────────────────────────────────────────────────────────

fn check_git_section(root: &Path) -> SectionResult {
    let mut items = Vec::new();

    // Branch
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    items.push(CheckItem::pass("Branch", &branch));

    // Dirty / clean
    let porcelain = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output();
    match porcelain {
        Ok(o) if o.status.success() => {
            let out = String::from_utf8_lossy(&o.stdout);
            let count = out.lines().count();
            if count == 0 {
                items.push(CheckItem::pass("Status", "clean"));
            } else {
                items.push(CheckItem::warn("Status", format!("{count} uncommitted changes")));
            }
        }
        _ => items.push(CheckItem::warn("Status", "could not determine git status")),
    }

    // Ahead of upstream
    let ahead = Command::new("git")
        .args(["rev-list", "@{u}..HEAD", "--count"])
        .current_dir(root)
        .output();
    match ahead {
        Ok(o) if o.status.success() => {
            let n: u64 = String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            if n == 0 {
                items.push(CheckItem::pass("Ahead", "up to date with origin"));
            } else {
                items.push(CheckItem::warn("Ahead", format!("{n} commits ahead of origin")));
            }
        }
        _ => {
            // No upstream configured — not an error
            items.push(CheckItem::pass("Ahead", "no upstream configured"));
        }
    }

    SectionResult { title: "Git".into(), items }
}

// ── Section: Environment ──────────────────────────────────────────────────────

fn check_env_section() -> SectionResult {
    let mut items = Vec::new();

    // Rust
    match run_cmd_version("rustc", &["--version"]) {
        Some(v) => items.push(CheckItem::pass("Rust", v)),
        None => items.push(CheckItem::fail("Rust", "rustc not found in PATH")),
    }

    // Node
    match run_cmd_version("node", &["--version"]) {
        Some(v) => items.push(CheckItem::pass("Node", v)),
        None => items.push(CheckItem::warn("Node", "node not found — required for pwa-staff")),
    }

    // UE4_ROOT
    let ue4 = std::env::var("UE4_ROOT").ok();
    match ue4 {
        Some(ref p) if Path::new(p).exists() => {
            items.push(CheckItem::pass("UE4_ROOT", p.as_str()))
        }
        Some(ref p) => items.push(CheckItem::fail("UE4_ROOT", format!("set but path missing: {p}"))),
        None => items.push(CheckItem::warn("UE4_ROOT", "not set")),
    }

    // BLENDER_PATH
    let blender = std::env::var("BLENDER_PATH")
        .ok()
        .or_else(|| run_cmd_version("blender", &["--version"]).map(|_| "blender".to_string()));
    match blender {
        Some(p) => items.push(CheckItem::pass("Blender", p)),
        None => items.push(CheckItem::warn("Blender", "not found — required for asset-pipeline")),
    }

    // Docker
    let docker_ok = Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if docker_ok {
        items.push(CheckItem::pass("Docker", "running"));
    } else {
        items.push(CheckItem::warn("Docker", "not running or not installed"));
    }

    SectionResult { title: "Environment".into(), items }
}

// ── Section: Rust Workspaces ──────────────────────────────────────────────────

fn check_rust_workspaces_section(root: &Path) -> SectionResult {
    // These are all the Rust workspaces in the monorepo (asset-pipeline excluded — it
    // lives in a worktree and may not always be present).
    let workspace_dirs: &[&str] = &[
        "tools",
        "nexus-engine",
        "blueprint-rs",
        "unify-rs",
        "infinity-blade-4/mud",
        "chicago-tdd-tools",
    ];

    // Run checks sequentially inside a sync context (callers use spawn_blocking).
    let items: Vec<CheckItem> = workspace_dirs
        .iter()
        .map(|rel| cargo_check_workspace(&root.join(rel)))
        .collect();

    SectionResult { title: "Rust Workspaces".into(), items }
}

// ── Section: PWA ─────────────────────────────────────────────────────────────

fn check_pwa_section(root: &Path) -> SectionResult {
    let pwa_dir = root.join("pwa-staff");
    let mut items = Vec::new();

    // node_modules
    if pwa_dir.join("node_modules").exists() {
        items.push(CheckItem::pass("node_modules", "installed"));
    } else {
        items.push(CheckItem::warn("node_modules", "not installed — run: cd pwa-staff && npm ci"));
    }

    // TypeScript
    if pwa_dir.exists() {
        let start = Instant::now();
        let result = Command::new("npx")
            .args(["tsc", "--noEmit"])
            .current_dir(&pwa_dir)
            .output();
        let elapsed = start.elapsed().as_secs_f64();
        match result {
            Ok(o) if o.status.success() => {
                items.push(CheckItem::pass("TypeScript", "tsc --noEmit").with_elapsed(elapsed));
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let errors: Vec<&str> = stderr.lines().take(3).collect();
                items.push(
                    CheckItem::fail("TypeScript", errors.join("; ")).with_elapsed(elapsed),
                );
            }
            Err(e) => items.push(CheckItem::warn("TypeScript", format!("npx not available: {e}"))),
        }

        // ESLint
        let start = Instant::now();
        let lint_result = Command::new("npx")
            .args(["eslint", "src/", "--max-warnings=0"])
            .current_dir(&pwa_dir)
            .output();
        let elapsed_lint = start.elapsed().as_secs_f64();
        match lint_result {
            Ok(o) if o.status.success() => {
                items.push(CheckItem::pass("lint", "no issues").with_elapsed(elapsed_lint));
            }
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let warnings = stdout
                    .lines()
                    .filter(|l| l.contains("warning") || l.contains("error"))
                    .count();
                let msg = if warnings > 0 {
                    format!("{warnings} warnings/errors")
                } else {
                    "lint failed".into()
                };
                items.push(CheckItem::warn("lint", msg).with_elapsed(elapsed_lint));
            }
            Err(_) => {
                items.push(CheckItem::warn("lint", "eslint not available"));
            }
        }
    } else {
        items.push(CheckItem::fail("TypeScript", "pwa-staff directory not found"));
        items.push(CheckItem::fail("lint", "pwa-staff directory not found"));
    }

    SectionResult { title: "PWA".into(), items }
}

// ── Section: UE4 Projects ─────────────────────────────────────────────────────

fn check_ue4_projects_section(root: &Path) -> SectionResult {
    let manifest_path = root.join("project-manifest.json");
    let mut items = Vec::new();

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => {
            items.push(CheckItem::fail("project-manifest.json", "not found"));
            return SectionResult { title: "UE4 Projects".into(), items };
        }
    };

    let manifest: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            items.push(CheckItem::fail("project-manifest.json", format!("invalid JSON: {e}")));
            return SectionResult { title: "UE4 Projects".into(), items };
        }
    };

    let projects = match manifest.get("projects").and_then(|p| p.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            items.push(CheckItem::warn("projects", "no 'projects' array in manifest"));
            return SectionResult { title: "UE4 Projects".into(), items };
        }
    };

    for proj in &projects {
        let name = proj
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("?")
            .to_string();
        let uproject = proj
            .get("uproject_path")
            .and_then(|p| p.as_str())
            .unwrap_or("");

        let abs = if Path::new(uproject).is_absolute() {
            PathBuf::from(uproject)
        } else {
            root.join(uproject)
        };

        if abs.exists() {
            items.push(CheckItem::pass(&name, uproject));
        } else {
            items.push(CheckItem::warn(&name, format!("{uproject} — not found on disk")));
        }
    }

    SectionResult { title: "UE4 Projects".into(), items }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run all checks in parallel using Tokio tasks and return a `StatusReport`.
pub async fn run_status(root: PathBuf) -> Result<StatusReport> {
    let r0 = root.clone();
    let r1 = root.clone();
    let r2 = root.clone();
    let r3 = root.clone();

    // Spawn all sections concurrently; git + env + UE4 are instant, workspaces/PWA may be slow.
    let git_handle = tokio::task::spawn_blocking(move || check_git_section(&r0));
    let env_handle = tokio::task::spawn_blocking(check_env_section);
    let ws_handle = tokio::task::spawn_blocking(move || check_rust_workspaces_section(&r1));
    let pwa_handle = tokio::task::spawn_blocking(move || check_pwa_section(&r2));
    let ue4_handle = tokio::task::spawn_blocking(move || check_ue4_projects_section(&r3));

    let (git_sec, env_sec, ws_sec, pwa_sec, ue4_sec) = tokio::join!(
        git_handle,
        env_handle,
        ws_handle,
        pwa_handle,
        ue4_handle,
    );

    Ok(StatusReport {
        timestamp: Utc::now(),
        sections: vec![
            git_sec?,
            env_sec?,
            ws_sec?,
            pwa_sec?,
            ue4_sec?,
        ],
    })
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render a `StatusReport` as coloured text on stdout.
///
/// `quiet` — only print failing / warning items.
pub fn render_text(report: &StatusReport, quiet: bool) {
    let today = report.timestamp.format("%Y-%m-%d");
    let header = format!("  rocket-craft  status  {today}  ");
    let width = header.len() + 2;
    let bar: String = "═".repeat(width);

    println!("{}", format!("╔{bar}╗").bold());
    println!("{}", format!("║{header}║").bold());
    println!("{}", format!("╚{bar}╝").bold());

    for section in &report.sections {
        println!();
        let sec_line = format!("── {} ", section.title);
        let dashes = if 46 > sec_line.len() { 46 - sec_line.len() } else { 2 };
        println!("{}{}", sec_line.bold(), "─".repeat(dashes));

        for item in &section.items {
            if quiet && item.health == Health::Pass {
                continue;
            }
            let icon = match item.health {
                Health::Pass => "✓".green().to_string(),
                Health::Warn => "!".yellow().to_string(),
                Health::Fail => "✗".red().to_string(),
            };
            let name_col = format!("{:<20}", item.name);
            let timing = item.elapsed_secs.map(|s| format!(" ({:.1}s)", s)).unwrap_or_default();
            let detail = match item.health {
                Health::Pass => item.detail.dimmed().to_string(),
                Health::Warn => item.detail.yellow().to_string(),
                Health::Fail => item.detail.red().to_string(),
            };
            println!("  {icon}  {}{}{}", name_col, detail, timing);
        }
    }

    // Summary bar
    let total = report.total_checks();
    let pass = report.passing();
    let fail = report.failing();
    let warn = report.warning();

    println!();
    let sec_line = "── Summary ";
    let dashes = if 46 > sec_line.len() { 46 - sec_line.len() } else { 2 };
    println!("{}{}", sec_line.bold(), "─".repeat(dashes));

    let pct = if total > 0 { pass * 100 / total } else { 0 };
    let filled = pct * 15 / 100;
    let bar_str = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(15_usize.saturating_sub(filled))
    );

    let summary_line = format!("{pass}/{total} checks passing   [{bar_str}]  {pct}%");
    let colored = if fail > 0 {
        summary_line.red().to_string()
    } else if warn > 0 {
        summary_line.yellow().to_string()
    } else {
        summary_line.green().to_string()
    };
    println!("  {colored}");
    if fail > 0 || warn > 0 {
        println!("  {} failed  {} warned", fail.to_string().red(), warn.to_string().yellow());
    }
    println!();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_temp_git_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    #[test]
    fn check_item_pass_has_correct_health() {
        let item = CheckItem::pass("Rust", "1.82.0");
        assert_eq!(item.health, Health::Pass);
        assert_eq!(item.name, "Rust");
        assert_eq!(item.detail, "1.82.0");
        assert!(item.elapsed_secs.is_none());
    }

    #[test]
    fn check_item_fail_with_elapsed() {
        let item = CheckItem::fail("tools", "compile error").with_elapsed(1.23);
        assert_eq!(item.health, Health::Fail);
        assert!((item.elapsed_secs.unwrap() - 1.23).abs() < 0.01);
    }

    #[test]
    fn status_report_counts_correctly() {
        let report = StatusReport {
            timestamp: Utc::now(),
            sections: vec![SectionResult {
                title: "Test".into(),
                items: vec![
                    CheckItem::pass("a", "ok"),
                    CheckItem::fail("b", "bad"),
                    CheckItem::warn("c", "meh"),
                    CheckItem::pass("d", "ok"),
                ],
            }],
        };
        assert_eq!(report.total_checks(), 4);
        assert_eq!(report.passing(), 2);
        assert_eq!(report.failing(), 1);
        assert_eq!(report.warning(), 1);
    }

    #[test]
    fn check_git_section_returns_three_items() {
        let repo = make_temp_git_repo();
        let section = check_git_section(repo.path());
        assert_eq!(section.title, "Git");
        // Branch, Status, Ahead
        assert_eq!(section.items.len(), 3);
    }

    #[test]
    fn check_env_section_returns_rust_item() {
        let section = check_env_section();
        assert_eq!(section.title, "Environment");
        assert!(section.items.iter().any(|i| i.name == "Rust"));
    }

    #[test]
    fn check_ue4_projects_missing_manifest_returns_fail() {
        let dir = TempDir::new().unwrap();
        let section = check_ue4_projects_section(dir.path());
        assert_eq!(section.title, "UE4 Projects");
        assert!(section.items.iter().any(|i| i.health == Health::Fail));
    }

    #[test]
    fn check_ue4_projects_reads_manifest() {
        let dir = TempDir::new().unwrap();
        let uproject = dir.path().join("Game.uproject");
        fs::write(&uproject, "{}").unwrap();
        let manifest = serde_json::json!({
            "projects": [{
                "name": "Game",
                "uproject_path": uproject.to_str().unwrap(),
                "targets": []
            }]
        });
        fs::write(dir.path().join("project-manifest.json"), manifest.to_string()).unwrap();

        let section = check_ue4_projects_section(dir.path());
        assert_eq!(section.items.len(), 1);
        assert_eq!(section.items[0].health, Health::Pass);
        assert_eq!(section.items[0].name, "Game");
    }

    #[test]
    fn check_ue4_projects_warns_on_missing_uproject() {
        let dir = TempDir::new().unwrap();
        let manifest = serde_json::json!({
            "projects": [{
                "name": "Ghost",
                "uproject_path": "/nonexistent/Ghost.uproject",
                "targets": []
            }]
        });
        fs::write(dir.path().join("project-manifest.json"), manifest.to_string()).unwrap();

        let section = check_ue4_projects_section(dir.path());
        assert_eq!(section.items[0].health, Health::Warn);
    }

    #[test]
    fn render_text_does_not_panic() {
        let report = StatusReport {
            timestamp: Utc::now(),
            sections: vec![SectionResult {
                title: "Test".into(),
                items: vec![
                    CheckItem::pass("a", "ok"),
                    CheckItem::fail("b", "bad"),
                    CheckItem::warn("c", "meh"),
                ],
            }],
        };
        // Should not panic in any mode
        render_text(&report, false);
        render_text(&report, true);
    }

    #[test]
    fn cargo_check_missing_dir_returns_warn() {
        let item = cargo_check_workspace(Path::new("/nonexistent/workspace/path"));
        assert_eq!(item.health, Health::Warn);
        assert!(item.detail.contains("not found"));
    }

    #[tokio::test]
    async fn run_status_returns_five_sections() {
        let dir = TempDir::new().unwrap();
        // Write a minimal project-manifest.json so the UE4 section doesn't error
        fs::write(
            dir.path().join("project-manifest.json"),
            r#"{"projects":[]}"#,
        )
        .unwrap();
        let report = run_status(dir.path().to_path_buf()).await.unwrap();
        assert_eq!(report.sections.len(), 5);
        assert_eq!(report.sections[0].title, "Git");
        assert_eq!(report.sections[1].title, "Environment");
        assert_eq!(report.sections[2].title, "Rust Workspaces");
        assert_eq!(report.sections[3].title, "PWA");
        assert_eq!(report.sections[4].title, "UE4 Projects");
    }
}
