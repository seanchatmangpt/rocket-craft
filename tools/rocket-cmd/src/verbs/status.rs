//! `rocket status` — rich project health dashboard

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_status(quiet: bool, json: bool) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    // Run checks inside a Tokio runtime (rocket-cmd ships tokio as a dep).
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("tokio: {e}")))?;

    let report = rt
        .block_on(rocket_sdk::status::run_status(root))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if json {
        let serialized = serde_json::to_value(&report)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
        println!("{}", serde_json::to_string_pretty(&serialized).unwrap_or_default());
        return Ok(serialized);
    }

    rocket_sdk::status::render_text(&report, quiet);

    // Build a compact JSON summary for the clap-noun-verb return value.
    let total = report.total_checks();
    let pass = report.passing();
    let fail = report.failing();
    let warn = report.warning();
    Ok(serde_json::json!({
        "total": total,
        "pass": pass,
        "fail": fail,
        "warn": warn,
        "ok": fail == 0,
    }))
}

/// Show a rich health dashboard for the entire monorepo.
///
/// Runs Git, environment, all Rust workspaces, PWA, and UE4 project checks in
/// parallel.  Exits with a non-zero summary when any check fails.
///
/// # Arguments
/// * `quiet` - Only print failing or warning checks (suppress passing ones)
/// * `json`  - Output machine-readable JSON instead of coloured text
#[verb("status", "workspace")]
fn status_workspace(quiet: bool, json: bool) -> Result<Value> {
    do_status(quiet, json)
}
