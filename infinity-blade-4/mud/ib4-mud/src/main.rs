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
