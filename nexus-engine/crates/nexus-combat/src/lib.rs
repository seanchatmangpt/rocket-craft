//! # Nexus Combat Subsystem
//!
//! The `nexus-combat` crate implements the core combat mechanics for the Nexus engine.
//! It utilizes a compile-time-verified typestate state machine (`CombatMachine`) to govern
//! combat status transitions, ensuring that invalid sequences—such as attacking while dodging
//! or resolving a parry that was never started—are mathematically unrepresentable.
//!
//! ## Key Modules
//! - **`combo`**: Manages attack combos, chain depths, and special combat sequences (e.g., standard vs. Trans-Am combos).
//! - **`damage`**: Calculates hit damage, defense mitigation, and tracks special debuffs such as Qip Scars.
//! - **`events`**: Defines combat-related events that drive telemetry, visuals, and UI updates.
//! - **`machine`**: The core typestate machine defining valid states (`Idle`, `Attacking`, `Parrying`, `PerfectParrying`, `Dodging`) and their transition functions.
//! - **`parry`**: Resolves timed defenses, calculating chip damage, blocks, and perfect parries.
//!
//! ## System Integration
//! This module coordinates with the `nexus-ecs` crate to update components representing entity health, combos,
//! and status effects, and integrates with `nexus-integration` to drive the main game loop turn updates.

pub mod combo;
pub mod damage;
pub mod events;
pub mod machine;
pub mod parry;

pub use combo::{ComboChain, StandardCombo, TransAmCombo};
pub use damage::{calculate_damage, QipScarTracker};
pub use events::CombatEvent;
pub use machine::CombatMachine;
pub use parry::ParryResolver;
