use anyhow::Result;
use clap::{Parser, Subcommand};
use pipeline_core::config::PipelineConfig;

/// Autonomous 3-D asset pipeline — watches a directory and converts 3D files
/// to Unreal-Engine-ready FBX via Blender.
#[derive(Parser, Debug)]
#[command(name = "pipeline-cli", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the pipeline once (scan → validate → convert → stage).
    Run {
        /// Path to the TOML config file.
        #[arg(short, long, default_value = "pipeline.toml")]
        config: std::path::PathBuf,
    },

    /// Watch the configured directory for new files and process them as they arrive.
    Watch {
        /// Path to the TOML config file.
        #[arg(short, long, default_value = "pipeline.toml")]
        config: std::path::PathBuf,
    },

    /// Print an example configuration file to stdout.
    InitConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::InitConfig => {
            print!("{}", PipelineConfig::example_toml());
        }

        Commands::Run { config } => {
            let cfg = PipelineConfig::from_file(&config)?;
            cfg.validate()?;

            // Initialise tracing using the configured log level.
            tracing_subscriber::fmt()
                .with_env_filter(&cfg.pipeline.log_level)
                .init();

            tracing::info!("pipeline run starting");
            tracing::info!(
                watch_dir = %cfg.pipeline.watch_dir.display(),
                output_dir = %cfg.pipeline.output_dir.display(),
                "configuration loaded"
            );

            // TODO (Agent 2): wire up discovery + validation
            // TODO (Agent 3): wire up conversion + staging
            // TODO (Agent 4): wire up reporting
            tracing::warn!("pipeline-cli run not yet fully implemented");
        }

        Commands::Watch { config } => {
            let cfg = PipelineConfig::from_file(&config)?;
            cfg.validate()?;

            tracing_subscriber::fmt()
                .with_env_filter(&cfg.pipeline.log_level)
                .init();

            tracing::info!("pipeline watch mode starting");
            tracing::info!(
                watch_dir = %cfg.pipeline.watch_dir.display(),
                "watching for new assets"
            );

            // TODO (Agent 2): wire up file-system watcher (notify)
            tracing::warn!("watch mode not yet fully implemented");
        }
    }

    Ok(())
}
