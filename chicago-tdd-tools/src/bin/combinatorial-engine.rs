use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use chicago_tdd_tools::{
    coordinate::{
        GundamCoordinateSystem, GundamSessionSimulation, InfinityBladeCoordinateSystem,
        SessionState,
    },
    discover_games, explore_state_space,
};
use ib4_mud::session::GameSession;
use nexus_session::player::PlayerProfile;

#[derive(Parser, Debug)]
#[command(
    name = "combinatorial-engine",
    about = "Rocket-Craft Combinatorial State Space Exploration Engine"
)]
struct Cli {
    /// Output path for the JSON report
    #[arg(short, long, default_value = "combinatorial_report.json")]
    output: PathBuf,
}

#[derive(serde::Serialize)]
struct ReportTransition {
    game: String,
    source: String,
    #[serde(rename = "move")]
    mv: String,
    target: String,
}

#[derive(serde::Serialize)]
struct GameSummary {
    visited_states_count: usize,
    transition_count: usize,
    errors: Vec<String>,
}

#[derive(serde::Serialize)]
struct CombinatorialReport {
    total_states_visited: usize,
    transition_count: usize,
    transitions: Vec<ReportTransition>,
    games: HashMap<String, GameSummary>,
}

fn run_engine(cli: Cli) -> Result<()> {
    // 1. Discover games
    println!("Executing discover_games()...");
    let discovered = discover_games();
    let mut found_ib4 = false;
    let mut found_gundam = false;

    for game in &discovered {
        println!(
            "  Discovered game: {} (crate: {})",
            game.name, game.crate_name
        );
        if game.name == "Infinity Blade 4 MUD" {
            found_ib4 = true;
        }
        if game.name == "Gundam Nexus" {
            found_gundam = true;
        }
    }

    if !found_ib4 || !found_gundam {
        anyhow::bail!(
            "Could not discover both 'Infinity Blade 4 MUD' and 'Gundam Nexus'. Discovered: {:?}",
            discovered
        );
    }

    // 2. Instantiate initial states and systems
    let ib4_sys = InfinityBladeCoordinateSystem;
    let ib4_state = GameSession::new("SirisAimbot");

    let gundam_sys = GundamCoordinateSystem;
    let gundam_state = GundamSessionSimulation {
        state: SessionState::Connecting,
        profile: PlayerProfile::new(1001, "PilotAimbot".to_string()),
        inventory: Vec::new(),
    };

    // 3. Explore state space
    println!("Exploring Infinity Blade 4 MUD state space...");
    let ib4_result = explore_state_space(&ib4_sys, ib4_state, 1000);

    println!("Exploring Gundam Nexus state space...");
    let gundam_result = explore_state_space(&gundam_sys, gundam_state, 1000);

    // 4. Print clean human-readable output to standard output
    println!("\n========================================");
    println!("EXPLORATION RESULTS");
    println!("========================================");

    println!("\nGame: Infinity Blade 4 MUD");
    println!("Visited States Count: {}", ib4_result.visited_states_count);
    println!("Transitions Count: {}", ib4_result.transitions.len());
    println!("Transitions:");
    for (src, mv, dst) in &ib4_result.transitions {
        println!("  {} --({})--> {}", src, mv, dst);
    }
    if !ib4_result.errors.is_empty() {
        println!("Errors:");
        for err in &ib4_result.errors {
            println!("  [ERR] {}", err);
        }
    }

    println!("\nGame: Gundam Nexus");
    println!(
        "Visited States Count: {}",
        gundam_result.visited_states_count
    );
    println!("Transitions Count: {}", gundam_result.transitions.len());
    println!("Transitions:");
    for (src, mv, dst) in &gundam_result.transitions {
        println!("  {} --({})--> {}", src, mv, dst);
    }
    if !gundam_result.errors.is_empty() {
        println!("Errors:");
        for err in &gundam_result.errors {
            println!("  [ERR] {}", err);
        }
    }
    println!("========================================\n");

    // 5. Generate JSON report
    let mut transitions = Vec::new();
    for (src, mv, dst) in &ib4_result.transitions {
        transitions.push(ReportTransition {
            game: "Infinity Blade 4 MUD".to_string(),
            source: src.clone(),
            mv: mv.clone(),
            target: dst.clone(),
        });
    }
    for (src, mv, dst) in &gundam_result.transitions {
        transitions.push(ReportTransition {
            game: "Gundam Nexus".to_string(),
            source: src.clone(),
            mv: mv.clone(),
            target: dst.clone(),
        });
    }

    let mut games = HashMap::new();
    games.insert(
        "Infinity Blade 4 MUD".to_string(),
        GameSummary {
            visited_states_count: ib4_result.visited_states_count,
            transition_count: ib4_result.transitions.len(),
            errors: ib4_result.errors,
        },
    );
    games.insert(
        "Gundam Nexus".to_string(),
        GameSummary {
            visited_states_count: gundam_result.visited_states_count,
            transition_count: gundam_result.transitions.len(),
            errors: gundam_result.errors,
        },
    );

    let report = CombinatorialReport {
        total_states_visited: ib4_result.visited_states_count + gundam_result.visited_states_count,
        transition_count: transitions.len(),
        transitions,
        games,
    };

    let report_json =
        serde_json::to_string_pretty(&report).context("Failed to serialize report JSON")?;
    let mut file = File::create(&cli.output)
        .with_context(|| format!("Failed to create report file at {:?}", cli.output))?;
    file.write_all(report_json.as_bytes())
        .context("Failed to write report content to file")?;

    println!("Report successfully generated at: {:?}", cli.output);

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run_engine(cli) {
        eprintln!("Unhandled error occurred during traversal: {:?}", e);
        std::process::exit(1);
    }
}
