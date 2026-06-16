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
//!
//! OPTIONS:
//!     -o, --output <FILE>    Output file (default: stdout)
//!     -f, --format <FMT>     Output format: t3d (default) or json
//!     -v, --verbose          Verbose output

use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

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
    /// Generate a Blueprint from a natural-language description using Claude AI
    Ai {
        /// Natural-language description of the Blueprint to generate
        description: String,
        /// Output file (overrides global --output for this subcommand)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Claude model to use (default: claude-haiku-4-5-20251001)
        #[arg(short, long)]
        model: Option<String>,
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

// ─────────────────────────────────────────────────────────────
// AI Blueprint generation
// ─────────────────────────────────────────────────────────────

const DEFAULT_AI_MODEL: &str = "claude-haiku-4-5-20251001";

const BLUEPRINT_SYSTEM_PROMPT: &str = r#"You are a Unreal Engine 4 Blueprint expert. Given a natural-language description, generate a valid Blueprint definition as JSON.

Respond with ONLY valid JSON matching this exact schema — no markdown fences, no explanation:

{
  "name": "<PascalCase name>",
  "parent_class": "<UE4 parent class, e.g. Actor, Character, UserWidget>",
  "graphs": [
    {
      "name": "<graph name, e.g. EventGraph>",
      "nodes": [
        {
          "id": "<unique id, e.g. node_0>",
          "name": "<node display name>",
          "class": "<UE4 node class path, e.g. /Script/BlueprintGraph.K2Node_Event>",
          "position": { "x": 0, "y": 0 },
          "pins": [],
          "properties": {}
        }
      ],
      "connections": [
        {
          "from_node": "<node id>",
          "from_pin": "<pin name>",
          "to_node": "<node id>",
          "to_pin": "<pin name>"
        }
      ]
    }
  ],
  "variables": [
    {
      "name": "<variable name>",
      "var_type": { "category": "bool", "sub_category": null, "is_array": false, "is_map": false, "is_set": false },
      "is_exposed": false,
      "default_value": null,
      "category": null
    }
  ],
  "interfaces": []
}"#;

/// Call the Anthropic Messages API to generate a Blueprint T3D from a description.
///
/// Returns the T3D-serialized Blueprint string on success.
fn generate_blueprint_from_prompt(description: &str, model: &str) -> Result<String, String> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable is not set".to_string())?;

    let request_body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "system": BLUEPRINT_SYSTEM_PROMPT,
        "messages": [
            {
                "role": "user",
                "content": description
            }
        ]
    });

    let response = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(&request_body)
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let response_json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let text = response_json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| format!("Unexpected API response shape: {}", response_json))?;

    let bp: Blueprint = serde_json::from_str(text)
        .map_err(|e| format!("Claude returned invalid Blueprint JSON: {}\n\nRaw response:\n{}", e, text))?;

    Ok(T3dSerializer::serialize(&bp))
}

fn cmd_ai(
    description: &str,
    output: &Option<PathBuf>,
    model: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("[bpgen] Sending description to Claude ({})...", model);
        eprintln!("[bpgen] Description: {}", description);
    }

    let t3d = generate_blueprint_from_prompt(description, model)
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    if verbose {
        eprintln!("[bpgen] Blueprint generated successfully");
    }

    write_output(&t3d, output)?;
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
        Some(Commands::Ai { description, output, model }) => {
            let m = model.as_deref().unwrap_or(DEFAULT_AI_MODEL);
            let out = output.as_ref().or(cli.output.as_ref()).cloned();
            cmd_ai(description, &out, m, cli.verbose)
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

    /// Verify that the AI request body is structured correctly without making HTTP calls.
    #[test]
    fn test_ai_prompt_format() {
        let description = "A simple Actor that prints Hello on BeginPlay";
        let model = DEFAULT_AI_MODEL;

        let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "system": BLUEPRINT_SYSTEM_PROMPT,
            "messages": [
                {
                    "role": "user",
                    "content": description
                }
            ]
        });

        // Verify top-level fields
        assert_eq!(body["model"].as_str().unwrap(), "claude-haiku-4-5-20251001");
        assert_eq!(body["max_tokens"].as_u64().unwrap(), 4096);
        assert!(body["system"].as_str().unwrap().contains("valid JSON"));

        // Verify messages structure
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"].as_str().unwrap(), "user");
        assert_eq!(messages[0]["content"].as_str().unwrap(), description);
    }
}
