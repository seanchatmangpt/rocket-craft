use clap::{Parser, Subcommand};

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
    ExportUe4 {
        #[arg(long)]
        out: String,
    },
    Report,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Simulate { scenario } => {
            println!("Simulating scenario: {}", scenario);
        }
        Commands::Verify => {
            println!("Verifying...");
        }
        Commands::Replay => {
            println!("Replaying...");
        }
        Commands::ExportUe4 { out } => {
            println!("Exporting to {}", out);
        }
        Commands::Report => {
            println!("Reporting...");
        }
    }
    Ok(())
}
