//! UE4 / SDK environment health checks

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use rocket_sdk::doctor::{print_json_report, render_diagnostics, RocketDoctor};
use serde_json::Value;

fn do_doctor_check(fix: bool, json: bool) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let doctor = RocketDoctor::new(root);

    let report = if fix {
        doctor.run_fix_and_recheck()
    } else {
        doctor.run_diagnostics()
    };

    if json {
        print_json_report(&report);
    } else {
        render_diagnostics(&report);
    }

    Ok(serde_json::json!({
        "overall": if report.all_required_pass() { "PASS" } else { "FAIL" },
        "required_pass": report.required_pass_count(),
        "required_total": report.required_total(),
        "timestamp": report.timestamp.to_rfc3339(),
    }))
}

/// Diagnose the UE4 and SDK environment for this project.
///
/// Checks are grouped into Required (blocking) and Optional sections.
/// The health score counts only required checks.
///
/// Flags:
///   --fix   Auto-repair fixable failures (rustfmt/clippy, npm ci, .env), then recheck
///   --json  Emit structured JSON report for CI consumption
#[verb("check", "doctor")]
fn doctor_check(fix: bool, json: bool) -> Result<Value> {
    do_doctor_check(fix, json)
}
