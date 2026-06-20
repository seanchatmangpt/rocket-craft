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
    let path = Path::new(&file);
    let info = rocket_sdk::wasm::validate_wasm_artifact(path, min_size)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{:#}", e)))?;
    println!(
        "PASS: {} ({:.1} MB)",
        file,
        info.size_bytes as f64 / 1_048_576.0
    );
    Ok(serde_json::json!({
        "status": "pass",
        "path": file,
        "size_bytes": info.size_bytes,
        "size_mb": info.size_bytes as f64 / 1_048_576.0,
    }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_wasm(dir: &TempDir, name: &str, size: usize) -> String {
        let path = dir.path().join(name);
        let mut data = vec![0x00u8, 0x61, 0x73, 0x6d];
        data.extend(vec![0u8; size]);
        std::fs::write(&path, &data).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn write_stub(dir: &TempDir, name: &str) -> String {
        let path = dir.path().join(name);
        std::fs::write(&path, b"\x00asm\x01\x00\x00\x00").unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn verify_real_wasm_returns_pass() {
        let dir = TempDir::new().unwrap();
        let path = write_wasm(&dir, "game.wasm", 1024 * 1024 + 1);
        let val = do_wasm_verify(path.clone(), None).unwrap();
        assert_eq!(val["status"], "pass");
        assert!(val["size_bytes"].as_u64().unwrap() > 1024 * 1024);
    }

    #[test]
    fn verify_stub_fails_with_error() {
        let dir = TempDir::new().unwrap();
        let path = write_stub(&dir, "stub.wasm");
        let err = do_wasm_verify(path, None).unwrap_err();
        assert!(err.to_string().contains("stub") || err.to_string().contains("bytes"),
            "error must mention stub size: {err}");
    }

    #[test]
    fn verify_missing_file_returns_error() {
        let err = do_wasm_verify("/nonexistent/path/to/game.wasm".into(), None).unwrap_err();
        assert!(err.to_string().contains("nonexistent") || err.to_string().contains("cannot stat"),
            "error must reference path: {err}");
    }

    #[test]
    fn verify_returns_size_mb_field() {
        let dir = TempDir::new().unwrap();
        let path = write_wasm(&dir, "game.wasm", 1024 * 1024 + 1);
        let val = do_wasm_verify(path, None).unwrap();
        let mb = val["size_mb"].as_f64().expect("size_mb must be float");
        assert!(mb > 1.0, "size_mb must be > 1 for a real wasm: {mb}");
    }

    #[test]
    fn verify_custom_min_size_passes_small_wasm() {
        let dir = TempDir::new().unwrap();
        let path = write_wasm(&dir, "small.wasm", 500);
        let val = do_wasm_verify(path, Some(100)).unwrap();
        assert_eq!(val["status"], "pass");
    }

    #[test]
    fn wasm_run_missing_file_returns_error() {
        let err = do_wasm_run("/no/such/plugin.wasm".into()).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("WASM file"),
            "must mention file not found: {err}");
    }
}
