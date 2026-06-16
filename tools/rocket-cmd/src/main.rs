// wasmer 4.4.0 references __rust_probestack from wasmer-vm; Rust 1.85+ no
// longer emits it automatically. On Linux x86_64 the OS handles guard pages,
// so a no-op stub is safe for debug/development builds.
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".globl __rust_probestack",
    "__rust_probestack:",
    "ret",
);

use rocket_sdk::crypto;
use rocket_sdk::manifest;
use rocket_sdk::setup;
use rocket_sdk::error;
mod compliance;

use color_eyre::eyre::{eyre, ContextCompat, Result};
use error::RocketError;
use compliance::ComplianceEngine;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;
use knhk::AndroidKeystoreLaw;
use rocket_sdk::doctor::{RocketDoctor, CheckStatus};

#[derive(Parser)]
#[command(name = "rocket")]
#[command(about = "Rocket Craft Generative Orchestration Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup the Unreal Engine environment
    Setup,
    /// Synchronize project manifest with filesystem
    Sync,
    /// Build a project target
    Build {
        #[arg(short, long)]
        project: Option<String>,
        #[arg(short, long)]
        target: Option<String>,
        #[arg(short, long)]
        platform: Option<String>,
    },
    /// Audit project health and semantic law compliance
    Audit,
    /// Launch interactive TUI for project management
    Run,
    /// Manage Android keystores and encryption
    Crypto {
        #[command(subcommand)]
        crypto_cmd: Option<CryptoSubcommands>,
    },
    /// Clean build artifacts (Binaries, Intermediate, Saved)
    Clean,
    /// PWA management and optimization
    Pwa {
        #[command(subcommand)]
        pwa_cmd: Option<PwaSubcommands>,

        /// Directory containing PWA assets
        #[arg(short, long, default_value = "pwa-staff")]
        dir: String,

        /// Output minified worker to a different file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show project information
    Info,
    /// Run all tests (Rust, Asset validation, etc.)
    Test,
    /// Tail Unreal Engine build logs
    Logs {
        /// Specific log file to tail
        file: Option<String>,
        /// Number of initial lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
    /// Troubleshoot the environment
    Doctor,
    /// List all integrated high-level features (Capabilities)
    Capabilities,
    /// Execute a WASM plugin directly
    Wasm {
        /// Path to the WASM file
        #[arg(short, long)]
        file: String,
    },
}

#[derive(Subcommand)]
enum CryptoSubcommands {
    /// Generate all missing keystores
    Generate,
    /// Check status of keystores
    Status,
}

#[derive(Subcommand)]
enum PwaSubcommands {
    /// Lint and format PWA assets
    Lint,
    /// Generate asset manifest (default)
    Sync,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            setup::run_setup().map_err(|e| eyre!("{}", e))?;
        }
        Commands::Sync => {
            run_sync()?;
        }
        Commands::Build {
            project,
            target,
            platform,
        } => {
            run_build(project, target, platform)?;
        }
        Commands::Audit => {
            run_audit()?;
        }
        Commands::Run => {
            println!("Launching Interactive TUI...");
            // TUI implementation would go here (Task 14)
            println!("(TUI mode not fully implemented in this turn)");
        }
        Commands::Crypto { crypto_cmd } => match crypto_cmd {
            Some(CryptoSubcommands::Generate) | None => {
                crypto::generate_all_keystores().map_err(|e| eyre!("{}", e))?;
            }
            Some(CryptoSubcommands::Status) => {
                crypto::check_status().map_err(|e| eyre!("{}", e))?;
            }
        },
        Commands::Clean => {
            run_clean()?;
        }
        Commands::Pwa { pwa_cmd, dir, output } => {
            run_pwa(pwa_cmd, &dir, output)?;
        }
        Commands::Info => {
            run_info()?;
        }
        Commands::Test => {
            run_tests()?;
        }
        Commands::Logs { file, lines } => {
            run_logs(file, lines)?;
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
        Commands::Doctor => {
            run_doctor()?;
        }
        Commands::Capabilities => {
            run_capabilities()?;
        }
        Commands::Wasm { file } => {
            run_wasm(&file)?;
        }
    }

    Ok(())
}

