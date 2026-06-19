//! # Nexus Session Subsystem
//!
//! The `nexus-session` crate handles the lifecycle of player connections, game session states,
//! player progression profiles, inventories, and NPC dialogue trees. It uses a type-safe typestate
//! state machine (`PlayerSession`) to enforce valid connection state transitions at compile time.
//!
//! ## Key Modules
//! - **`session`**: Contains connection state machine states (`Connecting`, `Authenticated`, `InLobby`, `InMatch`, `Spectating`, `Disconnected`).
//! - **`player`**: Defines player profiles, rankings, and pilot configurations (such as mobile suit preferences).
//! - **`inventory`**: Manages entity inventory items, including slot limits, equipping gear, and custom shop inventory definitions.
//! - **`npc`**: Governs NPC dialogues, branching dialogue trees, and local vendor state profiles.
//!
//! ## System Integration
//! This crate works closely with `nexus-net` to authenticate users and manage network sockets, mapping each
//! connected socket to a type-safe state in the `PlayerSession` machine before transitioning into gameplay.

pub mod inventory;
pub mod npc;
pub mod player;
pub mod session;

pub use inventory::{Inventory, NpcInventory, PlayerInventory, ShopInventory};
pub use npc::{Npc, NpcDialogueTree, NpcState};
pub use player::{GundamSeries, NewtypeRank, PlayerProfile};
pub use session::{
    PlayerSession, PlayerSessionBuilder, SessionBuildError, SessionState, SessionTransitionError,
};
