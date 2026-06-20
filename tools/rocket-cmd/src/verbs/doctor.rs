//! UE4 / SDK environment health checks

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use rocket_sdk::doctor::{
    CheckCategory, print_check, print_health_score, print_json_report, print_section,
    RocketDoctor,
};
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
        // -- Required section
        print_section("Required");
        for check in report
            .checks
            .iter()
            .filter(|c| c.category == Some(CheckCategory::Required))
        {
            print_check(check);
        }

        // -- Optional section
        print_section("Optional");
        for check in report
            .checks
            .iter()
            .filter(|c| c.category == Some(CheckCategory::Optional))
        {
            print_check(check);
        }

        // Ungrouped checks (no category assigned)
        let ungrouped: Vec<_> = report
            .checks
            .iter()
            .filter(|c| c.category.is_none())
            .collect();
        if !ungrouped.is_empty() {
            print_section("Other");
            for check in &ungrouped {
                print_check(check);
            }
        }

        print_health_score(&report);
    }

    let pass = report.required_pass_count();
    let total = report.required_total();
    let all_pass = report.all_required_pass();

    Ok(serde_json::json!({
        "overall": if all_pass { "PASS" } else { "FAIL" },
        "required_pass": pass,
        "required_total": total,
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