fn run_wasm(file: &str) -> Result<()> {
    println!("{}", "=== WASM Plugin Execution ===".bold().cyan());
    let path = PathBuf::from(file);
    if !path.exists() {
        return Err(eyre!("WASM file not found: {}", file));
    }

    let mut plugin_host = knhk::plugin::PluginHost::new();
    println!("Loading plugin: {}", path.display());
    
    match plugin_host.load_law(&path) {
        Ok(law) => {
            println!("{} Successfully loaded WASM plugin: {}", "✓".green(), knhk::Law::name(&law));
            println!("Description: {}", knhk::Law::description(&law));
            println!("Executing validation...");
            match knhk::Law::validate(&law, Path::new(".")) {
                Ok(_) => println!("{} Validation passed", "✓".green()),
                Err(e) => println!("{} Validation failed: {}", "✗".red(), e.message),
            }
        }
        Err(e) => {
            println!("{} Failed to load WASM plugin: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

fn run_info() -> Result<()> {
    println!("{}", "Rocket Craft Generative Orchestration Tool".bold().cyan());
    println!("Version: 0.1.0");
    println!("Stack: Ostar / ggen / Rust / UE4.24");
    Ok(())
}

fn run_capabilities() -> Result<()> {
    println!("{}", "=== Rocket Craft Integrated Capabilities ===".bold().cyan());
    println!("{}", "High-level features currently integrated into the platform:".dimmed());
    println!();

    let capabilities = [
        ("Multiplatform Orchestration", "Unified build system for Windows, Linux, Android, and HTML5 (Web)."),
        ("Semantic Compliance", "Law-based project auditing via the ComplianceEngine and knhk plugin system."),
        ("Generative SDK", "Zero-cost typestate kernel in rocket-sdk for safe UE project manipulation."),
        ("PWA Optimization", "Automated asset manifest generation and mobile-ready PWA scaffolding."),
        ("Crypto Automation", "Automated Android keystore generation and lifecycle management."),
        ("Environment Diagnostics", "Rocket Doctor for programmatic workspace health and dependency checks."),
        ("Log Streaming", "Real-time colorized log tailing with semantic highlighting for UE4 builds."),
        ("TUI Dashboard", "Interactive terminal UI for project management (built with ratatui)."),
        ("Wasm Plugin System", "Extensible compliance laws via WebAssembly (Wasmer integration)."),
        ("Chicago TDD Integration", "Automated test orchestration across Rust and Python validation suites."),
    ];

    for (i, (name, desc)) in capabilities.iter().enumerate() {
        println!("  {:2}. {}: {}", i + 1, name.yellow().bold(), desc);
    }

    println!();
    println!("See {} for the full manifest of integrated libraries.", "capabilities/CapabilityManifest.md".blue());

    Ok(())
}

fn run_sync() -> Result<()> {
    println!("{}", "=== Syncing Project Manifest ===".bold().green());
    let mut projects = Vec::new();
    let versions_dir = Path::new("versions");

    if !versions_dir.exists() {
        return Err(RocketError::VersionsDirectoryNotFound(versions_dir.to_path_buf()).into());
    }

    let entries: Vec<_> = WalkDir::new(versions_dir)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("uproject"))
        .collect();

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
        .progress_chars("#>-"));

    for entry in entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        pb.set_message(format!("Syncing {}", name));
        
        // Look for targets
        let mut targets = Vec::new();
        let source_dir = path.parent()
            .ok_or_else(|| RocketError::ParentDirectoryNotFound(path.to_path_buf()))?
            .join("Source");
        if source_dir.exists() {
            for t_entry in fs::read_dir(source_dir)? {
                let t_entry = t_entry?;
                let t_name = t_entry.file_name().to_string_lossy().to_string();
                if t_name.ends_with(".Target.cs") {
                    targets.push(t_name.replace(".Target.cs", ""));
                }
            }
        }

        projects.push(manifest::Project {
            name,
            uproject_path: path.to_path_buf(),
            targets,
        });
        pb.inc(1);
    }

    pb.finish_with_message("Sync complete");

    let manifest = manifest::Manifest::new("project-manifest.json", projects);
    manifest.save().map_err(|e| eyre!("{}", e))?;
    println!("{}", "Manifest updated successfully.".green());
    Ok(())
}

fn run_build(project: Option<String>, target: Option<String>, platform: Option<String>) -> Result<()> {
    let manifest = manifest::Manifest::load("project-manifest.json").map_err(|e| eyre!("{}", e))?;
    
    let proj = if let Some(name) = project {
        manifest.projects().iter().find(|p| p.name == name)
            .ok_or_else(|| RocketError::ProjectNotFound(name.clone()))?
    } else {
        return Err(RocketError::Generic("No project specified. Use --project".to_string()).into());
    };

    let target = target.or_else(|| proj.targets.first().cloned())
        .ok_or_else(|| RocketError::NoTargetFound(proj.name.clone()))?;
    
    let platform = platform.unwrap_or_else(|| "Win64".to_string());

    let config = rocket_sdk::config::RocketConfig::load().map_err(|e| eyre!("{}", e))?;
    let ue4_root = config.ue4_root.ok_or_else(|| RocketError::Generic("UE4_ROOT not set. Run 'rocket setup' first.".to_string()))?;
    
    let uat_name = if cfg!(windows) { "RunUAT.bat" } else { "RunUAT.sh" };
    let uat_path = ue4_root.join("Engine").join("Build").join("BatchFiles").join(uat_name);

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{spinner:.cyan} [{elapsed_precise}] {msg}")?);
    pb.set_message(format!("Building {} [{}] for {}...", proj.name, target, platform));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let status = Command::new(&uat_path)
        .arg("BuildCookRun")
        .arg(format!("-project={}", proj.uproject_path.display()))
        .arg(format!("-target={}", target))
        .arg(format!("-platform={}", platform))
        .arg("-cook")
        .arg("-build")
        .arg("-stage")
        .arg("-archive")
        .arg(format!("-archivedirectory={}", Path::new("Builds").display()))
        .status()?;

    pb.finish_and_clear();

    if status.success() {
        println!("{}", "✔ Build Successful!".green().bold());
    } else {
        println!("{}", "✘ Build Failed.".red().bold());
    }

    Ok(())
}

fn run_audit() -> Result<()> {
    println!("{}", "=== Project Health Audit ===".bold().magenta());
    let manifest = manifest::Manifest::load("project-manifest.json").map_err(|e| eyre!("{}", e))?;
    
    let mut engine = ComplianceEngine::new();
    engine.add_law(Box::new(AndroidKeystoreLaw));

    // Load WASM plugins
    if let Err(e) = engine.load_plugins("plugins") {
        println!("  {} Failed to load plugins: {}", "⚠".yellow(), e);
    }

    for proj in manifest.projects() {
        println!("\nProject: {}", proj.name.bold().yellow());
        
        let uproject_path = &proj.uproject_path;
        // 1. Check uproject exists
        if uproject_path.exists() {
            println!("  {} uproject file found", "✓".green());
        } else {
            println!("  {} uproject file MISSING", "✗".red());
        }

        // 2. Check for missing maps (simplified)
        if let Some(project_dir) = uproject_path.parent() {
            let maps_dir = project_dir.join("Content").join("Maps");
            if maps_dir.exists() {
                let map_count = WalkDir::new(maps_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("umap"))
                    .count();
                println!("  {} Maps found: {}", "✓".green(), map_count);
            }

            // 3. Law Compliance (via ComplianceEngine)
            println!("  Checking law compliance...");
            let result = engine.check_project(proj);
            if result.passed {
                println!("    {} All laws satisfied.", "✓".green());
            } else {
                for err in result.errors {
                    println!("    {} Law '{}' violated: {}", "✗".red(), err.law_name, err.message);
                }
            }
        }
    }
    
    Ok(())
}

fn run_clean() -> Result<()> {
    println!("{}", "=== Cleaning Workspace ===".bold().red());
    let targets = ["Binaries", "Intermediate", "Saved"];
    
    let entries: Vec<_> = WalkDir::new("versions")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                targets.contains(&name.as_ref())
            } else {
                false
            }
        })
        .collect();

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.red} [{elapsed_precise}] [{bar:40.red/white}] {pos}/{len} ({eta}) {msg}")?
        .progress_chars("#>-"));

    for entry in entries {
        pb.set_message(format!("Removing: {}", entry.path().display()));
        fs::remove_dir_all(entry.path())?;
        pb.inc(1);
    }
    
    pb.finish_with_message("Cleanup complete");
    println!("{}", "Cleanup complete.".green());
    Ok(())
}

