//! MUD inspection commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_mud_inspect(file: Option<String>, summary: bool) -> Result<Value> {
    crate::inspect::run_inspect(file.as_deref(), summary)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

/// Inspect MUD saved states, receipts, and OCEL event trails interactively
///
/// # Arguments
/// * `file` - Optional path to a specific JSON file to inspect directly
/// * `summary` - Only show a count summary of all detected files and exit
#[verb("inspect", "mud")]
fn inspect_mud(file: Option<String>, summary: Option<bool>) -> Result<Value> {
    do_mud_inspect(file, summary.unwrap_or(false))
}
