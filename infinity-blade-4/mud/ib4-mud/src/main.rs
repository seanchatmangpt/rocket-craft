//! # Infinity Blade IV MUD CLI Executable
//!
//! This executable provides the command-line entrypoint for the Infinity Blade IV MUD.
//! It processes argument options using `clap` to either start a brand-new adventure
//! or restore a previously saved state from a JSON file, starting the interactive terminal REPL.
//!
//! ## System Integration
//! As the main driver binary, it configures logging telemetry, initializes standard IO handles,
//! constructs the runtime `GameSession` via `ib4-mud::session::GameSession`, and handovers control
//! to the REPL loop located in `ib4-mud::repl::run_repl`.

use clap::{Parser, Subcommand};
use ib4_mud::repl::run_repl;
use ib4_mud::session::GameSession;

#[derive(Parser)]
#[command(name = "ib4")]
#[command(about = "Infinity Blade 4 \u{2014} Command Line MUD Simulation")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Start a new game
    New {
        /// Character name
        #[arg(default_value = "Siris")]
        name: String,
    },
    /// Load from save file
    Load {
        #[arg(short, long, default_value = "ib4_save.json")]
        file: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut session = match cli
        .command
        .unwrap_or(CliCommand::New {
            name: "Siris".to_string(),
        }) {
        CliCommand::New { name } => {
            tracing::info!("Creating new game for {}...", name);
            GameSession::new(&name)
        }
        CliCommand::Load { file } => {
            let json = std::fs::read_to_string(&file)
                .map_err(|e| anyhow::anyhow!("Cannot load '{}': {}", file, e))?;
            GameSession::from_json(&json)?
        }
    };

    run_repl(&mut session)
}
