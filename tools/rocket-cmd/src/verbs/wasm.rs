//! WASM plugin management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_wasm_run(file: String) -> Result<Value> {
    use std::path::{Path, PathBuf};
    tracing::info!("{}", "=== WASM Plugin Execution ===");
    let path = PathBuf::from(&file);
    if !path.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "WASM file not found: {}",
            file
        )));
    }
    let mut plugin_host = knhk::plugin::PluginHost::new();
    tracing::info!("Loading plugin: {}", path.display());
    match plugin_host.load_law(&path) {
        Ok(law) => {
            tracing::info!(
                "Loaded: {} — {}",
                knhk::Law::name(&law),
                knhk::Law::description(&law)
            );
            match knhk::Law::validate(&law, Path::new(".")) {
                Ok(_) => tracing::info!("Validation passed"),
                Err(e) => tracing::info!("Validation failed: {}", e.message),
            }
        }
        Err(e) => tracing::info!("Failed to load WASM plugin: {}", e),
    }
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_wasm_verify(file: String, min_size: Option<u64>) -> Result<Value> {
    use std::path::Path;
    let min_size = min_size.unwrap_or(1_048_576);
    let path = Path::new(&file);
    // Basic check: read file, verify WASM magic bytes \0asm
    let bytes = std::fs::read(path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("cannot read {}: {}", file, e))
    })?;
    let size = bytes.len() as u64;
    if size < min_size {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "WASM file too small: {} bytes (min: {} bytes)",
            size, min_size
        )));
    }
    if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "Invalid WASM magic bytes in: {}",
            file
        )));
    }
    println!("PASS: {} ({} MB)", file, size / 1_048_576);
    Ok(serde_json::json!({"status": "pass", "path": file, "size_bytes": size}))
}

/// Execute a WASM plugin directly
///
/// # Arguments
/// * `file` - Path to the WASM file
#[verb("run", "wasm")]
fn run_wasm(file: String) -> Result<Value> {
    do_wasm_run(file)
}

/// Verify a .wasm file has valid magic bytes and is not a stub
///
/// # Arguments
/// * `file` - Path to .wasm file, or directory to search
/// * `min_size` - Minimum file size in bytes
#[verb("verify", "wasm")]
fn verify_wasm(file: String, min_size: Option<u64>) -> Result<Value> {
    do_wasm_verify(file, min_size)
}
