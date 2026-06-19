//! # Infinity Blade IV Integration Tests
//!
//! The `ib4-integration-tests` crate provides centralized test scaffolding, mock environments,
//! and shared helpers to run high-level, end-to-end integration tests on the MUD gameplay simulation.
//!
//! ## Key Contents
//! - Exposes common setup utilities (e.g. `new_session`) to bootstrap a game session with default configurations.
//! - Integrates the underlying MUD command interpreter and core engine states to run test scripts.
//!
//! ## System Integration
//! This test module targets the entire MUD system by dispatching inputs through `ib4-mud` and validating state changes
//! across combat, progression, and narrative sub-crates.

// Shared helpers for integration tests
pub use ib4_core::types::AttackDir;
pub use ib4_mud::command::Command;
pub use ib4_mud::session::GameSession;

pub fn new_session() -> GameSession {
    GameSession::new("Siris")
}
