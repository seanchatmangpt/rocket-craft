//! # Chicago TDD Tools Crate
//!
//! This crate provides core testing, simulation, coordinate state space tracking, and logger tools for Chicago-style TDD workflows.
//!
//! Submodules include:
//! - `domain`: Core domain models (accounts, transfer services, environment).
//! - `cli`: Command line integration interfaces.
//! - `logging`: Multiple-sink logging facilities.
//! - `discovery`: Dynamic session verification and suite detection.
//! - `coordinate`: Formal state space coordinate definitions.
//! - `aimbot`: State space traversal and exploration algorithms.

pub mod aimbot;
pub mod cli;
pub mod coordinate;
pub mod discovery;
pub mod domain;
pub mod logging;

pub use aimbot::{TraversalResult, explore_state_space};
pub use cli::ClapNoun;
pub use coordinate::{
    GameCoordinateSystem, GundamCoordinateSystem, GundamMove, GundamSessionSimulation,
    InfinityBladeCoordinateSystem, SessionState,
};
pub use discovery::{DiscoveredGame, discover_games};
pub use domain::account::Account;
pub use domain::environment::TestEnvironment;
pub use domain::transfer::TransferService;

pub use logging::{FileSink, LogLevel, LogSink, Logger, StdoutSink, TuiBufferSink};
