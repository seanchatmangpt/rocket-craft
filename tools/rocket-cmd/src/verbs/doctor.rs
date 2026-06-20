//! UE4 / SDK environment health checks

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_doctor_check() -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let report = rocket_sdk::doctor::RocketDoctor::new(root).run_diagnostics();

    let checks: Vec<Value> = report
        .checks
        .iter()
        .map(|c| {
            let status = match c.status {
                rocket_sdk::doctor::CheckStatus::Pass => "PASS",
                rocket_sdk::doctor::CheckStatus::Warn => "WARN",
                rocket_sdk::doctor::CheckStatus::Fail => "FAIL",
            };
            println!("[{status}] {}: {}", c.name, c.message);
            let mut obj = serde_json::json!({
                "name": c.name,
                "status": status,
                "message": c.message,
            });
            if let Some(d) = &c.details {
                obj["details"] = Value::String(d.clone());
            }
            obj
        })
        .collect();

    let pass_count = report
        .checks
        .iter()
        .filter(|c| c.status == rocket_sdk::doctor::CheckStatus::Pass)
        .count();
    let fail_count = report
        .checks
        .iter()
        .filter(|c| c.status == rocket_sdk::doctor::CheckStatus::Fail)
        .count();
    let warn_count = report
        .checks
        .iter()
        .filter(|c| c.status == rocket_sdk::doctor::CheckStatus::Warn)
        .count();

    let overall = if fail_count > 0 {
        "FAIL"
    } else if warn_count > 0 {
        "WARN"
    } else {
        "PASS"
    };
    println!("\n[{overall}] {pass_count} passed, {warn_count} warned, {fail_count} failed");

    Ok(serde_json::json!({
        "overall": overall,
        "pass": pass_count,
        "warn": warn_count,
        "fail": fail_count,
        "checks": checks,
        "timestamp": report.timestamp.to_rfc3339(),
    }))
}

/// Diagnose the UE4 and SDK environment for this project
///
/// Checks: UE4_ROOT resolution, RunUAT.sh existence, HTML5 package status,
/// Python 3, Node.js, and Blender availability.
#[verb("check", "doctor")]
fn doctor_check() -> Result<Value> {
    do_doctor_check()
}
