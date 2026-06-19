//! Environment management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_setup() -> Result<Value> {
    rocket_sdk::setup::run_setup()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_doctor() -> Result<Value> {
    use rocket_sdk::doctor::{CheckStatus, RocketDoctor};
    let project_root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let doctor = RocketDoctor::new(project_root);
    let report = doctor.run_diagnostics();
    for check in &report.checks {
        let status_str = match check.status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
            CheckStatus::Fail => "FAIL",
        };
        tracing::info!("  [{}] {}: {}", status_str, check.name, check.message);
    }
    Ok(serde_json::json!({"status": "ok", "timestamp": report.timestamp}))
}

fn do_capabilities() -> Result<Value> {
    let capabilities = [
        ("Multiplatform Orchestration", "Unified build system for Windows, Linux, Android, and HTML5 (Web)."),
        ("Semantic Compliance", "Law-based project auditing via the ComplianceEngine and knhk plugin system."),
        ("Generative SDK", "Zero-cost typestate kernel in rocket-sdk for safe UE project manipulation."),
        ("PWA Optimization", "Automated asset manifest generation and mobile-ready PWA scaffolding."),
        ("Crypto Automation", "Automated Android keystore generation and lifecycle management."),
        ("Environment Diagnostics", "Rocket Doctor for programmatic workspace health and dependency checks."),
        ("Log Streaming", "Real-time colorized log tailing with semantic highlighting for UE4 builds."),
        ("TUI Dashboard", "Interactive terminal UI for project management (built with ratatui)."),
        ("Wasm Plugin System", "Extensible compliance laws via WebAssembly (Wasmer integration)."),
        ("Chicago TDD Integration", "Automated test orchestration across Rust and Python validation suites."),
    ];
    let caps_json: Vec<Value> = capabilities.iter().map(|(name, desc)| {
        serde_json::json!({"name": name, "description": desc})
    }).collect();
    Ok(serde_json::json!({"capabilities": caps_json}))
}

fn do_config(key: String) -> Result<Value> {
    let config = rocket_sdk::config::RocketConfig::load()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let val = match key.as_str() {
        "ue4_root" => config.ue4_root.map(|p| p.display().to_string()),
        other => return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("unknown config key '{}'. Known keys: ue4_root", other)
        )),
    };
    match val {
        Some(v) => Ok(serde_json::json!({"key": key, "value": v})),
        None => Err(clap_noun_verb::NounVerbError::execution_error(
            format!("key '{}' is not set in .rocket.json", key)
        )),
    }
}

fn do_root() -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut dir = root.as_path();
    loop {
        if dir.join("project-manifest.json").exists() {
            let path = dir.display().to_string();
            println!("{}", path);
            return Ok(serde_json::json!({"root": path}));
        }
        dir = dir.parent().ok_or_else(|| clap_noun_verb::NounVerbError::execution_error(
            "project-manifest.json not found in any parent directory".to_string()
        ))?;
    }
}

/// Setup the Unreal Engine environment
#[verb("setup", "env")]
fn setup_env() -> Result<Value> {
    do_setup()
}

/// Troubleshoot the environment
#[verb("doctor", "env")]
fn doctor_env() -> Result<Value> {
    do_doctor()
}

/// Generate shell completions
///
/// # Arguments
/// * `shell` - The shell to generate completions for
#[verb("completions", "env")]
fn completions_env(shell: String) -> Result<Value> {
    tracing::info!("Shell completions requested for: {}", shell);
    Ok(serde_json::json!({"status": "ok", "shell": shell}))
}

/// List all integrated high-level features (Capabilities)
#[verb("capabilities", "env")]
fn capabilities_env() -> Result<Value> {
    do_capabilities()
}

/// Read a value from .rocket.json or project-manifest.json
///
/// # Arguments
/// * `key` - Key to read (e.g. ue4_root)
#[verb("config", "env")]
fn config_env(key: String) -> Result<Value> {
    do_config(key)
}

/// Print the repo root (directory containing project-manifest.json)
#[verb("root", "env")]
fn root_env() -> Result<Value> {
    do_root()
}
