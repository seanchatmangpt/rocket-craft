//! Dependency management utilities — outdated checks, security audits, and
//! dependency tree display across all Rust workspaces and the TypeScript PWA.

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum DepsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

// ── Data types — outdated ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedPackage {
    pub name: String,
    pub current: String,
    pub latest: String,
    /// "patch", "minor", "major", or "unknown"
    pub kind: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorkspaceOutdated {
    pub name: String,
    pub packages: Vec<OutdatedPackage>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DepsReport {
    pub rust: Vec<WorkspaceOutdated>,
    pub npm: Vec<WorkspaceOutdated>,
}

// ── Data types — audit ────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VulnCount {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub unknown: u32,
}

impl VulnCount {
    pub fn total(&self) -> u32 {
        self.critical + self.high + self.medium + self.low + self.unknown
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceAudit {
    pub name: String,
    pub vulns: VulnCount,
    pub ecosystem: String, // "rust" or "npm"
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuditReport {
    pub workspaces: Vec<WorkspaceAudit>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// All Rust workspaces within the monorepo (relative to repo root).
fn rust_workspaces(root: &Path) -> Vec<(&'static str, PathBuf)> {
    let dirs = [
        "tools",
        "nexus-engine",
        "blueprint-rs",
        "unify-rs",
        "infinity-blade-4/mud",
        "chicago-tdd-tools",
    ];
    dirs.iter()
        .map(|d| (*d, root.join(d)))
        .filter(|(_, p)| p.join("Cargo.toml").exists())
        .collect()
}

fn bump_kind(current: &str, latest: &str) -> String {
    let parse = |v: &str| -> Option<(u64, u64, u64)> {
        let v = v.trim_start_matches(|c: char| !c.is_numeric());
        let parts: Vec<&str> = v.splitn(3, '.').collect();
        if parts.len() < 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].split(|c: char| !c.is_numeric()).next()?.parse().ok()?,
        ))
    };
    if let (Some((cmaj, cmin, cpat)), Some((lmaj, lmin, lpat))) =
        (parse(current), parse(latest))
    {
        if lmaj > cmaj {
            "major".into()
        } else if lmin > cmin {
            "minor".into()
        } else if lpat > cpat {
            "patch".into()
        } else {
            "unknown".into()
        }
    } else {
        "unknown".into()
    }
}

// ── cargo-outdated JSON shape (subset we care about) ─────────────────────────

#[derive(Deserialize)]
struct OutdatedRoot {
    dependencies: Vec<OutdatedDep>,
}

#[derive(Deserialize)]
struct OutdatedDep {
    name: String,
    project: String,
    latest: String,
    #[serde(rename = "latest_that_matches")]
    _latest_that_matches: Option<String>,
    #[serde(rename = "compat")]
    _compat: Option<String>,
}

// ── Cargo.lock fallback parser ────────────────────────────────────────────────

/// Very small lock-file analyser: finds packages whose version is pinned with
/// `=` in *any* workspace member `Cargo.toml`, then reports current vs the
/// same value (since we can't look up the registry, latest == current here).
fn pinned_from_lock(workspace_dir: &Path) -> Vec<OutdatedPackage> {
    let lock_path = workspace_dir.join("Cargo.lock");
    let lock_text = match std::fs::read_to_string(&lock_path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };

    // Collect all Cargo.toml files under the workspace
    let mut pinned_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    let walker = walkdir::WalkDir::new(workspace_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml");

    for entry in walker {
        if let Ok(text) = std::fs::read_to_string(entry.path()) {
            for line in text.lines() {
                // Look for version = "=1.2.3" style pins
                if line.contains("= \"=") {
                    if let Some(name) = extract_dep_name(line) {
                        pinned_names.insert(name);
                    }
                }
            }
        }
    }

    if pinned_names.is_empty() {
        return vec![];
    }

    // Parse lock file for current versions of those pinned packages
    let mut result = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;

    for line in lock_text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if let (Some(n), Some(v)) = (current_name.take(), current_version.take()) {
                if pinned_names.contains(&n) {
                    result.push(OutdatedPackage {
                        name: n.clone(),
                        current: v.clone(),
                        latest: v.clone(), // can't know without registry
                        kind: "pinned (=)".into(),
                    });
                }
            }
        } else if let Some(rest) = line.strip_prefix("name = \"") {
            current_name = Some(rest.trim_end_matches('"').to_owned());
        } else if let Some(rest) = line.strip_prefix("version = \"") {
            current_version = Some(rest.trim_end_matches('"').to_owned());
        }
    }
    // Handle last block
    if let (Some(n), Some(v)) = (current_name, current_version) {
        if pinned_names.contains(&n) {
            result.push(OutdatedPackage {
                name: n,
                current: v.clone(),
                latest: v,
                kind: "pinned (=)".into(),
            });
        }
    }

