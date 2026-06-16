use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{error, info, warn};
use pipeline_core::{
    config::PipelineConfig,
    validation::{ValidationConfig, Validator},
    staging::Stager,
    reporting::{AssetStatus, Reporter},
    discovery::{Scanner, DirectoryWatcher},
    conversion::BlenderConverter,
    PipelineEvent,
};

#[derive(Parser)]
#[command(name = "asset-pipeline")]
#[command(version, about = "Autonomous 3D asset pipeline for Unreal Engine 4")]
struct Cli {
    /// Path to pipeline.toml config file
    #[arg(long, default_value = "pipeline.toml")]
    config: PathBuf,

    /// Path to blender_convert.py script (overrides auto-detection)
    #[arg(long)]
    script: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch a directory for new 3D model files (runs forever)
    Watch {
        /// Override watch directory from config
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Override output directory from config
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Scan a directory once and process all found files
    Once {
        /// Directory to scan (overrides config)
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Override output directory from config
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Convert a single file directly
    Convert {
        /// Input 3D model file
        #[arg(long)]
        input: PathBuf,
        /// Output directory (defaults to current dir)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Print an example pipeline.toml to stdout
    InitConfig,
    /// Show pipeline status (last run summary from manifest)
    Status {
        /// Output directory containing pipeline-manifest.json
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config (may not exist for InitConfig/Status)
    let config = match &cli.command {
        Commands::InitConfig | Commands::Status { .. } => None,
        _ => Some(PipelineConfig::from_file(&cli.config)
            .with_context(|| format!("loading config from {}", cli.config.display()))?),
    };

    // Init tracing
    let log_level = config.as_ref().map(|c| c.pipeline.log_level.as_str()).unwrap_or("info");
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    let script = cli.script;

    match cli.command {
        Commands::InitConfig => {
            println!("{}", PipelineConfig::example_toml());
        }

        Commands::Status { output } => {
            let out_dir = output.unwrap_or_else(|| PathBuf::from("."));
            let reporter = Reporter::new(&out_dir);
            match reporter.load_or_create() {
                Ok(manifest) => {
                    if manifest.runs.is_empty() {
                        println!("No pipeline runs recorded in {}", out_dir.display());
                    } else {
                        let last = manifest.runs.last().unwrap();
                        println!("Last run: {} runs total", manifest.runs.len());
                        Reporter::print_summary(last);
                    }
                }
                Err(e) => eprintln!("Could not read manifest: {e}"),
            }
        }

        Commands::Once { dir, output } => {
            let cfg = config.unwrap();
            let watch_dir = dir.unwrap_or(cfg.pipeline.watch_dir.clone());
            let output_dir = output.unwrap_or(cfg.pipeline.output_dir.clone());
            run_once(&cfg, watch_dir, output_dir, script).await?;
        }

        Commands::Convert { input, output } => {
            let cfg = config.unwrap();
            let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
            run_single_conversion(&cfg, input, output_dir, script).await?;
        }

        Commands::Watch { dir, output } => {
            let cfg = config.unwrap();
            let watch_dir = dir.unwrap_or(cfg.pipeline.watch_dir.clone());
            let output_dir = output.unwrap_or(cfg.pipeline.output_dir.clone());
            info!("Watching {} for new 3D model files...", watch_dir.display());
            run_watch_mode(&cfg, watch_dir, output_dir, script).await?;
        }
    }

    Ok(())
}

fn resolve_script(override_path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p;
    }
    // Try relative to CARGO_MANIFEST_DIR at compile time
    let compile_time = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("scripts/blender_convert.py"))
        .unwrap_or_else(|| PathBuf::from("scripts/blender_convert.py"));

    if compile_time.exists() {
        return compile_time;
    }

    // Fallback: look in CWD
    PathBuf::from("asset-pipeline/scripts/blender_convert.py")
}

async fn run_once(cfg: &PipelineConfig, watch_dir: PathBuf, output_dir: PathBuf, script: Option<PathBuf>) -> Result<()> {
    info!("Scanning {}", watch_dir.display());

    let scan = Scanner::scan_once(&watch_dir);
    info!("Found {} assets, {} errors", scan.assets.len(), scan.errors.len());
    for (path, err) in &scan.errors {
        warn!("Scan error for {}: {err}", path.display());
    }

    process_assets(cfg, scan.assets, watch_dir, output_dir, script).await
}

async fn run_single_conversion(cfg: &PipelineConfig, input: PathBuf, output_dir: PathBuf, script: Option<PathBuf>) -> Result<()> {
    use pipeline_core::discovery;
    let hash = discovery::content_hash(&input)
        .context("hashing input file")?;
    let meta = std::fs::metadata(&input).context("reading metadata")?;
    let fmt = pipeline_core::Format::from_extension(
        input.extension().and_then(|e| e.to_str()).unwrap_or("")
    ).ok_or_else(|| anyhow::anyhow!("unsupported format"))?;

    let asset = pipeline_core::DiscoveredAsset::new(input, hash, fmt, meta.len());
    process_assets(cfg, vec![asset], PathBuf::from("."), output_dir, script).await
}

async fn run_watch_mode(cfg: &PipelineConfig, watch_dir: PathBuf, output_dir: PathBuf, script: Option<PathBuf>) -> Result<()> {
    // Broadcast channel for pipeline events
    let (event_tx, mut event_rx) = tokio::sync::broadcast::channel::<PipelineEvent>(128);

    // Start watcher in background task
    let watcher = DirectoryWatcher::new(watch_dir.clone(), event_tx);
    let _watcher_handle = tokio::spawn(async move {
        if let Err(e) = watcher.run().await {
            error!("Watcher error: {e}");
        }
    });

    // Process events as they arrive
    loop {
        match event_rx.recv().await {
            Ok(PipelineEvent::FileDiscovered { path }) => {
                info!("New file detected: {}", path.display());
                let hash = match pipeline_core::discovery::content_hash(&path) {
                    Ok(h) => h,
                    Err(e) => { error!("Hash error for {}: {e}", path.display()); continue; }
                };
                let meta = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(e) => { error!("Metadata error: {e}"); continue; }
                };
                let fmt = match pipeline_core::Format::from_extension(
                    path.extension().and_then(|e| e.to_str()).unwrap_or("")
                ) {
                    Some(f) => f,
                    None => { warn!("Unknown format for {}", path.display()); continue; }
                };
                let asset = pipeline_core::DiscoveredAsset::new(path, hash, fmt, meta.len());
                if let Err(e) = process_assets(cfg, vec![asset], watch_dir.clone(), output_dir.clone(), script.clone()).await {
                    error!("Pipeline error: {e}");
                }
            }
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                info!("Watcher closed, exiting");
                break;
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!("Dropped {n} events due to slow processing");
            }
        }
    }
    Ok(())
}

