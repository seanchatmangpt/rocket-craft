//! # Infinity Blade IV MUD Library
//!
//! The `ib4-mud` crate serves as the text-based user interface (REPL) and game coordinator for
//! the Infinity Blade IV MUD simulation. It parses user console input commands and formats
//! underlying combat, progression, and narrative event outputs into rich, interactive narrative descriptions.
//!
//! ## Key Modules
//! - **`command`**: Parser that maps textual console commands (e.g. `explore`, `attack overhead`, `shop buy`) to structured gameplay commands.
//! - **`narrative`**: Formats raw combat events and progression state changes into immersive text responses.
//! - **`session`**: Manages the overarching game session lifecycle, handling game initialization, command dispatching, and serializing/deserializing player progress.
//! - **`repl`**: Implements the main Read-Eval-Print-Loop (REPL) CLI interface using stdin and standard output logging.
//!
//! ## System Integration
//! This crate integrates all subsystems of the Infinity Blade IV MUD (combat, AI, progression, and core types),
//! providing a unified entrypoint for both the command-line interface executable (`main.rs`) and test suites.

pub mod command;
pub mod narrative;
pub mod session;
pub mod repl;
pub mod telemetry;

