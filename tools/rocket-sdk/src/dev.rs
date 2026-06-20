//! `rocket dev` — combined developer-experience entry point.
//!
//! Prints a condensed status summary (git branch, failing checks count) then
//! enters watch mode so every `.rs`, `.toml`, or `.ts` change triggers an
//! incremental rebuild of the affected workspace.  The goal is a single
//! command that gets a developer productive immediately after cloning.
//!
//! # Lifecycle
//! 1. Print a one-line header with the current git branch and timestamp.
//! 2. Run `status::run_status` (parallel checks) and render a compact summary.
//!    Warnings are shown; passing checks are suppressed unless `--verbose`.
//! 3. Call `watch::run` with an optional workspace filter — loops until Ctrl-C.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::{
    status::{run_status, render_text},
    watch::{run as run_watch, WatchConfig},
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Configuration for `rocket dev`.
pub struct DevConfig {
    /// Monorepo root (directory containing `project-manifest.json`).
    pub root: std::path::PathBuf,
    /// If `Some`, restrict the watcher to a single named workspace.
    pub only_workspace: Option<String>,
    /// When `true`, print all status check results (not just failures/warnings).
    pub verbose: bool,
    /// Skip the initial status snapshot and go straight to watch mode.
    pub skip_status: bool,
}

impl DevConfig {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            root: root.into(),
            only_workspace: None,
            verbose: false,
            skip_status: false,
        }
    }

    pub fn only(mut self, name: impl Into<String>) -> Self {
        self.only_workspace = Some(name.into());
        self
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    pub fn skip_status(mut self) -> Self {
        self.skip_status = true;
        self
    }
}

/// Print the one-line dev-mode header.
fn print_header(root: &std::path::Path) {
    // Best-effort git branch; fall back gracefully if git is unavailable.
    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    let now = chrono::Local::now().format("%H:%M:%S").to_string();

    println!(
        "\n{} {} {} {}",
        "rocket dev".bold().cyan(),
        "on".dimmed(),
        branch.bold(),
        format!("({now})").dimmed()
    );
    println!("{}", "─".repeat(48).dimmed());
}

/// Run a Tokio runtime, execute the async status check, and print a compact
/// one-line summary.  Returns `true` when there are no failing checks.
fn print_status_summary(root: &std::path::Path, verbose: bool) -> bool {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build();

    let rt = match rt {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} could not build tokio runtime: {e}", "!".yellow());
            return false;
        }
    };

    let report = match rt.block_on(run_status(root.to_path_buf())) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} status check failed: {e}", "✗".red());
            return false;
        }
    };

    // quiet=true suppresses passing checks; verbose inverts that.
    render_text(&report, !verbose);

    report.failing() == 0
}

/// Entry point: run status snapshot then enter the watch loop.
pub fn run(cfg: DevConfig) -> Result<()> {
    let root = cfg
        .root
        .canonicalize()
        .context("resolving monorepo root")?;

    print_header(&root);

    if !cfg.skip_status {
        println!("{}", "Checking workspace health…".dimmed());
        print_status_summary(&root, cfg.verbose);
        println!();
    }

    // ── Enter watch mode ────────────────────────────────────────────────────
    let mut watch_cfg = WatchConfig::new(root);
    if let Some(ws) = cfg.only_workspace {
        watch_cfg = watch_cfg.only(ws);
    }

    run_watch(watch_cfg)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn dev_config_new_sets_root() {
        let cfg = DevConfig::new("/tmp/test-repo");
        assert_eq!(cfg.root, Path::new("/tmp/test-repo"));
        assert!(cfg.only_workspace.is_none());
        assert!(!cfg.verbose);
        assert!(!cfg.skip_status);
    }

    #[test]
    fn dev_config_builder_only() {
        let cfg = DevConfig::new("/tmp").only("tools");
        assert_eq!(cfg.only_workspace, Some("tools".to_string()));
    }

    #[test]
    fn dev_config_builder_verbose() {
        let cfg = DevConfig::new("/tmp").verbose();
        assert!(cfg.verbose);
    }

    #[test]
    fn dev_config_builder_skip_status() {
        let cfg = DevConfig::new("/tmp").skip_status();
        assert!(cfg.skip_status);
    }

    #[test]
    fn dev_config_builder_chain() {
        let cfg = DevConfig::new("/tmp")
            .only("nexus-engine")
            .verbose()
            .skip_status();
        assert_eq!(cfg.only_workspace, Some("nexus-engine".to_string()));
        assert!(cfg.verbose);
        assert!(cfg.skip_status);
    }

    #[test]
    fn print_header_does_not_panic_on_non_git_dir() {
        // /tmp is not a git repo; print_header should degrade gracefully.
        print_header(Path::new("/tmp"));
    }
}
