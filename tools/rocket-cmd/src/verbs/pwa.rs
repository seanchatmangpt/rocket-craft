//! PWA management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_pwa_lint(dir: String) -> Result<Value> {
    use std::path::PathBuf;
    use std::process::Command;
    let pwa_dir = PathBuf::from(&dir);
    if !pwa_dir.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!("PWA directory not found: {}", dir)));
    }
    tracing::info!("{}", "=== Linting & Formatting PWA Assets ===");
    let status = Command::new("npm").arg("run").arg("format").current_dir(&pwa_dir)
        .status().map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error("Prettier failed".to_string()));
    }
    let status = Command::new("npm").arg("run").arg("lint").current_dir(&pwa_dir)
        .status().map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error("ESLint failed".to_string()));
    }
    tracing::info!("{}", "PWA Assets are clean!");
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_pwa_sync(dir: String) -> Result<Value> {
    use std::fs;
    use std::path::PathBuf;
    use walkdir::WalkDir;
    let pwa_dir = PathBuf::from(&dir);
    if !pwa_dir.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!("PWA directory not found: {}", dir)));
    }
    tracing::info!("Syncing PWA assets in: {}", dir);
    let mut assets = Vec::new();
    for entry in WalkDir::new(&pwa_dir) {
        let entry = entry.map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(&pwa_dir)
                .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
            let rel_str = rel.to_string_lossy().to_string();
            if rel_str != "manifest.json" && !rel_str.starts_with("node_modules") && !rel_str.starts_with('.') {
                assets.push(rel_str);
            }
        }
    }
    let count = assets.len();
    let manifest = serde_json::json!({"version": "1.0.1", "assets": assets});
    fs::write(pwa_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    tracing::info!("   manifest.json generated.");
    Ok(serde_json::json!({"status": "ok", "assets_count": count}))
}

/// Lint and format PWA assets
///
/// # Arguments
/// * `dir` - Directory containing PWA assets
#[verb("lint", "pwa")]
fn lint_pwa(dir: Option<String>) -> Result<Value> {
    do_pwa_lint(dir.unwrap_or_else(|| "pwa-staff".to_string()))
}

/// Generate asset manifest (sync PWA assets)
///
/// # Arguments
/// * `dir` - Directory containing PWA assets
#[verb("sync", "pwa")]
fn sync_pwa(dir: Option<String>) -> Result<Value> {
    do_pwa_sync(dir.unwrap_or_else(|| "pwa-staff".to_string()))
}
