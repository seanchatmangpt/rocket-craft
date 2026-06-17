use anyhow::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
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

#[derive(Serialize)]
pub struct EmptyResponse {}

/// Setup the Unreal Engine environment
#[verb("setup")]
pub fn cmd_setup() -> clap_noun_verb::Result<EmptyResponse> {
    setup::run_setup().map_err(|e| clap_noun_verb::Error::from(e.to_string()))?;
    Ok(EmptyResponse {})
}

/// Synchronize project manifest with filesystem
#[verb("sync")]
pub fn cmd_sync() -> clap_noun_verb::Result<EmptyResponse> {
    crate::run_sync().map_err(|e| clap_noun_verb::Error::from(e.to_string()))?;
    Ok(EmptyResponse {})
}

/// Build a project target
#[verb("build")]
pub fn cmd_build(
    /// Project name
    #[arg(short, long)]
    project: Option<String>,
    /// Target name (e.g., ShooterGame, Brm)
    #[arg(short, long)]
    target: Option<String>,
    /// Platform (e.g., Win64, Android, HTML5)
    #[arg(short, long)]
    platform: Option<String>,
) -> clap_noun_verb::Result<EmptyResponse> {
    crate::run_build(project, target, platform).map_err(|e| clap_noun_verb::Error::from(e.to_string()))?;
    Ok(EmptyResponse {})
}

/// Audit project health and semantic law compliance
#[verb("audit")]
pub fn cmd_audit() -> clap_noun_verb::Result<EmptyResponse> {
    crate::run_audit().map_err(|e| clap_noun_verb::Error::from(e.to_string()))?;
    Ok(EmptyResponse {})
}

/// Clean build artifacts (Binaries, Intermediate, Saved)
#[verb("clean")]
pub fn cmd_clean() -> clap_noun_verb::Result<EmptyResponse> {
    crate::run_clean().map_err(|e| clap_noun_verb::Error::from(e.to_string()))?;
    Ok(EmptyResponse {})
}

/// Show project information
#[verb("info")]
pub fn cmd_info() -> clap_noun_verb::Result<EmptyResponse> {
    println!("{}", "Rocket Craft Generative Orchestration Tool".bold().cyan());
    println!("Version: 0.1.0");
    println!("Stack: Ostar / ggen / Rust / UE4.24");
    Ok(EmptyResponse {})
}

/// Tail Unreal Engine build logs
#[verb("logs")]
pub fn cmd_logs(
    /// Specific log file to tail
    file: Option<String>,
    /// Number of initial lines to show [default: 50]
    #[arg(short, long, default_value = "50")]
    lines: usize,
) -> clap_noun_verb::Result<EmptyResponse> {
    // TODO(anti-cheat): Both `file` and `lines` arguments were accepted but silently
    // discarded — only "Tailing logs... (Not fully implemented)" was printed, giving
    // the false impression of a working log tailer.
    // The real log-tailing implementation lives in `rocket-cmd/src/main.rs::run_logs()`.
    // This clap_noun_verb wrapper must delegate to that function once the
    // `clap-noun-verb` path dependency is properly vendored.
    // Until then, invoke `rocket logs` (the rocket-cmd binary) directly.
    let _ = (file, lines); // suppress unused-variable warnings
    Err(clap_noun_verb::Error::from(
        "cmd_logs is not yet implemented in the clap_noun_verb wrapper; \
         use `rocket logs` (rocket-cmd binary) directly".to_string(),
    ))
}
