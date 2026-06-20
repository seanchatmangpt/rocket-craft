//! Lint runner across all Rust workspaces and the TypeScript PWA.
//!
//! `run_lint(config)` iterates every known workspace, invokes `cargo clippy`
//! (or `--fix` variants), and for `pwa-staff/` runs `npm run lint` /
//! `npx eslint --fix`.  Results are collected into a `LintReport` that can be
//! rendered as coloured terminal output or serialised as JSON.

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Output};
use thiserror::Error;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum LintError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("workspace root does not exist: {0}")]
    MissingRoot(PathBuf),

    #[error("lint runner failed to launch in {workspace}: {reason}")]
    SpawnFailed {
        workspace: String,
        reason: String,
    },
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for a lint run.
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// Repository root (where `tools/`, `nexus-engine/`, `pwa-staff/` live).
    pub root: PathBuf,
    /// When `true` run auto-fix commands instead of check-only commands.
    pub fix: bool,
    /// Restrict the run to a single workspace name (e.g. `"nexus-engine"`).
    /// `None` means run all workspaces.
    pub workspace: Option<String>,
    /// Emit a JSON-serialisable summary instead of coloured prose.
    pub json: bool,
}

// ── Per-workspace result ──────────────────────────────────────────────────────

/// Language family of a linted workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceKind {
    Rust,
    TypeScript,
}

/// Outcome for a single workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub name: String,
    pub kind: WorkspaceKind,
    pub warnings: usize,
    pub errors: usize,
    /// Combined stdout + stderr captured from the linter.
    pub output: String,
    /// `true` when the linter exited with status 0.
    pub pass: bool,
}

// ── Aggregate report ──────────────────────────────────────────────────────────

/// Aggregated report returned by `run_lint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    pub results: Vec<LintResult>,
    pub total_warnings: usize,
    pub total_errors: usize,
    pub all_pass: bool,
}

impl LintReport {
    fn from_results(results: Vec<LintResult>) -> Self {
        let total_warnings = results.iter().map(|r| r.warnings).sum();
        let total_errors = results.iter().map(|r| r.errors).sum();
        let all_pass = results.iter().all(|r| r.pass);
        Self {
            results,
            total_warnings,
            total_errors,
            all_pass,
        }
    }

    /// Print the coloured, section-separated report to stdout.
    pub fn print(&self) {
        let rust_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.kind == WorkspaceKind::Rust)
            .collect();
        let ts_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.kind == WorkspaceKind::TypeScript)
            .collect();

        if !rust_results.is_empty() {
            println!("{}", format!("{:─<44}", "── Rust ").bold());
            for r in &rust_results {
                print_result(r);
            }
        }

        if !ts_results.is_empty() {
            println!("{}", format!("{:─<44}", "── TypeScript ").bold());
            for r in &ts_results {
                print_result(r);
            }
        }

        println!();
        let summary = format!(
            "Total: {} errors, {} warnings — {}",
            self.total_errors,
            self.total_warnings,
            if self.all_pass { "PASS" } else { "FAIL" }
        );
        if self.all_pass {
            println!("{}", summary.green().bold());
        } else {
            println!("{}", summary.red().bold());
        }
    }
}

fn print_result(r: &LintResult) {
    let icon = if r.pass { "✓".green() } else { "✗".red() };
    let name = format!("{:<18}", r.name);
    let detail = if r.errors > 0 {
        format!("{} errors, {} warnings", r.errors, r.warnings)
            .red()
            .to_string()
    } else if r.warnings > 0 {
        format!("0 errors, {} warnings", r.warnings)
            .yellow()
            .to_string()
    } else {
        "0 errors".green().to_string()
    };
    println!("  {icon}  {name} {detail}");
}

// ── Workspace catalogue ───────────────────────────────────────────────────────

struct RustWorkspace {
    /// Display name used in the report.
    name: &'static str,
    /// Path relative to the repository root.
    rel_path: &'static str,
}

const RUST_WORKSPACES: &[RustWorkspace] = &[
    RustWorkspace { name: "tools", rel_path: "tools" },
    RustWorkspace { name: "nexus-engine", rel_path: "nexus-engine" },
    RustWorkspace { name: "blueprint-rs", rel_path: "blueprint-rs" },
    RustWorkspace { name: "unify-rs", rel_path: "unify-rs" },
    RustWorkspace { name: "infinity-blade-4/mud", rel_path: "infinity-blade-4/mud" },
    RustWorkspace { name: "chicago-tdd-tools", rel_path: "chicago-tdd-tools" },
];

const PWA_NAME: &str = "pwa-staff";
const PWA_REL_PATH: &str = "pwa-staff";

// ── Runner ────────────────────────────────────────────────────────────────────

/// Run lint across all (or a single) workspace(s).
pub fn run_lint(config: LintConfig) -> Result<LintReport, LintError> {
    if !config.root.exists() {
        return Err(LintError::MissingRoot(config.root.clone()));
    }

    let mut results: Vec<LintResult> = Vec::new();

    // --- Rust workspaces ---
    for ws in RUST_WORKSPACES {
        if let Some(ref filter) = config.workspace {
            if filter != ws.name {
                continue;
            }
        }

        let ws_path = config.root.join(ws.rel_path);
        if !ws_path.exists() {
            // Skip workspaces that aren't checked out on this machine.
            continue;
        }

        let result = lint_rust_workspace(ws.name, &ws_path, config.fix)?;
        results.push(result);
    }

    // --- PWA workspace ---
    let pwa_included = config
        .workspace
        .as_deref()
        .map(|f| f == PWA_NAME)
        .unwrap_or(true);

    if pwa_included {
        let pwa_path = config.root.join(PWA_REL_PATH);
        if pwa_path.exists() {
            let result = lint_pwa(&pwa_path, config.fix)?;
            results.push(result);
        }
    }

    Ok(LintReport::from_results(results))
}

