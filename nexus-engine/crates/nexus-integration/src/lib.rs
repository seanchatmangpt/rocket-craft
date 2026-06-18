//! # Nexus Integration Subsystem
//!
//! The `nexus-integration` crate orchestrates the central gameplay loop and acts as the bridge connecting
//! diverse systems (e.g., combat, progression, shopping, rebirth mechanics) into a coherent, deterministic,
//! turn-based game session (`GameSession`).
//!
//! ## Key Modules
//! - **`game_loop`**: Contains the full gameplay loop state machine, defining command dispatching (`GameCommand`), player/enemy lifecycle management, stat allocation, and item purchasing.
//!
//! ## System Integration
//! This crate serves as the unified interface for external callers (such as test suites, MUD interfaces,
//! and server endpoints) to execute deterministic gameplay commands. It pulls in domain primitives from
//! `nexus-types` and implements simplified equivalents of modules in `nexus-combat` and `nexus-shop`
//! to provide a self-contained, easily testable simulation.

pub mod game_loop;
