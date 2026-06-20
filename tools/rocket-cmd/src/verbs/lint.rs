//! `rocket lint` noun — check and fix lint across all workspaces.
//!
//! Verbs:
//!   lint check  — runs clippy + eslint/prettier across all workspaces, reports findings
//!   lint fix    — runs clippy --fix + rustfmt + eslint --fix, mutates files

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use rocket_sdk::lint::{LintConfig, run_lint};
use serde_json::Value;

fn current_root() -> std::result::Result<std::path::PathBuf, clap_noun_verb::NounVerbError> {
    std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))
}

fn lint_error_to_noun_verb(e: rocket_sdk::lint::LintError) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(format!("{e}"))
}

/// Check all workspaces for lint errors without modifying files.
///
/// Runs `cargo clippy --workspace --all-features -- -D warnings` for every Rust
/// workspace (tools, nexus-engine, blueprint-rs, unify-rs, infinity-blade-4/mud,
/// chicago-tdd-tools) and `npm run lint` for `pwa-staff/`.
///
/// # Arguments
/// * `workspace` - Restrict to a single workspace name (e.g. nexus-engine)
/// * `json`      - Output machine-readable JSON instead of coloured prose
#[verb("check", "lint")]
fn lint_check(workspace: Option<String>, json: bool) -> Result<Value> {
    let root = current_root()?;

    let config = LintConfig {
        root,
        fix: false,
        workspace,
        json,
    };

    let report = run_lint(config).map_err(lint_error_to_noun_verb)?;

    if json {
        let v = serde_json::to_value(&report)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        return Ok(v);
    }

    report.print();

    Ok(serde_json::json!({
        "overall":         if report.all_pass { "PASS" } else { "FAIL" },
        "total_warnings":  report.total_warnings,
        "total_errors":    report.total_errors,
        "workspace_count": report.results.len(),
    }))
}

/// Fix auto-fixable lint errors and format all code.
///
/// Runs `cargo clippy --fix`, `cargo fmt --all`, and `npx eslint --fix src/`
/// across all workspaces.  Only modifies files; does not fail on unfixable
/// warnings left behind after the fix pass.
///
/// # Arguments
/// * `workspace` - Restrict to a single workspace name (e.g. nexus-engine)
#[verb("fix", "lint")]
fn lint_fix(workspace: Option<String>) -> Result<Value> {
    let root = current_root()?;

    let config = LintConfig {
        root,
        fix: true,
        workspace,
        json: false,
    };

    let report = run_lint(config).map_err(lint_error_to_noun_verb)?;

    report.print();

    Ok(serde_json::json!({
        "overall":         if report.all_pass { "PASS" } else { "FAIL" },
        "total_warnings":  report.total_warnings,
        "total_errors":    report.total_errors,
        "workspace_count": report.results.len(),
    }))
}