    result
}

fn extract_dep_name(line: &str) -> Option<String> {
    // Handles:  serde = { version = "=1.0.0", ... }
    //           serde = "=1.0.0"
    // The name is the key before '='
    let name = line.split('=').next()?.trim().trim_matches('"').to_owned();
    if name.is_empty() || name.starts_with('[') || name.starts_with('#') {
        None
    } else {
        Some(name)
    }
}

// ── Public API — check_outdated ───────────────────────────────────────────────

/// Check for outdated dependencies across Rust workspaces and the PWA.
///
/// Uses `cargo outdated --root-deps-only --format json` when available;
/// falls back to scanning for `=`-pinned versions in `Cargo.toml` files.
/// `npm outdated --json` is run non-fatally in `pwa-staff/`.
///
/// If `workspace` is `Some`, only that workspace is checked (Rust side).
pub fn check_outdated(root: &Path, workspace: Option<&str>) -> Result<DepsReport, DepsError> {
    let mut report = DepsReport::default();

    // ── Rust workspaces ──────────────────────────────────────────────────────
    let workspaces = rust_workspaces(root);
    let to_check: Vec<_> = if let Some(ws) = workspace {
        workspaces
            .into_iter()
            .filter(|(name, _)| *name == ws)
            .collect()
    } else {
        workspaces
    };

    // Probe for cargo-outdated once
    let has_cargo_outdated = Command::new("cargo")
        .args(["outdated", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_cargo_outdated {
        eprintln!(
            "{} cargo-outdated not found — falling back to lock-file analysis.\n  \
             Install with: cargo install cargo-outdated",
            "hint:".yellow().bold()
        );
    }

    for (name, dir) in &to_check {
        let mut ws_report = WorkspaceOutdated {
            name: name.to_string(),
            packages: vec![],
        };

        if has_cargo_outdated {
            let output = Command::new("cargo")
                .args(["outdated", "--root-deps-only", "--format", "json"])
                .current_dir(dir)
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    if let Ok(parsed) =
                        serde_json::from_slice::<OutdatedRoot>(&out.stdout)
                    {
                        for dep in parsed.dependencies {
                            let kind = bump_kind(&dep.project, &dep.latest);
                            ws_report.packages.push(OutdatedPackage {
                                name: dep.name,
                                current: dep.project,
                                latest: dep.latest,
                                kind,
                            });
                        }
                    }
                }
                Ok(out) => {
                    // cargo-outdated exits non-zero when nothing is outdated on some versions
                    if let Ok(parsed) =
                        serde_json::from_slice::<OutdatedRoot>(&out.stdout)
                    {
                        for dep in parsed.dependencies {
                            let kind = bump_kind(&dep.project, &dep.latest);
                            ws_report.packages.push(OutdatedPackage {
                                name: dep.name,
                                current: dep.project,
                                latest: dep.latest,
                                kind,
                            });
                        }
                    }
                }
                Err(_) => {
                    ws_report.packages = pinned_from_lock(dir);
                }
            }
        } else {
            ws_report.packages = pinned_from_lock(dir);
        }

        report.rust.push(ws_report);
    }

    // ── npm (pwa-staff) ──────────────────────────────────────────────────────
    let pwa_dir = root.join("pwa-staff");
    if pwa_dir.join("package.json").exists() {
        let has_npm = Command::new("npm")
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_npm {
            eprintln!(
                "{} npm not found — skipping PWA outdated check.",
                "hint:".yellow().bold()
            );
        } else {
            let out = Command::new("npm")
                .args(["outdated", "--json"])
                .current_dir(&pwa_dir)
                .output();

            if let Ok(out) = out {
                // npm outdated --json exits non-zero when there are outdated packages;
                // that's normal — parse stdout regardless.
                if let Ok(map) =
                    serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(
                        &out.stdout,
                    )
                {
                    let mut ws = WorkspaceOutdated {
                        name: "pwa-staff".into(),
                        packages: vec![],
                    };
                    for (pkg_name, info) in map {
                        let current = info
                            .get("current")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                            .to_owned();
                        let latest = info
                            .get("latest")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                            .to_owned();
                        let kind = bump_kind(&current, &latest);
                        ws.packages.push(OutdatedPackage {
                            name: pkg_name,
                            current,
                            latest,
                            kind,
                        });
                    }
                    report.npm.push(ws);
                }
            }
        }
    }

    Ok(report)
}

