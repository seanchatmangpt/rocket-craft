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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let out_dir = PathBuf::from("generated/mech_factory_mud");
    fs::create_dir_all(&out_dir)?;

    match cli.command {
        Commands::Simulate { scenario } => {
            let sim = Simulation::run(&scenario);
            let prefix = out_dir.join(format!("{}.", scenario));
            fs::write(
                prefix.with_extension("trace.json"),
                serde_json::to_string_pretty(&sim.traces)?,
            )?;
            fs::write(
                prefix.with_extension("ocel.json"),
                serde_json::to_string_pretty(&sim.ocel)?,
            )?;
            let mut receipt_lines = String::new();
            for r in &sim.receipts {
                receipt_lines.push_str(&serde_json::to_string(r)?);
                receipt_lines.push('\n');
            }
            fs::write(prefix.with_extension("receipts.jsonl"), receipt_lines)?;
            fs::write(
                prefix.with_extension("projection_manifest.json"),
                serde_json::to_string_pretty(&sim.projections)?,
            )?;
            fs::write(
                prefix.with_extension("report.json"),
                serde_json::to_string_pretty(&sim.report)?,
            )?;
            fs::write(
                prefix.with_extension("report.md"),
                format!(
                    "# Simulation Report: {}\n\nStatus: {}",
                    scenario, sim.report.status
                ),
            )?;
            println!("Simulated scenario: {}", scenario);
        }
        Commands::Verify => {
            let base = out_dir.join("factory_walkthrough.");
            if !base.with_extension("trace.json").exists() {
                anyhow::bail!("Missing trace.json");
            }
            if !base.with_extension("ocel.json").exists() {
                anyhow::bail!("Missing ocel.json");
            }
            if !base.with_extension("receipts.jsonl").exists() {
                anyhow::bail!("Missing receipts.jsonl");
            }
            if !base.with_extension("projection_manifest.json").exists() {
                anyhow::bail!("Missing projection_manifest.json");
            }

            let ue4_dir = out_dir.join("ue4");
            if !ue4_dir.join("DataTables/FactoryStations.csv").exists() {
                anyhow::bail!("Missing CSV");
            }
            println!("Verification passed.");
        }
        Commands::Replay => {
            let receipts_path = out_dir.join("factory_walkthrough.receipts.jsonl");
            let data = fs::read_to_string(receipts_path)?;
            let mut receipts = Vec::new();
            for line in data.lines() {
                receipts.push(serde_json::from_str(line)?);
            }
            verify_receipt_chain(&receipts)?;
            println!("Replay passed.");
        }
        Commands::ExportUe4 => {
            let manifest_path = out_dir.join("factory_walkthrough.projection_manifest.json");
            let data = fs::read_to_string(manifest_path)?;
            let manifest: Vec<mech_factory_mud::projection::ProjectionRow> =
                serde_json::from_str(&data)?;
            export_ue4(&manifest, &out_dir.join("ue4"))?;
            println!("Exported to UE4");
        }
        Commands::Report => {
            println!("Reporting...");
        }
    }
    Ok(())
}
