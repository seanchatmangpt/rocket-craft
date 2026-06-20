//! # Nexus Entity-Component-System (ECS) Subsystem
//!
//! The `nexus-ecs` crate defines the core data model and system scheduling architecture for the Nexus engine.
//! Built on top of the lightweight and fast `hecs` crate, it decouples game state data from logic systems.
//!
//! ## Key Modules
//! - **`components`**: Defines plain old data (POD) structures representing attributes, positions, and tags of game objects.
//! - **`world`**: Provides a high-level wrapper (`GameWorld`) around the raw ECS world, providing methods for spawning players, enemies, and projectiles, as well as managing attachments.
//! - **`systems`**: Implements update logic functions (e.g., movement, projectile collision, stats ticks) that act on entity component queries.
//! - **`scheduler`**: Coordinates the sequence and parallel execution rules of systems per game frame tick.
//!
//! ## System Integration
//! The ECS serves as the central hub of state for the active simulation. Both `nexus-combat` (for combat states)
//! and `nexus-integration` (for the game loop) query and mutate components managed within this crate.

pub mod components;
pub mod scheduler;
pub mod systems;
pub mod world;
