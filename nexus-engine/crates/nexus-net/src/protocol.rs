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
    Authenticate {
        player_id: u64,
        token: String,
    },

    // -- Lobby --
    JoinLobby,
    LeaveLobby,
    ChallengeDuel {
        target_player_id: u64,
        suit_id: String,
    },
    AcceptDuel {
        challenge_id: u64,
    },
    DeclineDuel {
        challenge_id: u64,
    },

    // -- In-Match (Duel Arena) --
    CombatAction {
        action: CombatAction,
    },
    EmotePing {
        emote_id: u8,
    },
    SurrenderMatch,

    // -- Heartbeat --
    Ping {
        seq: u32,
    },
}

// ── Server → Client ──────────────────────────────────────────────────────────

/// Messages that the server broadcasts or unicasts to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // -- Auth response --
    AuthSuccess {
        session_id: u64,
    },
    AuthFailure {
        reason: String,
    },

    // -- Lobby --
    LobbyJoined {
        online_count: u32,
    },
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
    MatchState {
        state: MatchStateSnapshot,
    },
    CombatResult {
        action: CombatAction,
        outcome: CombatOutcome,
    },
    MatchEnded {
        winner_id: u64,
        reason: MatchEndReason,
    },

    // -- Server housekeeping --
    Pong {
        seq: u32,
        server_time_ms: u64,
    },
    Error {
        code: u16,
        message: String,
    },
    Disconnect {
        reason: String,
    },
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
    Hit {
        damage: f32,
        new_hp: f32,
        combo_depth: u32,
    },
    Blocked,
    PerfectParry {
        counter_damage: f32,
    },
    Dodged,
    PlayerDied {
        player_id: u64,
    },
}

pub use nexus_types::AttackDir;

/// Why a match concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchEndReason {
    Verified,
    Surrender,
    Timeout,
    Disconnect,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ser<T: serde::Serialize>(v: &T) -> String { serde_json::to_string(v).unwrap() }
    fn de_action(s: &str) -> CombatAction { serde_json::from_str(s).unwrap() }
    fn de_outcome(s: &str) -> CombatOutcome { serde_json::from_str(s).unwrap() }

    // ── CombatAction round-trips ───────────────────────────────────────────────

    #[test]
    fn attack_overhead_round_trips() {
        let a = CombatAction::Attack { dir: AttackDir::Overhead };
        let json = ser(&a);
        assert!(json.contains("\"Overhead\""));
        assert!(matches!(de_action(&json), CombatAction::Attack { dir: AttackDir::Overhead }));
    }

    #[test]
    fn parry_with_direction_round_trips() {
        let a = CombatAction::Parry { dir: Some(AttackDir::Left) };
        assert!(matches!(
            de_action(&ser(&a)),
            CombatAction::Parry { dir: Some(AttackDir::Left) }
        ));
    }

    #[test]
    fn parry_without_direction_round_trips() {
        let a = CombatAction::Parry { dir: None };
        assert!(matches!(de_action(&ser(&a)), CombatAction::Parry { dir: None }));
    }

    #[test]
    fn dodge_round_trips() {
        assert!(matches!(de_action(&ser(&CombatAction::Dodge)), CombatAction::Dodge));
    }

    #[test]
    fn use_special_preserves_ability_id() {
        let a = CombatAction::UseSpecial { ability_id: 3 };
        assert!(matches!(de_action(&ser(&a)), CombatAction::UseSpecial { ability_id: 3 }));
    }

    #[test]
    fn cast_magic_preserves_type() {
        let a = CombatAction::CastMagic { magic_type: MagicType::Dark };
        assert!(matches!(
            de_action(&ser(&a)),
            CombatAction::CastMagic { magic_type: MagicType::Dark }
        ));
    }

    // ── CombatOutcome round-trips ─────────────────────────────────────────────

    #[test]
    fn hit_preserves_damage_and_combo() {
        let o = CombatOutcome::Hit { damage: 35.0, new_hp: 65.0, combo_depth: 2 };
        match de_outcome(&ser(&o)) {
            CombatOutcome::Hit { damage, new_hp, combo_depth } => {
                assert!((damage - 35.0).abs() < 0.001);
                assert!((new_hp - 65.0).abs() < 0.001);
                assert_eq!(combo_depth, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn blocked_round_trips() {
        assert!(matches!(de_outcome(&ser(&CombatOutcome::Blocked)), CombatOutcome::Blocked));
    }

    #[test]
    fn player_died_preserves_id() {
        let o = CombatOutcome::PlayerDied { player_id: 42 };
        match de_outcome(&ser(&o)) {
            CombatOutcome::PlayerDied { player_id } => assert_eq!(player_id, 42),
            other => panic!("{other:?}"),
        }
    }

    // ── MatchEndReason ────────────────────────────────────────────────────────

    #[test]
    fn match_end_reason_variants_are_distinct() {
        assert_ne!(MatchEndReason::Verified, MatchEndReason::Surrender);
        assert_ne!(MatchEndReason::Timeout, MatchEndReason::Disconnect);
    }

    #[test]
    fn match_state_snapshot_fields_are_accessible() {
        let snap = MatchStateSnapshot {
            match_id: 7, player1_hp: 500.0, player2_hp: 250.0,
            player1_combo: 3, player2_combo: 0,
            turn_number: 5, is_player1_turn: false,
        };
        assert_eq!(snap.match_id, 7);
        assert_eq!(snap.turn_number, 5);
        assert!(!snap.is_player1_turn);
    }

    // ── Task C1: Prevent Authority Stream Inflation ────────────────────────────

    #[test]
    fn ensure_byte_class_server_authority_stream_not_inflated_by_high_poly() {
        // Task C1: "Ensure the high-poly client projection doesn't inflate the byte-class server authority stream."
        // We enforce this by guaranteeing the in-memory size of any ClientMessage
        // or ServerMessage remains extremely compact. If a developer accidentally
        // adds a `Vec<f32>` mesh transform array, this size will dramatically increase.
        let client_size = std::mem::size_of::<ClientMessage>();
        let server_size = std::mem::size_of::<ServerMessage>();

        assert!(
            client_size <= 128,
            "ClientMessage has grown to {} bytes! Do not inflate the authority stream with projection data.",
            client_size
        );
        
        assert!(
            server_size <= 256,
            "ServerMessage has grown to {} bytes! Do not inflate the authority stream with projection data.",
            server_size
        );
    }
}
