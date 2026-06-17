//! bpgen — Blueprint RS code generator
//!
//! Generates Unreal Engine 4 Blueprint T3D format from Rust definitions.
//!
//! USAGE:
//!     bpgen [OPTIONS] [COMMAND]
//!
//! COMMANDS:
//!     generate   Generate T3D from a Blueprint JSON definition
//!     inspect    Pretty-print a Blueprint JSON file
//!     example    Generate a built-in example Blueprint
//!     watch      Watch a directory for .json Blueprint files and regenerate on change
//!
//! OPTIONS:
//!     -o, --output <FILE>    Output file (default: stdout)
//!     -f, --format <FMT>     Output format: t3d (default) or json
//!     -v, --verbose          Verbose output

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use blueprint_core::ast::Blueprint;
use blueprint_core::builder::{BlueprintBuilder, VarType};
use blueprint_core::serializer::{JsonSerializer, T3dSerializer};

// ─────────────────────────────────────────────────────────────
// CLI definition
// ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "bpgen")]
#[command(about = "Generate Unreal Engine 4 Blueprint T3D from Rust definitions")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output file (default: stdout)
    #[arg(short, long, global = true)]
    output: Option<PathBuf>,

    /// Output format: t3d (default) or json
    #[arg(short, long, default_value = "t3d", global = true)]
    format: String,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate T3D from a Blueprint JSON definition file
    Generate {
        /// Input JSON file
        input: PathBuf,
    },
    /// Inspect and pretty-print a Blueprint JSON file
    Inspect {
        /// Input JSON file
        input: PathBuf,
    },
    /// Generate a built-in example Blueprint
    Example {
        /// Example name: hello, fps, ui, timer
        #[arg(default_value = "hello")]
        name: String,
        /// List available examples
        #[arg(short, long)]
        list: bool,
    },
    /// Watch a directory for .json Blueprint files and regenerate on change
    Watch {
        /// Directory to watch for .json Blueprint files
        dir: PathBuf,
        /// Output directory for generated .t3d files (default: same directory as input)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
    /// Decompile a .t3d file back to Blueprint Rust builder code
    Decompile {
        /// Input .t3d file
        input: PathBuf,
        /// Blueprint name to use in generated code
        #[arg(short = 'n', long, default_value = "MyBlueprint")]
        name: String,
        /// Parent class to use in generated code
        #[arg(short = 'p', long, default_value = "Actor")]
        parent: String,
    },
}

// ─────────────────────────────────────────────────────────────
// Built-in example Blueprints
// ─────────────────────────────────────────────────────────────

/// Simple Actor Blueprint: BeginPlay → PrintString("Hello from Blueprint-RS!")
fn example_hello() -> Blueprint {
    let mut b = BlueprintBuilder::new("HelloActor", "Actor");

    b.add_variable_mut("Greeting", VarType::String, Some("Hello from Blueprint-RS!".to_string()));

    let ev = b.begin_play_node();
    let ps = b.print_string("Hello from Blueprint-RS!");
    b.exec_connect(&ev, &ps);

    let ps2 = b.print_string("Blueprint transpilation works!");
    b.exec_connect(&ps, &ps2);

    b.build()
}

/// Character Blueprint with movement, camera, and death logic
fn example_fps() -> Blueprint {
    let mut b = BlueprintBuilder::new("FPSCharacter", "Character");

    // Variables
    b.add_variable_mut("Health", VarType::Int, Some("100".to_string()));
    b.add_variable_mut("MaxHealth", VarType::Int, Some("100".to_string()));
    b.add_variable_mut("WalkSpeed", VarType::Float, Some("600.0".to_string()));
    b.add_variable_mut("IsAlive", VarType::Bool, Some("true".to_string()));

    // BeginPlay: initialize at spawn location
    let begin_play = b.begin_play_node();
    let init_loc = b.set_actor_location(0.0, 0.0, 90.0);
    b.exec_connect(&begin_play, &init_loc);

    let init_print = b.print_string("FPS Character initialized!");
    b.exec_connect(&init_loc, &init_print);

    // Tick: add movement input (simplified)
    let tick = b.tick_node();
    let move_input = b.add_movement_input();
    b.exec_connect(&tick, &move_input);

    // Custom "OnDamageTaken" event with branch for death
    let damage_event = b.custom_event_node("OnDamageTaken");
    let damage_print = b.print_string("Player took damage!");
    b.exec_connect(&damage_event, &damage_print);

    let death_check = b.branch_node();
    b.exec_connect(&damage_print, &death_check);

    let death_print = b.print_string("Player died!");
    b.connect(&death_check, "then", &death_print, "execute");

    b.build()
}

