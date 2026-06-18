//! All network message types — strongly typed enums for every client/server exchange.
//!
//! Every message sent over the wire is represented here.  No raw JSON strings
//! should appear in game logic; everything routes through `MessageCodec`.

use nexus_types::MagicType;
use serde::{Deserialize, Serialize};

// ── Client → Server ──────────────────────────────────────────────────────────

/// Messages that the game client sends to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    // -- Authentication --
    Authenticate { player_id: u64, token: String },

    // -- Lobby --
    JoinLobby,
    LeaveLobby,
    ChallengeDuel { target_player_id: u64, suit_id: String },
    AcceptDuel { challenge_id: u64 },
    DeclineDuel { challenge_id: u64 },

    // -- In-Match (Duel Arena) --
    CombatAction { action: CombatAction },
    EmotePing { emote_id: u8 },
    SurrenderMatch,

    // -- Heartbeat --
    Ping { seq: u32 },
}

// ── Server → Client ──────────────────────────────────────────────────────────

/// Messages that the server broadcasts or unicasts to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // -- Auth response --
    AuthSuccess { session_id: u64 },
    AuthFailure { reason: String },

    // -- Lobby --
    LobbyJoined { online_count: u32 },
    DuelChallenge {
        challenge_id: u64,
        challenger_id: u64,
        challenger_name: String,
        suit: String,
    },
    DuelStarted {
        match_id: u64,
        opponent_id: u64,
        opponent_suit: String,
    },

    // -- Match state sync --
    MatchState { state: MatchStateSnapshot },
    CombatResult { action: CombatAction, outcome: CombatOutcome },
    MatchEnded { winner_id: u64, reason: MatchEndReason },

    // -- Server housekeeping --
    Pong { seq: u32, server_time_ms: u64 },
    Error { code: u16, message: String },
    Disconnect { reason: String },
}

// ── Shared payload types ──────────────────────────────────────────────────────

/// Snapshot of match state sent to both clients on every turn resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStateSnapshot {
    pub match_id: u64,
    pub player1_hp: f32,
    pub player2_hp: f32,
    pub player1_combo: u32,
    pub player2_combo: u32,
    pub turn_number: u32,
    pub is_player1_turn: bool,
}

/// A single combat decision made by one player on their turn.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CombatAction {
    Attack { dir: AttackDir },
    Parry { dir: Option<AttackDir> },
    Dodge,
    UseSpecial { ability_id: u8 },
    CastMagic { magic_type: MagicType },
}

/// The resolution result returned after a combat action is evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CombatOutcome {
    Hit { damage: f32, new_hp: f32, combo_depth: u32 },
    Blocked,
    PerfectParry { counter_damage: f32 },
    Dodged,
    PlayerDied { player_id: u64 },
}

/// Directional component of an attack or parry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackDir {
    Overhead,
    Left,
    Right,
}

/// Why a match concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchEndReason {
    Verified,
    Surrender,
    Timeout,
    Disconnect,
}
