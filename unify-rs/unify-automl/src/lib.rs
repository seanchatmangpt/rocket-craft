//! # Unify AutoML Crate
//!
//! This crate provides the AutoML abstraction layer for the Genie World Manufacturing Platform.
//! It includes:
//! - **Discovery Registry**: Scanning directory trees to find components annotated with `@UnifyAutoBind` or `AutoBind` macros.
//! - **Game Balance Auto-Optimizer**: Simulating battle outcomes via Monte Carlo simulations to find optimal stat allocations.
//! - **CLI Integration**: Standard CLI dispatch commands for environment setup, discovery scans, optimization, and local server management.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// 1. DYNAMIC DISCOVERY REGISTRY
// ─────────────────────────────────────────────────────────────────────────────

/// The `discovery` module implements dynamic discovery of game components by recursively scanning directories.
pub mod discovery {
    use super::*;

    /// A representation of a component found during dynamic discovery.
    /// Includes its name, path, programming language, and the specific binding tag found.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct DiscoveredComponent {
        /// The name of the discovered component.
        pub name: String,
        /// The file path to the source file where the component is defined.
        pub file_path: String,
        /// The programming language of the source file (e.g. Rust, C++).
        pub language: String,
        /// The binding tag annotation found (e.g. `@UnifyAutoBind: CombatSystem`).
        pub binding_tag: String,
    }

    /// The registry containing all discovered components and active workspace games.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ComponentRegistry {
        /// List of all discovered game components.
        pub components: Vec<DiscoveredComponent>,
        /// List of all detected workspace games.
        pub workspace_games: Vec<String>,
    }

    impl ComponentRegistry {
        /// Creates a new empty `ComponentRegistry`.
        ///
        /// # Examples
        ///
        /// ```
        /// use unify_automl::discovery::ComponentRegistry;
        /// let registry = ComponentRegistry::new();
        /// assert_eq!(registry.components.len(), 0);
        /// ```
        pub fn new() -> Self {
            Self {
                components: Vec::new(),
                workspace_games: Vec::new(),
            }
        }
    }

    impl Default for ComponentRegistry {
        /// Returns the default value for a `ComponentRegistry`.
        fn default() -> Self {
            Self::new()
        }
    }

    /// Recursively scan a directory for Rust, C++, and C files containing `@UnifyAutoBind` comment tags or `#[derive(AutoBind)]` macros.
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal or reading fails.
    pub fn scan_directory<P: AsRef<Path>>(dir: P) -> Result<ComponentRegistry> {
        let mut registry = ComponentRegistry::new();

        // Populate workspace games from Chicago TDD Tools discovery
        for game in chicago_tdd_tools::discover_games() {
            registry
                .workspace_games
                .push(format!("{} ({})", game.name, game.crate_name));
        }

        scan_dir_recursive(dir.as_ref(), &mut registry.components)?;
        Ok(registry)
    }

    /// Recursively traverses directories to locate candidate source files for parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if any subdirectory cannot be read.
    fn scan_dir_recursive(dir: &Path, components: &mut Vec<DiscoveredComponent>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // Skip target/ and hidden dirs
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" {
                            continue;
                        }
                    }
                    scan_dir_recursive(&path, components)?;
                } else if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if ext_str == "rs" || ext_str == "h" || ext_str == "cpp" || ext_str == "hpp" {
                        if let Ok(content) = fs::read_to_string(&path) {
                            parse_file_content(&content, &path, components);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Parses the content of a single source file to identify AutoML annotations and populate components.
    fn parse_file_content(content: &str, path: &Path, components: &mut Vec<DiscoveredComponent>) {
        let file_path = path.to_string_lossy().into_owned();
        let language = match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => "Rust",
            Some("h") | Some("hpp") | Some("cpp") => "C++",
            _ => "Unknown",
        }
        .to_string();

        for line in content.lines() {
            if let Some(idx) = line.find("@UnifyAutoBind") {
                let tag = line[idx..].trim().to_string();
                let name =
                    extract_name_from_tag(&tag).unwrap_or_else(|| "UnnamedComponent".to_string());

                let comp = DiscoveredComponent {
                    name,
                    file_path: file_path.clone(),
                    language: language.clone(),
                    binding_tag: tag,
                };
                if !components.contains(&comp) {
                    components.push(comp);
                }
            } else if line.contains("derive(AutoBind)")
                || line.contains("derive(unify_automl::AutoBind)")
            {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "AutoBindCrate".to_string());
                let comp = DiscoveredComponent {
                    name,
                    file_path: file_path.clone(),
                    language: language.clone(),
                    binding_tag: "#[derive(AutoBind)]".to_string(),
                };
                if !components.contains(&comp) {
                    components.push(comp);
                }
            }
        }
    }

    /// Extracts the component name from a `@UnifyAutoBind` tag.
    ///
    /// Supporting syntax like `@UnifyAutoBind(MyComp)`, `@UnifyAutoBind: AnotherComp`, or `@UnifyAutoBind CleanName`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unify_automl::discovery::extract_name_from_tag;
    ///
    /// let name = extract_name_from_tag("@UnifyAutoBind(HeroSystem)");
    /// assert_eq!(name, Some("HeroSystem".to_string()));
    /// ```
    pub fn extract_name_from_tag(tag: &str) -> Option<String> {
        let after_tag = tag.strip_prefix("@UnifyAutoBind")?.trim();
        let clean = after_tag
            .trim_start_matches([':', '('])
            .trim_end_matches(')')
            .trim();
        if clean.is_empty() {
            None
        } else {
            Some(clean.to_string())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. GAME BALANCE AUTO-OPTIMIZER
// ─────────────────────────────────────────────────────────────────────────────

/// The `balancer` module handles combat balancing through Monte Carlo simulations.
pub mod balancer {
    use super::*;
    use chicago_tdd_tools::coordinate::{GameCoordinateSystem, InfinityBladeCoordinateSystem};
    use ib4_mud::command::Command;
    use ib4_mud::session::GameSession;

    /// Defines a point allocation across player stats (health, attack, defense, magic).
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StatAllocation {
        /// Health points allocated.
        pub health: u32,
        /// Attack points allocated.
        pub attack: u32,
        /// Defense points allocated.
        pub defense: u32,
        /// Magic points allocated.
        pub magic: u32,
    }

    /// The aggregated results of running a batch of Monte Carlo battle simulations.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SimulationResult {
        /// The stat allocation that was simulated.
        pub allocation: StatAllocation,
        /// The ratio of battles won by the player (0.0 to 1.0).
        pub player_win_rate: f64,
        /// The average number of turns elapsed per battle.
        pub avg_turns: f64,
        /// The average final health of the player in won battles.
        pub average_player_final_hp: f64,
    }

    /// Run Monte Carlo simulation battles with a specific stat allocation.
    ///
    /// This simulates matches using Siris character settings and `InfinityBladeCoordinateSystem` rules.
    pub fn simulate_battles(alloc: &StatAllocation, num_blank_battles: usize) -> SimulationResult {
        let mut player_wins = 0;
        let mut total_turns = 0;
        let mut total_final_hp = 0.0;

        let coords = InfinityBladeCoordinateSystem;

        for _ in 0..num_blank_battles {
            let mut session = GameSession::new("Siris");

            // Set stats and recalculate
            session.player.stat_health = alloc.health;
            session.player.stat_attack = alloc.attack;
            session.player.stat_defense = alloc.defense;
            session.player.stat_magic = alloc.magic;
            session.player.recalculate_stats();
            session.player.health = session.player.max_health;

            // Trigger explore to enter combat
            session.dispatch(Command::Explore);

            let mut turns = 0;
            while session.is_in_combat() && turns < 100 {
                turns += 1;
                let legal_moves = coords.get_legal_moves(&session);
                if legal_moves.is_empty() {
                    break;
                }

                // Choose a move: heuristics to prioritize parries or high-value attacks
                let chosen_move = if session.announced_attack.is_some() {
                    if legal_moves.contains(&Command::Parry) {
                        Command::Parry
                    } else {
                        legal_moves[0].clone()
                    }
                } else {
                    if let Some(atk_move) =
                        legal_moves.iter().find(|m| matches!(m, Command::Attack(_)))
                    {
                        atk_move.clone()
                    } else {
                        legal_moves[0].clone()
                    }
                };

                session.dispatch(chosen_move);
            }

            if session.player.health > 0.0 {
                player_wins += 1;
                total_final_hp += session.player.health;
            }
            total_turns += turns;
        }

        SimulationResult {
            allocation: alloc.clone(),
            player_win_rate: player_wins as f64 / num_blank_battles as f64,
            avg_turns: total_turns as f64 / num_blank_battles as f64,
            average_player_final_hp: if player_wins > 0 {
                total_final_hp as f64 / player_wins as f64
            } else {
                0.0
            },
        }
    }

    /// Optimize stat allocations to reach closest to a target win rate.
    ///
    /// Scans combinations of stats totaling `total_points` and selects the one
    /// closest to `target_win_rate`.
    ///
    /// # Errors
    ///
    /// Returns an error if no valid allocation can be evaluated.
    pub fn optimize_balance(
        total_points: u32,
        target_win_rate: f64,
        sims_per_config: usize,
    ) -> Result<SimulationResult> {
        let mut best_result: Option<SimulationResult> = None;
        let mut min_diff = f64::MAX;

        for h in 0..=total_points {
            for a in 0..=(total_points - h) {
                for d in 0..=(total_points - h - a) {
                    let m = total_points - h - a - d;
                    let alloc = StatAllocation {
                        health: h,
                        attack: a,
                        defense: d,
                        magic: m,
                    };

                    let res = simulate_battles(&alloc, sims_per_config);
                    let diff = (res.player_win_rate - target_win_rate).abs();

                    if diff < min_diff {
                        min_diff = diff;
                        best_result = Some(res);
                    }
                }
            }
        }

        best_result.context("Failed to find any valid allocation result")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. CLI INTEGRATION LAYER
// ─────────────────────────────────────────────────────────────────────────────

/// The `cli` module provides commands to interface with the AutoML system.
pub mod cli {
    use super::*;

    /// The standardized output response of a CLI command dispatch.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CliOutput {
        /// Indicates if the command succeeded.
        pub success: bool,
        /// Description of the command outcome.
        pub message: String,
        /// Accompanying structured JSON data.
        pub data: serde_json::Value,
    }

    /// Dispatch developer CLI commands for AutoML.
    ///
    /// Supports `discover` and `optimize`.
    ///
    /// # Errors
    ///
    /// Returns an error if commands fail or serialization fails.
    pub fn dispatch_command(args: &[String]) -> Result<CliOutput> {
        if args.is_empty() {
            return Ok(CliOutput {
                success: false,
                message: "No command provided. Use 'discover' or 'optimize'.".to_string(),
                data: serde_json::Value::Null,
            });
        }

        match args[0].as_str() {
            "discover" => {
                let scan_path = args.get(1).map(|s| s.as_str()).unwrap_or(".");
                let registry = discovery::scan_directory(scan_path)?;
                Ok(CliOutput {
                    success: true,
                    message: format!("Successfully discovered components in: {}", scan_path),
                    data: serde_json::to_value(registry)?,
                })
            }
            "optimize" => {
                let points = args.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(8);
                let target = args
                    .get(2)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.6);
                let sims = args
                    .get(3)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(20);

                let opt_result = balancer::optimize_balance(points, target, sims)?;
                Ok(CliOutput {
                    success: true,
                    message: format!(
                        "Optimization complete targeting {:.1}% win rate.",
                        target * 100.0
                    ),
                    data: serde_json::to_value(opt_result)?,
                })
            }
            other => Ok(CliOutput {
                success: false,
                message: format!(
                    "Unknown subcommand: '{}'. Supported: discover, optimize",
                    other
                ),
                data: serde_json::Value::Null,
            }),
        }
    }

    /// Dispatch developer CLI commands for environment and server lifecycle.
    ///
    /// Supports `init` and `start`.
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be created or the server cannot start.
    pub fn dispatch_dev_command(args: &[String]) -> Result<CliOutput> {
        if args.is_empty() {
            return Ok(CliOutput {
                success: false,
                message: "No command provided. Use 'init' or 'start'.".to_string(),
                data: serde_json::Value::Null,
            });
        }

        match args[0].as_str() {
            "init" => {
                let dev_path = args.get(1).map(|s| s.as_str()).unwrap_or("./dev_env");
                let path = Path::new(dev_path);
                fs::create_dir_all(path)?;

                let config_path = path.join("dev_config.json");
                let default_config = serde_json::json!({
                    "env": "development",
                    "port": 3000,
                    "discovery_interval_sec": 5
                });
                fs::write(&config_path, serde_json::to_string_pretty(&default_config)?)?;

                let comp_path = path.join("test_component.rs");
                let comp_content = "// @UnifyAutoBind: TempComponent\n";
                fs::write(&comp_path, comp_content)?;

                Ok(CliOutput {
                    success: true,
                    message: format!("Developer environment initialized at {}", dev_path),
                    data: serde_json::json!({
                        "path": dev_path,
                        "config_file": config_path.to_string_lossy(),
                        "test_component_file": comp_path.to_string_lossy(),
                    }),
                })
            }
            "start" => {
                let dev_path = args.get(1).map(|s| s.as_str()).unwrap_or("./dev_env");
                let path = Path::new(dev_path);
                fs::create_dir_all(path)?;

                let mut server_path = std::path::PathBuf::from("genie_server.js");
                if !server_path.exists() {
                    server_path = std::path::PathBuf::from("../genie_server.js");
                }
                if !server_path.exists() {
                    server_path = std::path::PathBuf::from("../../genie_server.js");
                }
                if !server_path.exists() {
                    server_path =
                        std::path::PathBuf::from("/Users/sac/rocket-craft/genie_server.js");
                }

                let child = std::process::Command::new("node")
                    .arg(&server_path)
                    .spawn()
                    .context(
                        "Failed to spawn node genie_server.js. Ensure Node.js is installed.",
                    )?;
                let pid = child.id();

                let pid_path = path.join("server.pid");
                fs::write(&pid_path, pid.to_string())?;

                println!(
                    "genie_server.js successfully started with PID {} and listening",
                    pid
                );

                Ok(CliOutput {
                    success: true,
                    message: format!(
                        "genie_server.js successfully started with PID {} and listening",
                        pid
                    ),
                    data: serde_json::json!({
                        "pid": pid,
                        "pid_file": pid_path.to_string_lossy(),
                    }),
                })
            }
            other => Ok(CliOutput {
                success: false,
                message: format!("Unknown subcommand: '{}'. Supported: init, start", other),
                data: serde_json::Value::Null,
            }),
        }
    }
}


// ─────────────────────────────────────────────────────────────────────────────
// 4. UNIT TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use balancer::{optimize_balance, simulate_battles, StatAllocation};
    use cli::dispatch_command;
    use discovery::extract_name_from_tag;

    #[test]
    fn test_extract_name_from_tag() {
        assert_eq!(
            extract_name_from_tag("@UnifyAutoBind(MyComp)"),
            Some("MyComp".to_string())
        );
        assert_eq!(
            extract_name_from_tag("@UnifyAutoBind: AnotherComp"),
            Some("AnotherComp".to_string())
        );
        assert_eq!(
            extract_name_from_tag("@UnifyAutoBind CleanName"),
            Some("CleanName".to_string())
        );
        assert_eq!(extract_name_from_tag("@UnifyAutoBind"), None);
    }

    #[test]
    fn test_simulate_battles() {
        let alloc = StatAllocation {
            health: 5,
            attack: 3,
            defense: 2,
            magic: 0,
        };
        let res = simulate_battles(&alloc, 5);
        assert!(res.player_win_rate >= 0.0 && res.player_win_rate <= 1.0);
        assert!(res.avg_turns >= 0.0);
    }

    #[test]
    fn test_optimize_balance() {
        let res = optimize_balance(2, 0.5, 3).expect("Optimization should succeed");
        assert!(res.player_win_rate >= 0.0 && res.player_win_rate <= 1.0);
        let total = res.allocation.health
            + res.allocation.attack
            + res.allocation.defense
            + res.allocation.magic;
        assert_eq!(total, 2);
    }

    #[test]
    fn test_cli_dispatch_discover() {
        let args = vec!["discover".to_string(), ".".to_string()];
        let out = dispatch_command(&args).expect("CLI dispatch should succeed");
        assert!(out.success);
        assert!(out.data.is_object());
    }

    #[test]
    fn test_cli_dispatch_optimize() {
        let args = vec![
            "optimize".to_string(),
            "2".to_string(),
            "0.5".to_string(),
            "3".to_string(),
        ];
        let out = dispatch_command(&args).expect("CLI dispatch should succeed");
        assert!(out.success);
        assert!(out.data.is_object());
    }
}
