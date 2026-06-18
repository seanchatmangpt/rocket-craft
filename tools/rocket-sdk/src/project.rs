use anyhow::Result;
use crate::setup;
use crate::manifest;
use crate::config;
use crate::crypto;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::Context;
use knhk::{Validator, AndroidKeystoreLaw};

/// Setup the Unreal Engine environment
pub fn cmd_setup() -> anyhow::Result<()> {
    setup::run_setup()?;
    Ok(())
}

/// Synchronize project manifest with filesystem
pub fn cmd_sync() -> anyhow::Result<()> {
    crate::run_sync()?;
    Ok(())
}

/// Build a project target
pub fn cmd_build(
    project: Option<String>,
    target: Option<String>,
    platform: Option<String>,
) -> anyhow::Result<()> {
    crate::run_build(project, target, platform)?;
    Ok(())
}

/// Audit project health and semantic law compliance
pub fn cmd_audit() -> anyhow::Result<()> {
    crate::run_audit()?;
    Ok(())
}

/// Clean build artifacts (Binaries, Intermediate, Saved)
pub fn cmd_clean() -> anyhow::Result<()> {
    crate::run_clean()?;
    Ok(())
}

/// Show project information
pub fn cmd_info() -> anyhow::Result<()> {
    println!("{}", "Rocket Craft Generative Orchestration Tool".bold().cyan());
    println!("Version: 0.1.0");
    println!("Stack: Ostar / ggen / Rust / UE4.24");
    Ok(())
}

/// Tail Unreal Engine build logs
pub fn cmd_logs(
    file: Option<String>,
    lines: usize,
) -> anyhow::Result<()> {
    // TODO(anti-cheat): `file` and `lines` were silently discarded — the old body printed
    // "Tailing logs... (Not fully implemented)" and returned Ok(()) with no real log tailing.
    // Real implementation: open the UE4 log file (Saved/Logs/<project>.log), seek to EOF-N
    // lines, then stream new content. Delegate to `rocket-cmd/src/main.rs::run_logs()`.
    let _ = (file, lines);
    Err(anyhow::anyhow!(
        "cmd_logs not yet implemented; use `rocket logs` (rocket-cmd binary) directly"
    ))
}