/// UserWidget Blueprint with button click handler
fn example_ui() -> Blueprint {
    let mut b = BlueprintBuilder::new("MainMenuWidget", "UserWidget");

    // Variables
    b.add_variable_mut("MenuTitle", VarType::String, Some("Main Menu".to_string()));
    b.add_variable_mut("bIsMenuOpen", VarType::Bool, Some("false".to_string()));

    // Construct event (Widget equivalent of BeginPlay)
    let construct = b.begin_play_node();
    let welcome = b.print_string("Widget constructed - Main Menu ready!");
    b.exec_connect(&construct, &welcome);

    // Bind start button on construction
    let bind_start = b.bind_event_on_clicked("StartButton");
    b.exec_connect(&welcome, &bind_start);

    // Custom event: OnStartButtonClicked
    let on_start_click = b.custom_event_node("OnStartButtonClicked");
    let click_print = b.print_string("Start button clicked!");
    b.exec_connect(&on_start_click, &click_print);

    // Custom event: OnQuitButtonClicked
    let on_quit_click = b.custom_event_node("OnQuitButtonClicked");
    let quit_print = b.print_string("Quit button clicked - goodbye!");
    b.exec_connect(&on_quit_click, &quit_print);

    b.build()
}

/// Actor Blueprint using a timer and custom events
fn example_timer() -> Blueprint {
    let mut b = BlueprintBuilder::new("TimerActor", "Actor");

    // Variables
    b.add_variable_mut("TimerInterval", VarType::Float, Some("1.0".to_string()));
    b.add_variable_mut("TickCount", VarType::Int, Some("0".to_string()));

    // BeginPlay: set up a looping timer
    let begin_play = b.begin_play_node();
    let start_print = b.print_string("Timer Actor started - beginning countdown!");
    b.exec_connect(&begin_play, &start_print);

    let set_timer = b.set_timer_by_event(1.0, true);
    b.exec_connect(&start_print, &set_timer);

    // Custom event "TimerTick" — called every second by the timer
    let timer_tick = b.custom_event_node("TimerTick");
    let tick_print = b.print_string("Timer fired!");
    b.exec_connect(&timer_tick, &tick_print);

    // Add an integer counter increment
    let add_count = b.add_int();
    b.connect(&tick_print, "then", &add_count, "execute");

    b.build()
}

// ─────────────────────────────────────────────────────────────
// Output helpers
// ─────────────────────────────────────────────────────────────

fn write_output(content: &str, path: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    match path {
        Some(p) => {
            fs::write(p, content)?;
        }
        None => {
            std::io::stdout().write_all(content.as_bytes())?;
        }
    }
    Ok(())
}

fn format_blueprint(
    bp: &Blueprint,
    format: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match format.to_lowercase().as_str() {
        "json" => {
            let s = JsonSerializer::serialize(bp)?;
            Ok(s)
        }
        _ => Ok(T3dSerializer::serialize(bp)),
    }
}

// ─────────────────────────────────────────────────────────────
// Command implementations
// ─────────────────────────────────────────────────────────────

fn cmd_generate(
    input: &PathBuf,
    output: &Option<PathBuf>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("[bpgen] Reading input: {}", input.display());
    }

    let json = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    let bp: Blueprint = JsonSerializer::deserialize(&json)
        .map_err(|e| format!("Failed to parse Blueprint JSON: {}", e))?;

    if verbose {
        eprintln!(
            "[bpgen] Loaded Blueprint '{}' (parent: {}, graphs: {}, variables: {})",
            bp.name,
            bp.parent_class,
            bp.graphs.len(),
            bp.variables.len()
        );
    }

    let content = format_blueprint(&bp, format)?;

    if let Some(p) = output {
        if verbose {
            eprintln!("[bpgen] Writing {} output to {}", format, p.display());
        }
    }

    write_output(&content, output)?;
    Ok(())
}

