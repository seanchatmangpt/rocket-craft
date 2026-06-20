//! Workspace management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_lock() -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    crate::lock::run_lock(&root)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_clean() -> Result<Value> {
    use std::fs;
    use walkdir::WalkDir;
    tracing::info!("{}", "=== Cleaning Workspace ===");
    let targets = ["Binaries", "Intermediate", "Saved"];
    for entry in WalkDir::new("versions")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_dir() && targets.contains(&e.file_name().to_string_lossy().as_ref())
        })
    {
        fs::remove_dir_all(entry.path())
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    }
    tracing::info!("{}", "Cleanup complete.");
    Ok(serde_json::json!({"status": "ok"}))
}

/// Recursively map dependencies of all local workspaces and enforce deterministic lock
#[verb("lock", "workspace")]
fn lock_workspace() -> Result<Value> {
    do_lock()
}

/// Clean build artifacts (Binaries, Intermediate, Saved)
#[verb("clean", "workspace")]
fn clean_workspace() -> Result<Value> {
    do_clean()
}
