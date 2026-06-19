//! nexus-net — Multiplayer networking for Gundam Nexus.
//!
//! Provides typestate WebSocket connections, duel matchmaking, game room state,
//! message serialization via serde_json, and async tokio-based infrastructure.

pub mod connection;
pub mod matchmaking;
pub mod message_codec;
pub mod protocol;
pub mod room;

pub use connection::{
    Authenticated, Connected, Connection, ConnectionBuilder, ConnectionState,
    ConnectionTransitionError, Disconnected, Handshaking, InLobby, InMatch,
};
