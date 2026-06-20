//! Live-rebuild watcher commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_watch(workspace: Option<String>) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let mut cfg = rocket_sdk::watch::WatchConfig::new(root);
    if let Some(name) = workspace {
        cfg = cfg.only(name);
    }

    rocket_sdk::watch::run(cfg)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    // `run` loops forever; this is only reached if it somehow returns.
    Ok(serde_json::json!({"status": "ok"}))
}

/// Watch all Rust workspaces and the PWA for file changes and auto-rebuild the affected workspace.
///
/// Watches `.rs` and `.toml` files in all Rust workspace roots and `pwa-staff/src/**/*.ts`.
/// Debounces 500 ms after the last event, then runs `cargo check --workspace` for Rust
/// workspaces (or `npm run build:ts` for the PWA).  When test files change, also runs
/// `cargo test --workspace`.
///
/// # Arguments
/// * `workspace` - Optional: restrict watching to a single named workspace
///   (one of: tools, nexus-engine, blueprint-rs, unify-rs, infinity-blade-4/mud,
///   chicago-tdd-tools, pwa-staff)
#[verb("watch", "workspace")]
fn watch_workspace(workspace: Option<String>) -> Result<Value> {
    do_watch(workspace)
}
