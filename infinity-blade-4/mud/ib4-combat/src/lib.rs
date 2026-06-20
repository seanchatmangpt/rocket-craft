//! # Infinity Blade IV Combat System
//!
//! The `ib4-combat` crate handles the resolution of physical and magical encounters between players and Titans.
//! It includes deep mechanics for defense timing, elemental magic, combo chains, and turn resolution.
//!
//! ## Key Modules
//! - **`parry`**: Resolves timing-based block and parry decisions (Overhead, Left, Right) to evaluate damage mitigation.
//! - **`combo`**: Tracks and calculates consecutive attack chains to determine damage multiplier scaling.
//! - **`damage`**: Implements basic and critical hit calculations, defense reduction, and elemental multiplier rules.
//! - **`magic`**: Manages spell actions, healing, and elemental status effects (such as burns or freezes).
//! - **`turn`**: The high-level turn scheduler coordinating player actions, enemy reactions, status tick downs, and phase shifts.
//!
//! ## System Integration
//! The combat engine sits between `ib4-core` (supplying entity stats) and `ib4-ai` (providing opponent decisions),
//! packaging results into standard `CombatEvent` structures consumed by `ib4-mud` for narrative output.

pub mod combo;
pub mod damage;
pub mod magic;
pub mod parry;
pub mod turn;