// ── cargo audit JSON shape (subset) ─────────────────────────────────────────

#[derive(Deserialize)]
struct CargoAuditOutput {
    vulnerabilities: CargoAuditVulns,
}

#[derive(Deserialize)]
struct CargoAuditVulns {
    list: Vec<CargoAuditVuln>,
}

#[derive(Deserialize)]
struct CargoAuditVuln {
    advisory: CargoAuditAdvisory,
}

#[derive(Deserialize)]
struct CargoAuditAdvisory {
    #[serde(default)]
    severity: Option<String>,
}

// ── Public API — audit_security ──────────────────────────────────────────────

/// Run `cargo audit` and `npm audit` across all workspaces.
///
/// `cargo-audit` must be installed (`cargo install cargo-audit`); missing
/// installation is non-fatal (a hint is printed and the workspace is skipped).
/// `npm audit` in `pwa-staff/` is always non-fatal.
pub fn audit_security(root: &Path) -> Result<AuditReport, DepsError> {
    let mut report = AuditReport::default();

    let has_cargo_audit = Command::new("cargo")
        .args(["audit", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_cargo_audit {
        eprintln!(
            "{} cargo-audit not found — skipping Rust audits.\n  \
             Install with: cargo install cargo-audit",
            "hint:".yellow().bold()
        );
    }

    for (name, dir) in rust_workspaces(root) {
        if !has_cargo_audit {
            report.workspaces.push(WorkspaceAudit {
                name: name.to_owned(),
                vulns: VulnCount::default(),
                ecosystem: "rust".into(),
            });
            continue;
        }

        let out = Command::new("cargo")
            .args(["audit", "--json"])
            .current_dir(&dir)
            .output();

        let vulns = match out {
            Err(_) => VulnCount::default(),
            Ok(o) => {
                // cargo-audit exits non-zero if vulns found; parse stdout anyway
                if let Ok(parsed) =
                    serde_json::from_slice::<CargoAuditOutput>(&o.stdout)
                {
                    let mut counts = VulnCount::default();
                    for v in parsed.vulnerabilities.list {
                        match v
                            .advisory
                            .severity
                            .as_deref()
                            .unwrap_or("unknown")
                            .to_lowercase()
                            .as_str()
                        {
                            "critical" => counts.critical += 1,
                            "high" => counts.high += 1,
                            "medium" => counts.medium += 1,
                            "low" => counts.low += 1,
                            _ => counts.unknown += 1,
                        }
                    }
                    counts
                } else {
                    VulnCount::default()
                }
            }
        };

        report.workspaces.push(WorkspaceAudit {
            name: name.to_owned(),
            vulns,
            ecosystem: "rust".into(),
        });
    }

    // ── npm audit ────────────────────────────────────────────────────────────
    let pwa_dir = root.join("pwa-staff");
    if pwa_dir.join("package.json").exists() {
        let out = Command::new("npm")
            .args(["audit", "--json"])
            .current_dir(&pwa_dir)
            .output();

        let vulns = if let Ok(o) = out {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&o.stdout) {
                // npm audit --json: { "metadata": { "vulnerabilities": { "critical": n, ... } } }
                let meta = json
                    .get("metadata")
                    .and_then(|m| m.get("vulnerabilities"));
                VulnCount {
                    critical: meta
                        .and_then(|v| v.get("critical"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    high: meta
                        .and_then(|v| v.get("high"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    medium: meta
                        .and_then(|v| v.get("moderate"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    low: meta
                        .and_then(|v| v.get("low"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    unknown: 0,
                }
            } else {
                VulnCount::default()
            }
        } else {
            VulnCount::default()
        };

        report.workspaces.push(WorkspaceAudit {
            name: "pwa-staff".into(),
            vulns,
            ecosystem: "npm".into(),
        });
    }

    Ok(report)
}

// ── Public API — show_tree ────────────────────────────────────────────────────

/// Print `cargo tree` for the requested workspace(s).
///
/// If `workspace` is `None`, iterates all known Rust workspaces.
/// `depth` controls the `--depth` flag (default: 3).
pub fn show_tree(
    root: &Path,
    workspace: Option<&str>,
    depth: Option<u32>,
) -> Result<(), DepsError> {
    let workspaces = rust_workspaces(root);
    let to_show: Vec<_> = if let Some(ws) = workspace {
        workspaces
            .into_iter()
            .filter(|(name, _)| *name == ws)
            .collect()
    } else {
        // Default to "tools" when no workspace specified
        workspaces
            .into_iter()
            .filter(|(name, _)| *name == "tools")
            .collect()
    };

    let depth_str = depth.unwrap_or(3).to_string();

    for (name, dir) in &to_show {
        println!("{}", format!("── {} ", name).cyan().bold());
        let status = Command::new("cargo")
            .args(["tree", "--depth", &depth_str])
            .current_dir(dir)
            .status()
            .map_err(DepsError::Io)?;

        if !status.success() {
            eprintln!(
                "{} cargo tree failed for workspace '{}'",
                "warn:".yellow().bold(),
                name
            );
        }
    }

    Ok(())
}

// ── Text rendering ────────────────────────────────────────────────────────────

/// Print a human-readable `deps check` report to stdout.
pub fn render_check_report(report: &DepsReport) {
    println!("{}", "── Rust ────────────────────────────────────────".cyan());
    if report.rust.is_empty() {
        println!("  (no workspaces found)");
    }
    for ws in &report.rust {
        if ws.packages.is_empty() {
            println!("  {} {}", ws.name.bold(), "(up to date)".green());
        } else {
            println!("  {}", ws.name.bold());
            for pkg in &ws.packages {
                println!(
                    "    {:<30} {:<12} →  {:<12} ({})",
                    pkg.name.yellow(),
                    pkg.current,
                    pkg.latest.green(),
                    pkg.kind
                );
            }
        }
    }

    println!("{}", "── npm ─────────────────────────────────────────".cyan());
    if report.npm.is_empty() {
        println!("  (no npm workspaces found or all up to date)");
    }
    for ws in &report.npm {
        if ws.packages.is_empty() {
            println!("  {} {}", ws.name.bold(), "(up to date)".green());
        } else {
            println!("  {}", ws.name.bold());
            for pkg in &ws.packages {
                println!(
                    "    {:<40} {:<12} →  {}",
                    pkg.name.yellow(),
                    pkg.current,
                    pkg.latest.green()
                );
            }
        }
    }
}

/// Print a human-readable `deps audit` report to stdout.
pub fn render_audit_report(report: &AuditReport) {
    println!("{}", "── Security Audit ──────────────────────────────".cyan());
    if report.workspaces.is_empty() {
        println!("  (no workspaces audited)");
        return;
    }
    for ws in &report.workspaces {
        let total = ws.vulns.total();
        let status = if total == 0 {
            "✓ clean".green().to_string()
        } else {
            format!(
                "{} critical, {} high, {} medium, {} low",
                ws.vulns.critical, ws.vulns.high, ws.vulns.medium, ws.vulns.low
            )
            .red()
            .to_string()
        };
        println!("  {:<30} [{}]  {}", ws.name.bold(), ws.ecosystem.dimmed(), status);
    }
}
