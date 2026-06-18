use clap::{Parser, Subcommand};
use simulator_core::{RocketContract, SimulationEngine};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rocket-simulator")]
#[command(about = "Rocket Simulator CLI", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PATH")]
    workspace_root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Prepare manufacturing by generating the world artifact
    Prepare {
        #[arg(short, long)]
        name: String,

        #[arg(short, long, default_value_t = 42)]
        world_seed: u64,
    },
    /// Run the E2E Playwright simulation and verify the receipt
    RunE2e {
        #[arg(short, long)]
        name: String,

        #[arg(short, long, default_value_t = 42)]
        world_seed: u64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Default workspace root to parent directory if not provided
    let workspace_root = cli.workspace_root.unwrap_or_else(|| {
        let current = std::env::current_dir().unwrap();
        // If current dir is simulator-core, go up two levels; if it's rocket-simulator, go up one level
        if current.ends_with("simulator-core") {
            current.parent().unwrap().parent().unwrap().to_path_buf()
        } else if current.ends_with("rocket-simulator") {
            current.parent().unwrap().to_path_buf()
        } else {
            current
        }
    });

    match cli.command {
        Commands::Prepare { name, world_seed } => {
            let contract = RocketContract::new(name, world_seed);
            let engine = SimulationEngine::new(contract, workspace_root);
            engine.prepare_manufacturing()?;
            println!("Prepared manufacturing successfully!");
        }
        Commands::RunE2e { name, world_seed } => {
            let contract = RocketContract::new(name, world_seed);
            let engine = SimulationEngine::new(contract, workspace_root);

            println!("Starting E2E simulation coordination natively in Rust...");
            let receipt = engine.run_e2e_simulation()?;
            println!("E2E Simulation SUCCESSful!");
            println!("Receipt details:");
            println!("  Verdict: {}", receipt.verdict);
            println!("  Visual Delta: {}", receipt.visual_delta);
            println!("  Contract Hash: {}", receipt.contract_hash);
            if let Some(sig) = receipt.signature {
                println!("  Signature: {}", sig);
            }
        }
    }
    Ok(())
}