fn cmd_inspect(input: &PathBuf, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("[bpgen] Inspecting: {}", input.display());
    }

    let json = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    let bp: Blueprint = JsonSerializer::deserialize(&json)
        .map_err(|e| format!("Failed to parse Blueprint JSON: {}", e))?;

    // Print summary info
    println!("Blueprint:    {}", bp.name);
    println!("Parent:       {}", bp.parent_class);
    println!("Graphs:       {}", bp.graphs.len());
    for g in &bp.graphs {
        println!("  - {} ({} nodes)", g.name, g.nodes.len());
        if verbose {
            for n in &g.nodes {
                println!("      node: {} [{}]", n.name, n.class.rsplit('.').next().unwrap_or(&n.class));
            }
        }
    }
    println!("Variables:    {}", bp.variables.len());
    for v in &bp.variables {
        let exposed = if v.is_exposed { " (exposed)" } else { "" };
        let default = v
            .default_value
            .as_deref()
            .map(|d| format!(" = {}", d))
            .unwrap_or_default();
        let category = v
            .category
            .as_deref()
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();
        println!(
            "  - {}: {:?}{}{}{}", v.name, v.var_type.category, default, category, exposed
        );
    }
    if !bp.interfaces.is_empty() {
        println!("Interfaces:");
        for iface in &bp.interfaces {
            println!("  - {}", iface);
        }
    }
    Ok(())
}

fn cmd_example(
    name: &str,
    list: bool,
    output: &Option<PathBuf>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if list {
        println!("Available built-in examples:");
        println!("  hello   — Actor Blueprint with BeginPlay → PrintString chain");
        println!("  fps     — Character Blueprint with movement, tick, and damage events");
        println!("  ui      — UserWidget Blueprint with button click handlers");
        println!("  timer   — Actor Blueprint using SetTimer and a looping custom event");
        return Ok(());
    }

    if verbose {
        eprintln!("[bpgen] Generating built-in example: {}", name);
    }

    let bp = match name.to_lowercase().as_str() {
        "hello" => example_hello(),
        "fps" => example_fps(),
        "ui" => example_ui(),
        "timer" => example_timer(),
        other => {
            return Err(format!(
                "Unknown example '{}'. Run `bpgen example --list` to see available examples.",
                other
            )
            .into())
        }
    };

    if verbose {
        eprintln!(
            "[bpgen] Built Blueprint '{}' ({} graphs, {} variables)",
            bp.name,
            bp.graphs.len(),
            bp.variables.len()
        );
    }

    let content = format_blueprint(&bp, format)?;

    if let Some(p) = output {
        if verbose {
            eprintln!("[bpgen] Writing to {}", p.display());
        }
    }

    write_output(&content, output)?;
    Ok(())
}

/// Process a single JSON Blueprint file and write the corresponding .t3d output.
///
/// Returns the path of the generated .t3d file on success, or an error string.
pub fn process_json_file(
    path: &std::path::Path,
    output_dir: Option<&std::path::Path>,
) -> Result<PathBuf, String> {
    let json = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;

    let bp: Blueprint = serde_json::from_str(&json)
        .map_err(|e| format!("{}", e))?;

    let t3d = T3dSerializer::serialize(&bp);

    let stem = path
        .file_stem()
        .ok_or_else(|| format!("'{}' has no file stem", path.display()))?;

    let out_path = match output_dir {
        Some(dir) => dir.join(stem).with_extension("t3d"),
        None => path.with_extension("t3d"),
    };

    fs::write(&out_path, &t3d)
        .map_err(|e| format!("Cannot write '{}': {}", out_path.display(), e))?;

    Ok(out_path)
}

