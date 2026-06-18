//! # Infinity Blade IV Titan AI Subsystem
//!
//! The `ib4-ai` crate implements artificial intelligence behaviors for Titans and the GodKing boss.
//! It handles action decisions (announced/telegraphed vs. actual attack directions), bluff mechanics
//! for advanced phases, weapon throw cooldowns, and phase transitions as health pools deplete.
//!
//! ## Key Modules
//! - **`titan`**: Logic for standard Titans, determining telegraphed directions, bluffing probabilities, and phase changes.
//! - **`godking`**: Specific attack patterns, defensive shield logic, and unique phase transitions for the final boss, Corrupted Galath.
//! - **`roster`**: Directory and configurations of available Titans in the game world.
//!
//! ## System Integration
//! The AI subsystem receives state from the player and enemy instances defined in `ib4-core` and is driven
//! by the turn resolution step inside `ib4-combat` to update action telemetries.

pub mod roster;
pub mod titan;
pub mod godking;
