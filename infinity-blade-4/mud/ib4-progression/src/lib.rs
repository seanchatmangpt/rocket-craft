//! # Infinity Blade IV Progression Subsystem
//!
//! The `ib4-progression` crate governs player progression dynamics. It coordinates leveling mechanics,
//! experience curves, perks selections, and bloodline rebirth transitions.
//!
//! ## Key Modules
//! - **`xp`**: Models experience gains, leveling thresholds, and attribute point gains.
//! - **`bloodline`**: Controls the rebirth loop, tracking the cumulative bloodline generations and applying scaling modifiers.
//! - **`perks`**: Implements custom passive upgrades that can be unlocked via perk points.
//!
//! ## System Integration
//! Progression data structures are attached to the `Player` profile inside `ib4-core` and are manipulated
//! by both the game loop state and the combat rewards step when Titans are defeated.

pub mod bloodline;
pub mod perks;
pub mod xp;