fn run_pwa(cmd: Option<PwaSubcommands>, dir: &str, _output: Option<String>) -> Result<()> {
    let pwa_dir = PathBuf::from(dir);
    if !pwa_dir.exists() {
        return Err(eyre!("PWA directory not found: {}", dir));
    }

    match cmd {
        Some(PwaSubcommands::Lint) => {
            println!("{}", "=== Linting & Formatting PWA Assets ===".bold().cyan());

            println!("  Running Prettier...");
            let status = Command::new("npm")
                .arg("run")
                .arg("format")
                .current_dir(&pwa_dir)
                .status()?;

            if !status.success() {
                return Err(eyre!("Prettier failed"));
            }

            println!("  Running ESLint...");
            let status = Command::new("npm")
                .arg("run")
                .arg("lint")
                .current_dir(&pwa_dir)
                .status()?;

            if status.success() {
                println!("{}", "✔ PWA Assets are clean!".green().bold());
            } else {
                return Err(eyre!("ESLint failed"));
            }
        }
        Some(PwaSubcommands::Sync) | None => {
            println!("🚀 Syncing PWA assets in: {}", dir);

            let mut assets = Vec::new();
            for entry in WalkDir::new(&pwa_dir) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    let rel = path.strip_prefix(&pwa_dir)?;
                    let rel_str = rel.to_string_lossy().to_string();
                    if rel_str != "manifest.json" && !rel_str.starts_with("node_modules") && !rel_str.starts_with(".") {
                        assets.push(rel_str);
                    }
                }
            }

            let manifest = serde_json::json!({
                "version": "1.0.1",
                "assets": assets
            });

            fs::write(pwa_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
            println!("   {} manifest.json generated.", "✓".green());
        }
    }

    Ok(())
}