// ── Rust lint helper ──────────────────────────────────────────────────────────

fn lint_rust_workspace(name: &str, path: &PathBuf, fix: bool) -> Result<LintResult, LintError> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path);

    if fix {
        // clippy --fix then rustfmt
        cmd.args([
            "clippy",
            "--workspace",
            "--all-features",
            "--fix",
            "--allow-dirty",
            "--allow-staged",
            "--",
            "-D",
            "warnings",
        ]);
    } else {
        cmd.args([
            "clippy",
            "--workspace",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ]);
    }

    let output = cmd.output().map_err(|e| LintError::SpawnFailed {
        workspace: name.to_string(),
        reason: e.to_string(),
    })?;

    let result = parse_rust_output(name, output);

    // When fix=true also run rustfmt (best effort; don't fail on rustfmt errors)
    if fix {
        let _ = Command::new("cargo")
            .current_dir(path)
            .args(["fmt", "--all"])
            .output();
    }

    Ok(result)
}

fn parse_rust_output(name: &str, output: Output) -> LintResult {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Count lines prefixed with "warning[" or "warning: " (rustc/clippy format).
    let warnings = combined
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("warning[") || t.starts_with("warning: ")
        })
        .count();
    let errors = combined
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("error[") || t.starts_with("error: ")
        })
        .count();

    let pass = output.status.success();

    LintResult {
        name: name.to_string(),
        kind: WorkspaceKind::Rust,
        warnings,
        errors,
        output: combined,
        pass,
    }
}

// ── PWA lint helper ───────────────────────────────────────────────────────────

fn lint_pwa(path: &PathBuf, fix: bool) -> Result<LintResult, LintError> {
    let output = if fix {
        Command::new("npx")
            .current_dir(path)
            .args(["eslint", "--fix", "."])
            .output()
            .map_err(|e| LintError::SpawnFailed {
                workspace: PWA_NAME.to_string(),
                reason: e.to_string(),
            })?
    } else {
        Command::new("npm")
            .current_dir(path)
            .args(["run", "lint"])
            .output()
            .map_err(|e| LintError::SpawnFailed {
                workspace: PWA_NAME.to_string(),
                reason: e.to_string(),
            })?
    };

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // ESLint: count lines containing " error " or " warning " in its default formatter
    let errors = combined
        .lines()
        .filter(|l| l.contains(" error "))
        .count();
    let warnings = combined
        .lines()
        .filter(|l| l.contains(" warning "))
        .count();

    let pass = output.status.success();

    Ok(LintResult {
        name: PWA_NAME.to_string(),
        kind: WorkspaceKind::TypeScript,
        warnings,
        errors,
        output: combined,
        pass,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_report_aggregates_correctly() {
        let results = vec![
            LintResult {
                name: "a".into(),
                kind: WorkspaceKind::Rust,
                warnings: 2,
                errors: 0,
                output: String::new(),
                pass: true,
            },
            LintResult {
                name: "b".into(),
                kind: WorkspaceKind::Rust,
                warnings: 0,
                errors: 3,
                output: String::new(),
                pass: false,
            },
            LintResult {
                name: "pwa-staff".into(),
                kind: WorkspaceKind::TypeScript,
                warnings: 0,
                errors: 0,
                output: String::new(),
                pass: true,
            },
        ];

        let report = LintReport::from_results(results);
        assert_eq!(report.total_warnings, 2);
        assert_eq!(report.total_errors, 3);
        assert!(!report.all_pass);
    }

    #[test]
    fn lint_report_all_pass_when_no_errors() {
        let results = vec![
            LintResult {
                name: "a".into(),
                kind: WorkspaceKind::Rust,
                warnings: 1,
                errors: 0,
                output: String::new(),
                pass: true,
            },
            LintResult {
                name: "pwa-staff".into(),
                kind: WorkspaceKind::TypeScript,
                warnings: 0,
                errors: 0,
                output: String::new(),
                pass: true,
            },
        ];

        let report = LintReport::from_results(results);
        assert!(report.all_pass);
        assert_eq!(report.total_errors, 0);
    }

    #[test]
    fn missing_root_returns_error() {
        let config = LintConfig {
            root: PathBuf::from("/nonexistent/path/that/cannot/exist"),
            fix: false,
            workspace: None,
            json: false,
        };
        assert!(matches!(run_lint(config), Err(LintError::MissingRoot(_))));
    }

    #[test]
    fn parse_rust_output_counts_warnings_and_errors() {
        use std::os::unix::process::ExitStatusExt;
        use std::process::ExitStatus;

        // Simulate clippy output with two warnings and one error
        let fake_stdout = b"warning: unused variable `x`\nwarning[C0001]: clippy thing\nerror[E0001]: something wrong\n";
        let output = Output {
            status: ExitStatus::from_raw(1),
            stdout: fake_stdout.to_vec(),
            stderr: vec![],
        };

        let result = parse_rust_output("test-ws", output);
        assert_eq!(result.warnings, 2);
        assert_eq!(result.errors, 1);
        assert!(!result.pass);
    }

    #[test]
    fn workspace_filter_skips_others() {
        // When filtering to a name that doesn't exist in any real workspace path
        // and the root is a temp dir (no workspace dirs present), results are empty.
        let dir = tempfile::tempdir().unwrap();
        let config = LintConfig {
            root: dir.path().to_path_buf(),
            fix: false,
            workspace: Some("nexus-engine".into()),
            json: false,
        };
        let report = run_lint(config).unwrap();
        // The workspace dir doesn't exist under the temp dir so it is skipped.
        assert_eq!(report.results.len(), 0);
        assert!(report.all_pass); // vacuously true
    }
}