fn cmd_decompile(
    input: &PathBuf,
    output: &Option<PathBuf>,
    name: &str,
    parent: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("[bpgen] Decompiling: {}", input.display());
    }
    let t3d = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;
    let nodes = blueprint_core::parser::parse_t3d(&t3d)
        .map_err(|e| format!("Failed to parse T3D: {}", e))?;
    if verbose {
        eprintln!("[bpgen] Parsed '{}' ({} nodes)", input.display(), nodes.len());
    }
    let code = blueprint_core::parser::generate_rust_code(&nodes, name, parent);
    write_output(&code, output)
}

fn cmd_watch(
    dir: &PathBuf,
    output_dir: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use notify::{EventKind, RecursiveMode, Watcher};

    println!(
        "[bpgen watch] Watching {} for .json Blueprint files...",
        dir.display()
    );

    let (tx, rx) = channel();

    let mut watcher = notify::RecommendedWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(dir, RecursiveMode::NonRecursive)?;

    for event in rx {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        match process_json_file(&path, output_dir.as_deref()) {
                            Ok(out) => {
                                println!(
                                    "[bpgen watch] Regenerated: {}",
                                    out.display()
                                );
                            }
                            Err(msg) => {
                                eprintln!(
                                    "[bpgen watch] Error in {}: {}",
                                    path.display(),
                                    msg
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Generate { input }) => {
            cmd_generate(input, &cli.output, &cli.format, cli.verbose)
        }
        Some(Commands::Inspect { input }) => cmd_inspect(input, cli.verbose),
        Some(Commands::Example { name, list }) => {
            cmd_example(name, *list, &cli.output, &cli.format, cli.verbose)
        }
        Some(Commands::Watch { dir, output_dir }) => cmd_watch(dir, output_dir),
        Some(Commands::Decompile { input, name, parent }) => {
            cmd_decompile(input, &cli.output, name, parent, cli.verbose)
        }
        // Default: no subcommand → print help
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
            println!();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Minimal valid Blueprint JSON for testing
    fn minimal_blueprint_json() -> &'static str {
        r#"{
            "name": "TestActor",
            "parent_class": "Actor",
            "graphs": [],
            "variables": [],
            "interfaces": []
        }"#
    }

    #[test]
    fn test_process_json_file_generates_t3d() {
        let tmp = std::env::temp_dir().join(format!(
            "bpgen_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let json_path = tmp.join("TestActor.json");
        fs::write(&json_path, minimal_blueprint_json()).expect("failed to write test JSON");

        let result = process_json_file(&json_path, None);
        assert!(result.is_ok(), "process_json_file failed: {:?}", result.err());

        let t3d_path = result.unwrap();
        assert_eq!(t3d_path, tmp.join("TestActor.t3d"));
        assert!(t3d_path.exists(), ".t3d file was not created");

        let content = fs::read_to_string(&t3d_path).expect("failed to read .t3d file");
        assert!(
            content.contains("Begin Object"),
            "T3D output missing expected content: {}", content
        );

        // Clean up
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_process_json_file_with_output_dir() {
        let tmp = std::env::temp_dir().join(format!(
            "bpgen_test_outdir_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let input_dir = tmp.join("input");
        let output_dir = tmp.join("output");
        fs::create_dir_all(&input_dir).expect("failed to create input dir");
        fs::create_dir_all(&output_dir).expect("failed to create output dir");

        let json_path = input_dir.join("TestActor.json");
        fs::write(&json_path, minimal_blueprint_json()).expect("failed to write test JSON");

        let result = process_json_file(&json_path, Some(&output_dir));
        assert!(result.is_ok(), "process_json_file failed: {:?}", result.err());

        let t3d_path = result.unwrap();
        assert_eq!(t3d_path, output_dir.join("TestActor.t3d"));
        assert!(t3d_path.exists(), ".t3d file was not created in output_dir");

        // Clean up
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_process_json_file_invalid_json_returns_error() {
        let tmp = std::env::temp_dir().join(format!(
            "bpgen_test_invalid_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let json_path = tmp.join("bad.json");
        fs::write(&json_path, "{ not valid json }").expect("failed to write bad JSON");

        let result = process_json_file(&json_path, None);
        assert!(result.is_err(), "expected error for invalid JSON");

        // Clean up
        let _ = fs::remove_dir_all(&tmp);
    }
}