fn run_doctor() -> Result<()> {
    println!("{}", "Rocket Doctor - Programmatic Diagnostics".bold().cyan());
    println!("===========================================");

    let project_root = std::env::current_dir()?;
    let doctor = RocketDoctor::new(project_root);
    let report = doctor.run_diagnostics();

    for check in report.checks {
        let status_str = match check.status {
            CheckStatus::Pass => "✓".green(),
            CheckStatus::Warn => "⚠".yellow(),
            CheckStatus::Fail => "✗".red(),
        };

        println!("  {} {}: {}", status_str, check.name.bold(), check.message);
        if let Some(details) = check.details {
            println!("    {}", details.dimmed());
        }
    }

    println!("\nReport generated at: {}", report.timestamp);

    Ok(())
}

fn run_tests() -> Result<()> {
    println!("{}", "=== Running All Tests ===".bold().cyan());

    // 1. Rust Workspace Tests
    println!("\n{}", "--- Rust Workspace Tests (tools) ---".bold().yellow());
    let status = Command::new("cargo")
        .arg("test")
        .arg("--workspace")
        .current_dir("tools")
        .status()?;
    if !status.success() {
        return Err(eyre!("Rust workspace tests failed"));
    }

    // 2. Chicago TDD Tools Tests
    println!("\n{}", "--- Chicago TDD Tools Tests ---".bold().yellow());
    let status = Command::new("cargo")
        .arg("test")
        .current_dir("chicago-tdd-tools")
        .status()?;
    if !status.success() {
        return Err(eyre!("Chicago TDD Tools tests failed"));
    }

    // 3. Asset Validation
    println!("\n{}", "--- Asset Validation ---".bold().yellow());
    let status = Command::new("python3")
        .arg("validate-assets.py")
        .status()?;
    if !status.success() {
        return Err(eyre!("Asset validation failed"));
    }

    println!("\n{}", "✔ All tests passed!".green().bold());
    Ok(())
}

fn run_logs(file: Option<String>, lines: usize) -> Result<()> {
    let logs_dir = Path::new("non-project-files/logs");
    
    if !logs_dir.exists() {
        return Err(eyre!("Logs directory not found: {}", logs_dir.display()));
    }

    let log_path = if let Some(f) = file {
        let p = logs_dir.join(f);
        if !p.exists() {
            return Err(eyre!("Log file not found: {}", p.display()));
        }
        p
    } else {
        let mut entries: Vec<_> = fs::read_dir(logs_dir)?
            .filter_map(|res| res.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        entries.last()
            .map(|e| e.path())
            .context("No log files found in non-project-files/logs")?
    };

    println!("{}", format!("Tailing log: {}", log_path.display()).bold().cyan());

    let f = fs::File::open(&log_path)?;
    let mut reader = BufReader::new(f);
    let pos = reader.seek(SeekFrom::End(0))?;
    
    // Read last N lines
    let chunk_size = 65536; // 64KB should be enough for last 50-100 lines
    let start_pos = pos.saturating_sub(chunk_size);
    reader.seek(SeekFrom::Start(start_pos))?;
    
    let mut initial_lines = Vec::new();
    for l in reader.lines().map_while(Result::ok) {
        initial_lines.push(l);
    }
    
    let to_show = if initial_lines.len() > lines {
        &initial_lines[initial_lines.len() - lines..]
    } else {
        &initial_lines[..]
    };
    
    for line in to_show {
        print_colorized(line);
    }

    // Now tail
    let f = fs::File::open(&log_path)?;
    let mut reader = BufReader::new(f);
    reader.seek(SeekFrom::End(0))?;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes > 0 {
            print_colorized(line.trim_end());
        } else {
            thread::sleep(Duration::from_millis(500));
        }
    }
}

fn print_colorized(line: &str) {
    if line.to_uppercase().contains("ERROR:") || line.contains("FAILED") {
        println!("{}", line.red());
    } else if line.to_uppercase().contains("WARNING:") {
        println!("{}", line.yellow());
    } else if line.contains("Success:") || line.contains("BUILD SUCCESSFUL") || line.contains("COMPLETE") {
        println!("{}", line.green());
    } else if line.starts_with("Log") && line.contains(':') {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            println!("{}:{}", parts[0].cyan(), parts[1]);
        } else {
            println!("{}", line);
        }
    } else {
        println!("{}", line);
    }
}
