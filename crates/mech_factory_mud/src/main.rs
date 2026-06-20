use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use mech_factory_mud::export::export_ue4;
use mech_factory_mud::receipt::verify_receipt_chain;
use mech_factory_mud::world::Simulation;

#[derive(Parser)]
#[command(name = "mech-factory-mud")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Simulate {
        #[arg(long)]
        scenario: String,
    },
    Verify,
    Replay,
    ExportUe4,
    Report,
    Falsify {
        #[arg(long)]
        case: String,
    },
    Counterfactual {
        #[arg(long)]
        case: String,
    },
}

#[derive(serde::Serialize)]
struct FalsifyResult {
    #[serde(rename = "case")]
    case_name: String,
    expected: String,
    actual: String,
    refusal_reason: String,
    passed: bool,
}

#[derive(serde::Serialize)]
struct CfResult {
    #[serde(rename = "case")]
    case_name: String,
    expected_effect: String,
    actual_effect: String,
    passed: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let out_dir = PathBuf::from("generated/mech_factory_mud");
    fs::create_dir_all(&out_dir)?;

    match cli.command {
        Commands::Simulate { scenario } => {
            let sim = Simulation::run(&scenario);

            // Write scenario-prefixed files
            let scenario_prefix = out_dir.join(format!("{}.", scenario));
            fs::write(
                scenario_prefix.with_extension("trace.json"),
                serde_json::to_string_pretty(&sim.traces)?,
            )?;
            fs::write(
                scenario_prefix.with_extension("ocel.json"),
                serde_json::to_string_pretty(&sim.ocel)?,
            )?;
            let mut receipt_lines = String::new();
            for r in &sim.receipts {
                receipt_lines.push_str(&serde_json::to_string(r)?);
                receipt_lines.push('\n');
            }
            fs::write(
                scenario_prefix.with_extension("receipts.jsonl"),
                &receipt_lines,
            )?;
            fs::write(
                scenario_prefix.with_extension("projection_manifest.json"),
                serde_json::to_string_pretty(&sim.projections)?,
            )?;
            fs::write(
                scenario_prefix.with_extension("report.json"),
                serde_json::to_string_pretty(&sim.report)?,
            )?;
            fs::write(
                scenario_prefix.with_extension("report.md"),
                format!(
                    "# Simulation Report: {}\n\nStatus: {}\nReason: {:?}\n",
                    scenario, sim.report.status, sim.report.reason
                ),
            )?;

            // Also write default/unprefixed files to be loaded by verify
            fs::write(
                out_dir.join("trace.json"),
                serde_json::to_string_pretty(&sim.traces)?,
            )?;
            fs::write(
                out_dir.join("ocel.json"),
                serde_json::to_string_pretty(&sim.ocel)?,
            )?;
            fs::write(out_dir.join("receipts.jsonl"), &receipt_lines)?;
            fs::write(
                out_dir.join("projection_manifest.json"),
                serde_json::to_string_pretty(&sim.projections)?,
            )?;
            fs::write(
                out_dir.join("report.json"),
                serde_json::to_string_pretty(&sim.report)?,
            )?;
            fs::write(
                out_dir.join("report.md"),
                format!(
                    "# Simulation Report: {}\n\nStatus: {}\nReason: {:?}\n",
                    scenario, sim.report.status, sim.report.reason
                ),
            )?;

            println!("Simulated scenario: {}", scenario);
        }
        Commands::Verify => {
            let trace_path = out_dir.join("trace.json");
            let ocel_path = out_dir.join("ocel.json");
            let receipts_path = out_dir.join("receipts.jsonl");
            let proj_manifest_path = out_dir.join("projection_manifest.json");

            if !trace_path.exists()
                || !ocel_path.exists()
                || !receipts_path.exists()
                || !proj_manifest_path.exists()
            {
                anyhow::bail!("Missing simulation files in generated/mech_factory_mud/");
            }

            let receipts_data = fs::read_to_string(&receipts_path)?;
            let mut receipts = Vec::new();
            for line in receipts_data.lines() {
                if !line.trim().is_empty() {
                    receipts.push(serde_json::from_str(line)?);
                }
            }
            verify_receipt_chain(&receipts)?;

            let ue4_dt_dir = out_dir.join("ue4/DataTables");
            let fs_csv = ue4_dt_dir.join("FactoryStations.csv");
            let wr_csv = ue4_dt_dir.join("WalkthroughRoute.csv");
            if !fs_csv.exists() || !wr_csv.exists() {
                anyhow::bail!(
                    "Missing FactoryStations.csv or WalkthroughRoute.csv under ue4/DataTables/"
                );
            }

            println!("PASS");
        }
        Commands::Replay => {
            let receipts_path = out_dir.join("factory_walkthrough.receipts.jsonl");
            if !receipts_path.exists() {
                anyhow::bail!("Missing factory_walkthrough.receipts.jsonl");
            }
            let receipts_data = fs::read_to_string(&receipts_path)?;
            let mut receipts = Vec::new();
            for line in receipts_data.lines() {
                if !line.trim().is_empty() {
                    receipts.push(serde_json::from_str(line)?);
                }
            }
            verify_receipt_chain(&receipts)?;
            println!("PASS");
        }
        Commands::ExportUe4 => {
            let manifest_path = out_dir.join("projection_manifest.json");
            if !manifest_path.exists() {
                anyhow::bail!("Missing projection_manifest.json");
            }
            let data = fs::read_to_string(manifest_path)?;
            let manifest: Vec<mech_factory_mud::projection::ProjectionRow> =
                serde_json::from_str(&data)?;
            export_ue4(&manifest, &out_dir.join("ue4"))?;
            println!("Exported to UE4");
        }
        Commands::Report => {
            println!("Reporting...");
        }
        Commands::Falsify { case } => {
            let falsify_dir = out_dir.join("falsification");
            fs::create_dir_all(&falsify_dir)?;

            let cases = vec![
                ("FALSIFY_RECEIPT_PREV_HASH", "RECEIPT_PREV_HASH_BROKEN"),
                (
                    "FALSIFY_RECEIPT_PAYLOAD_MUTATION",
                    "RECEIPT_PAYLOAD_MUTATION",
                ),
                ("FALSIFY_RECEIPT_SEQUENCE_GAP", "RECEIPT_SEQUENCE_GAP"),
                (
                    "FALSIFY_PROJECTION_WITHOUT_SOURCE_RECEIPT",
                    "PROJECTION_WITHOUT_SOURCE_RECEIPT",
                ),
                (
                    "FALSIFY_OCEL_EVENT_WITHOUT_OBJECT",
                    "OCEL_EVENT_WITHOUT_OBJECT",
                ),
                (
                    "FALSIFY_OCEL_PART_EVENT_WITHOUT_PART_OBJECT",
                    "OCEL_PART_EVENT_WITHOUT_PART_OBJECT",
                ),
                ("FALSIFY_ROUTE_UNREACHABLE", "ROUTE_UNREACHABLE"),
                ("FALSIFY_UE4_HEADER_CSV_MISMATCH", "UE4_HEADER_CSV_MISMATCH"),
            ];

            let mut results = Vec::new();
            for (c_name, expected_reason) in cases {
                if case != "all" && case != c_name {
                    continue;
                }
                let sim = Simulation::run(c_name);
                let actual_status = &sim.report.status;
                let actual_reason = sim.report.reason.as_deref().unwrap_or("");

                let passed = actual_status == "REFUSED" && actual_reason == expected_reason;
                results.push(FalsifyResult {
                    case_name: c_name.to_string(),
                    expected: "REFUSED".to_string(),
                    actual: actual_status.clone(),
                    refusal_reason: actual_reason.to_string(),
                    passed,
                });
            }

            fs::write(
                falsify_dir.join("falsification_report.json"),
                serde_json::to_string_pretty(&results)?,
            )?;

            let mut md = String::new();
            md.push_str("# Falsification Report\n\n");
            for r in &results {
                let status_str = if r.passed { "PASS" } else { "FAIL" };
                md.push_str(&format!("- {}: {}\n", r.case_name, status_str));
            }
            fs::write(falsify_dir.join("falsification_report.md"), md)?;

            println!("Falsified case: {}", case);
        }
        Commands::Counterfactual { case } => {
            let cf_dir = out_dir.join("counterfactuals");
            fs::create_dir_all(&cf_dir)?;

            let cases = vec![
                ("COUNTERFACTUAL_WITH_SOCKET", "ADMITTED"),
                ("COUNTERFACTUAL_WITHOUT_SOCKET", "REFUSED_MISSING_SOCKET"),
                ("COUNTERFACTUAL_SKIN_DOES_NOT_HIDE_VENT", "ADMITTED"),
                ("COUNTERFACTUAL_SKIN_HIDES_VENT", "REFUSED_SKIN_HIDES_VENT"),
                ("COUNTERFACTUAL_CLEARANCE_OK", "ADMITTED"),
                (
                    "COUNTERFACTUAL_CLEARANCE_BLOCKED",
                    "REFUSED_BLOCKED_CLEARANCE",
                ),
                ("COUNTERFACTUAL_ROUTE_CONNECTED", "ADMITTED"),
                ("COUNTERFACTUAL_ROUTE_BROKEN", "REFUSED_ROUTE_BROKEN"),
            ];

            let mut results = Vec::new();
            for (c_name, expected_effect) in cases {
                if case != "all" && case != c_name {
                    continue;
                }
                let sim = Simulation::run(c_name);
                let actual_effect = if sim.report.status == "REFUSED" {
                    sim.report
                        .reason
                        .clone()
                        .unwrap_or_else(|| "REFUSED".to_string())
                } else {
                    sim.report.status.clone()
                };
                let passed = actual_effect == expected_effect;
                results.push(CfResult {
                    case_name: c_name.to_string(),
                    expected_effect: expected_effect.to_string(),
                    actual_effect,
                    passed,
                });
            }

            fs::write(
                cf_dir.join("counterfactual_report.json"),
                serde_json::to_string_pretty(&results)?,
            )?;

            let mut md = String::new();
            md.push_str("# Counterfactual Report\n\n");
            for r in &results {
                let status_str = if r.passed { "PASS" } else { "FAIL" };
                md.push_str(&format!("- {}: {}\n", r.case_name, status_str));
            }
            fs::write(cf_dir.join("counterfactual_report.md"), md)?;

            println!("Counterfactual case: {}", case);
        }
    }
    Ok(())
}
