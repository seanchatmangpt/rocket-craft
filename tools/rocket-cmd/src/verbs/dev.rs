//! `rocket dev` — combined developer-experience entry point
//!
//! Prints a condensed health snapshot of the monorepo, then enters live
//! watch mode (incremental rebuilds on every file save).  A single command
//! that keeps a developer in the fast-feedback loop.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_dev(workspace: Option<String>, verbose: bool, skip_status: bool) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let mut cfg = rocket_sdk::dev::DevConfig::new(root);

    if let Some(ws) = workspace {
        cfg = cfg.only(ws);
    }
    if verbose {
        cfg = cfg.verbose();
    }
    if skip_status {
        cfg = cfg.skip_status();
    }

    rocket_sdk::dev::run(cfg)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    // `run` loops forever (watch mode); only reached on Ctrl-C / channel close.
    Ok(serde_json::json!({"status": "ok"}))
}

/// Start the developer loop: status snapshot then live rebuild watcher.
///
/// 1. Prints a one-line header with the current git branch.
/// 2. Runs a parallel health check of the whole monorepo and shows any
///    warnings or failures (pass `--verbose` to show passing checks too).
/// 3. Enters watch mode — every `.rs`, `.toml`, or `.ts` change triggers
///    `cargo check` (or `npm run build:ts` for the PWA) in the affected
///    workspace.  Press Ctrl-C to exit.
///
/// # Arguments
/// * `workspace`    - Optional: restrict watching to one named workspace
///                    (tools, nexus-engine, blueprint-rs, unify-rs,
///                     infinity-blade-4/mud, chicago-tdd-tools, pwa-staff)
/// * `verbose`      - Show all health checks, not just failures/warnings
/// * `skip_status`  - Skip the initial status snapshot and go straight to
///                    watch mode
#[verb("start", "dev")]
fn dev_start(workspace: Option<String>, verbose: bool, skip_status: bool) -> Result<Value> {
    do_dev(workspace, verbose, skip_status)
}
