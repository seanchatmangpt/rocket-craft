//! `rocket deps` — dependency management across all Rust workspaces and the PWA.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn current_root() -> Result<std::path::PathBuf> {
    std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))
}

fn sdk_err(e: impl std::fmt::Display) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(format!("{e}"))
}

// ── deps check ───────────────────────────────────────────────────────────────

/// Show outdated dependencies across all Rust workspaces and the PWA.
///
/// Requires cargo-outdated (install: cargo install cargo-outdated).
/// Falls back to .lock-file analysis if not available.
///
/// # Arguments
/// * `workspace` - Restrict to a single workspace
/// * `json`      - Output machine-readable JSON
#[verb("check", "deps")]
fn deps_check(workspace: Option<String>, json: bool) -> Result<Value> {
    let root = current_root()?;

    let report = rocket_sdk::deps::check_outdated(&root, workspace.as_deref())
        .map_err(|e| sdk_err(e))?;

    if json {
        let v = serde_json::to_value(&report).map_err(|e| sdk_err(e))?;
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        return Ok(v);
    }

    rocket_sdk::deps::render_check_report(&report);

    let rust_total: usize = report.rust.iter().map(|w| w.packages.len()).sum();
    let npm_total: usize = report.npm.iter().map(|w| w.packages.len()).sum();

    Ok(serde_json::json!({
        "rust_outdated": rust_total,
        "npm_outdated": npm_total,
        "ok": rust_total == 0 && npm_total == 0,
    }))
}

// ── deps audit ───────────────────────────────────────────────────────────────

/// Run cargo audit and npm audit to find known security vulnerabilities.
///
/// Requires cargo-audit (install: cargo install cargo-audit).
/// # Arguments
/// * `json` - Output machine-readable JSON
#[verb("audit", "deps")]
fn deps_audit(json: bool) -> Result<Value> {
    let root = current_root()?;

    let report =
        rocket_sdk::deps::audit_security(&root).map_err(|e| sdk_err(e))?;

    if json {
        let v = serde_json::to_value(&report).map_err(|e| sdk_err(e))?;
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        return Ok(v);
    }

    rocket_sdk::deps::render_audit_report(&report);

    let total_vulns: u32 = report.workspaces.iter().map(|w| w.vulns.total()).sum();

    Ok(serde_json::json!({
        "total_vulnerabilities": total_vulns,
        "ok": total_vulns == 0,
        "workspaces": report.workspaces.len(),
    }))
}

// ── deps tree ────────────────────────────────────────────────────────────────

/// Print the dependency tree for a workspace.
///
/// # Arguments
/// * `workspace` - Which workspace to show (defaults to tools)
/// * `depth`     - Maximum tree depth (default: 3)
#[verb("tree", "deps")]
fn deps_tree(workspace: Option<String>, depth: Option<u32>) -> Result<Value> {
    let root = current_root()?;

    rocket_sdk::deps::show_tree(&root, workspace.as_deref(), depth)
        .map_err(|e| sdk_err(e))?;

    Ok(serde_json::json!({ "ok": true }))
}