async fn process_assets(
    cfg: &PipelineConfig,
    assets: Vec<pipeline_core::DiscoveredAsset>,
    watch_dir: PathBuf,
    output_dir: PathBuf,
    script: Option<PathBuf>,
) -> Result<()> {
    std::fs::create_dir_all(&output_dir)?;

    let work_dir = output_dir.join(".pipeline-work");
    std::fs::create_dir_all(&work_dir)?;

    let script_path = resolve_script(script);

    let converter = BlenderConverter::discover(script_path)
        .context("finding Blender installation")?;
    let val_config = ValidationConfig::from_pipeline_config(&cfg.pipeline);
    let mut validator = Validator::new(val_config);
    let stager = Stager::new(output_dir.clone());
    let reporter = Reporter::new(&output_dir);

    let mut manifest = reporter.load_or_create()?;
    let mut run = Reporter::begin_run(watch_dir, output_dir.clone());

    for asset in assets {
        let start = Instant::now();
        let path = asset.path.clone();
        let hash = asset.hash;

        // Validate
        let validated = match validator.validate(asset) {
            Ok(v) => v,
            Err((e, _)) => {
                let status = if e.is_skippable() { AssetStatus::Skipped } else { AssetStatus::ValidationFailed };
                warn!("Skipping {}: {e}", path.display());
                Reporter::record_failure(&mut run, path, hash, status, &e, start.elapsed().as_millis() as u64);
                continue;
            }
        };

        // Convert
        let converted = match converter.convert(validated, &work_dir).await {
            Ok(c) => c,
            Err((e, _)) => {
                error!("Conversion failed for {}: {e}", path.display());
                Reporter::record_failure(&mut run, path, hash, AssetStatus::ConversionFailed, &e, start.elapsed().as_millis() as u64);
                continue;
            }
        };

        // Stage
        match stager.stage(converted) {
            Ok(staged) => {
                info!("\u{2713} {}", staged.content_path.display());
                Reporter::record_success(&mut run, &staged, start.elapsed().as_millis() as u64);
            }
            Err((e, _)) => {
                error!("Staging failed: {e}");
                Reporter::record_failure(&mut run, path, hash, AssetStatus::StagingFailed, &e, start.elapsed().as_millis() as u64);
            }
        }
    }

    reporter.finish_run(&mut manifest, run)?;

    if let Some(last) = manifest.runs.last() {
        Reporter::print_summary(last);
    }

    Ok(())
}
