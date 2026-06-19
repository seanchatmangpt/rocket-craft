//! Ontology registry management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_registry_copy(manifest: String) -> Result<Value> {
    crate::registry::run_copy(&manifest)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_registry_index() -> Result<Value> {
    crate::registry::run_index()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

/// Consolidate ontology files from all_semantic_files.txt into the catalogue
///
/// # Arguments
/// * `manifest` - Optional custom path to the manifest file
#[verb("copy", "registry")]
fn copy_registry(manifest: Option<String>) -> Result<Value> {
    do_registry_copy(manifest.unwrap_or_else(|| "all_semantic_files.txt".to_string()))
}

/// Index O-crates in the ontology catalogue and build registry index.json
#[verb("index", "registry")]
fn index_registry() -> Result<Value> {
    do_registry_index()
}
