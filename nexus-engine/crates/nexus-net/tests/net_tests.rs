//! Integration and property tests for nexus-net.

use nexus_net::{
    matchmaking::{MatchPhase, MatchmakingQueue, QueueEntry},
    message_codec::MessageCodec,
    protocol::{AttackDir, ClientMessage, CombatAction, CombatOutcome, ServerMessage},
    room::{GameRoom, RoomPlayer, RoomState, ServerRoomEvent},
};
use proptest::prelude::*;

// ── Codec roundtrip ───────────────────────────────────────────────────────────

#[test]
fn roundtrip_client_message_serialization() {
    let msgs: Vec<ClientMessage> = vec![
        ClientMessage::Authenticate { player_id: 42, token: "token123".to_string() },
        ClientMessage::CombatAction { action: CombatAction::Attack { dir: AttackDir::Overhead } },
        ClientMessage::Ping { seq: 999 },
        ClientMessage::SurrenderMatch,
        ClientMessage::JoinLobby,
        ClientMessage::LeaveLobby,
        ClientMessage::ChallengeDuel {
            target_player_id: 7,
            suit_id: "Unicorn".to_string(),
        },
        ClientMessage::AcceptDuel { challenge_id: 1 },
        ClientMessage::DeclineDuel { challenge_id: 2 },
        ClientMessage::EmotePing { emote_id: 3 },
    ];

    for msg in msgs {
        let encoded = MessageCodec::encode_client(&msg).unwrap();
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        // Re-encode and compare JSON bytes — structural equality check.
        let re_encoded = MessageCodec::encode_client(&decoded).unwrap();
        assert_eq!(
            encoded, re_encoded,
            "roundtrip should preserve message: {}",
            String::from_utf8_lossy(&encoded)
        );
    }
}

#[test]
fn roundtrip_server_message_serialization() {
    let msgs: Vec<ServerMessage> = vec![
        ServerMessage::AuthSuccess { session_id: 12345 },
        ServerMessage::AuthFailure { reason: "bad token".to_string() },
        ServerMessage::LobbyJoined { online_count: 42 },
        ServerMessage::Pong { seq: 1, server_time_ms: 99999 },
        ServerMessage::Error { code: 404, message: "not found".to_string() },
        ServerMessage::Disconnect { reason: "server restart".to_string() },
        ServerMessage::MatchEnded {
            winner_id: 1,
            reason: nexus_net::protocol::MatchEndReason::Victory,
        },
    ];

    for msg in msgs {
        let encoded = MessageCodec::encode_server(&msg).unwrap();
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        let re_encoded = MessageCodec::encode_server(&decoded).unwrap();
        assert_eq!(
            encoded, re_encoded,
            "server message roundtrip failed: {}",
            String::from_utf8_lossy(&encoded)
        );
    }
}

#[test]
fn all_combat_actions_roundtrip() {
    let actions: Vec<CombatAction> = vec![
        CombatAction::Attack { dir: AttackDir::Overhead },
        CombatAction::Attack { dir: AttackDir::Left },
        CombatAction::Attack { dir: AttackDir::Right },
        CombatAction::Parry { dir: Some(AttackDir::Overhead) },
        CombatAction::Parry { dir: None },
        CombatAction::Dodge,
        CombatAction::UseSpecial { ability_id: 1 },
        CombatAction::CastMagic { magic_type: 2 },
    ];

    for action in actions {
        let msg = ClientMessage::CombatAction { action };
        let bytes = MessageCodec::encode_client(&msg).unwrap();
        let decoded = MessageCodec::decode_client(&bytes).unwrap();
        let re_bytes = MessageCodec::encode_client(&decoded).unwrap();
        assert_eq!(bytes, re_bytes);
    }
}

#[test]
fn size_check_rejects_oversized_payload() {
    let msg = ClientMessage::Ping { seq: 1 };
    let bytes = MessageCodec::encode_client(&msg).unwrap();
    // Should pass when max is larger than actual size
    assert!(MessageCodec::check_size(&bytes, bytes.len()).is_ok());
    // Should fail when max is smaller
    assert!(MessageCodec::check_size(&bytes, bytes.len() - 1).is_err());
}

// ── Matchmaking ───────────────────────────────────────────────────────────────

fn make_entry(player_id: u64, name: &str, suit: &str, rating: u32) -> QueueEntry {
    QueueEntry {
        player_id,
        player_name: name.to_string(),
        suit_id: suit.to_string(),
        rating,
        queued_at: chrono::Utc::now(),
    }
}

#[test]
fn matchmaking_pairs_players_within_rating() {
    let mut queue = MatchmakingQueue::new();

    let p1 = make_entry(1, "Suletta", "Aerial", 1500);
    let p2 = make_entry(2, "Miorine", "Pharact", 1600);

    let result1 = queue.enqueue(p1);
    assert!(result1.is_none(), "first player should wait in queue");
    assert_eq!(queue.queue_len(), 1);

    let result2 = queue.enqueue(p2);
    assert!(result2.is_some(), "second player within 200 rating should match");
    assert_eq!(queue.queue_len(), 0);

    let m = result2.unwrap();
    assert_eq!(m.state, MatchPhase::InProgress);
    assert_eq!(m.player1_id, 1);
    assert_eq!(m.player2_id, 2);
}

#[test]
fn matchmaking_does_not_pair_players_too_far_apart() {
    let mut queue = MatchmakingQueue::new();

    let p1 = make_entry(1, "Amuro", "NuGundam", 1000);
    let p2 = make_entry(2, "Char", "Sazabi", 1500); // 500 rating gap

    let r1 = queue.enqueue(p1);
    let r2 = queue.enqueue(p2);

    assert!(r1.is_none());
    assert!(r2.is_none(), "500 rating gap should not match");
    assert_eq!(queue.queue_len(), 2);
}

#[test]
fn matchmaking_dequeue_removes_player() {
    let mut queue = MatchmakingQueue::new();
    // Use ratings more than 200 apart so neither enqueue triggers a match.
    queue.enqueue(make_entry(1, "P1", "s1", 800));
    queue.enqueue(make_entry(2, "P2", "s2", 1500));
    assert_eq!(queue.queue_len(), 2);
    queue.dequeue(1);
    assert_eq!(queue.queue_len(), 1);
}

#[test]
fn matchmaking_complete_match_updates_state() {
    let mut queue = MatchmakingQueue::new();
    queue.enqueue(make_entry(1, "A", "s1", 1500));
    let active = queue.enqueue(make_entry(2, "B", "s2", 1550)).unwrap();
    let mid = active.match_id;

    queue.complete_match(mid);
    assert_eq!(queue.get_match(mid).unwrap().state, MatchPhase::Completed);

    queue.remove_match(mid);
    assert!(queue.get_match(mid).is_none());
}

#[test]
fn matchmaking_next_match_id_increments() {
    let mut queue = MatchmakingQueue::new();
    queue.enqueue(make_entry(1, "A", "s1", 1000));
    let m1 = queue.enqueue(make_entry(2, "B", "s2", 1001)).unwrap();
    queue.enqueue(make_entry(3, "C", "s3", 1000));
    let m2 = queue.enqueue(make_entry(4, "D", "s4", 1001)).unwrap();
    assert_ne!(m1.match_id, m2.match_id);
}

// ── Room ──────────────────────────────────────────────────────────────────────

fn make_player(id: u64, name: &str, suit: &str, hp: f32) -> RoomPlayer {
    RoomPlayer {
        player_id: id,
        name: name.to_string(),
        suit_id: suit.to_string(),
        hp,
        max_hp: hp,
        attack: 30.0,
        magic: 50.0,
        combo_depth: 0,
    }
}

#[test]
fn room_tracks_hp_and_win_condition() {
    let p1 = make_player(1, "Amuro", "NuGundam", 100.0);
    let p2 = make_player(2, "Char", "Sazabi", 100.0);
    let mut room = GameRoom::new(1, p1, p2);
    room.state = RoomState::Active;

    // Player 1 attacks player 2
    let outcome =
        room.apply_action(1, CombatAction::Attack { dir: AttackDir::Overhead }).unwrap();
    assert!(matches!(outcome, CombatOutcome::Hit { .. }), "expected Hit, got {:?}", outcome);
    assert!(room.player2.hp < 100.0, "player2 HP should be reduced");
    assert!(!room.is_player1_turn, "turn should have switched to player 2");
    assert_eq!(room.turn_number, 1);
}

#[test]
fn room_rejects_action_when_not_active() {
    let p1 = make_player(1, "A", "s1", 100.0);
    let p2 = make_player(2, "B", "s2", 100.0);
    let mut room = GameRoom::new(1, p1, p2);
    // state is WaitingForBothPlayers by default

    let err = room.apply_action(1, CombatAction::Dodge).unwrap_err();
    assert!(err.to_string().contains("not active"));
}

#[test]
fn room_rejects_out_of_turn_action() {
    let p1 = make_player(1, "A", "s1", 100.0);
    let p2 = make_player(2, "B", "s2", 100.0);
    let mut room = GameRoom::new(1, p1, p2);
    room.state = RoomState::Active;

    // It is player 1's turn; player 2 attempts to act.
    let err = room.apply_action(2, CombatAction::Dodge).unwrap_err();
    assert!(err.to_string().contains("turn"));
}

#[test]
fn room_rejects_unknown_player() {
    let p1 = make_player(1, "A", "s1", 100.0);
    let p2 = make_player(2, "B", "s2", 100.0);
    let mut room = GameRoom::new(1, p1, p2);
    room.state = RoomState::Active;

    let err = room.apply_action(99, CombatAction::Dodge).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn room_ends_when_player_dies() {
    let p1 = make_player(1, "A", "s1", 1.0); // 1 HP — one hit kills
    let p2 = make_player(2, "B", "s2", 100.0);
    let mut room = GameRoom::new(1, p1, p2);
    room.state = RoomState::Active;

    // Player 2's turn first? No — player 1 goes first, but player 1 has 1 HP.
    // Let's swap so player 2 can kill player 1 on the first action.
    room.is_player1_turn = false; // make it player 2's turn
    let outcome = room.apply_action(2, CombatAction::Attack { dir: AttackDir::Left }).unwrap();
    assert!(
        matches!(outcome, CombatOutcome::PlayerDied { .. } | CombatOutcome::Hit { .. }),
        "outcome: {:?}",
        outcome
    );
    assert_eq!(room.state, RoomState::Ended);
}

#[test]
fn room_snapshot_reflects_state() {
    let p1 = make_player(1, "A", "s1", 100.0);
    let p2 = make_player(2, "B", "s2", 80.0);
    let room = GameRoom::new(42, p1, p2);
    let snap = room.snapshot();
    assert_eq!(snap.match_id, 42);
    assert_eq!(snap.player1_hp, 100.0);
    assert_eq!(snap.player2_hp, 80.0);
    assert!(snap.is_player1_turn);
}

#[tokio::test]
async fn room_broadcast_received_by_subscriber() {
    let p1 = make_player(1, "test1", "s1", 1.0); // 1 HP — will die on first hit
    let p2 = make_player(2, "test2", "s2", 100.0);
    let mut room = GameRoom::new(1, p1, p2);
    let mut sub = room.subscribe();
    room.state = RoomState::Active;

    // Player 2 attacks player 1 (who has 1 HP) — should kill immediately.
    room.is_player1_turn = false;
    let _ = room.apply_action(2, CombatAction::Attack { dir: AttackDir::Overhead }).unwrap();

    // Subscriber should receive MatchEnded.
    let event = sub.try_recv();
    assert!(event.is_ok(), "subscriber should have received MatchEnded event");
    assert!(
        matches!(event.unwrap(), ServerRoomEvent::MatchEnded { .. }),
        "expected MatchEnded"
    );
}

#[tokio::test]
async fn room_broadcast_manual_event() {
    let p1 = make_player(1, "A", "s1", 100.0);
    let p2 = make_player(2, "B", "s2", 100.0);
    let room = GameRoom::new(1, p1, p2);
    let mut sub = room.subscribe();

    room.broadcast(ServerRoomEvent::MatchStarted { match_id: 1 });
    let event = sub.try_recv().unwrap();
    assert!(matches!(event, ServerRoomEvent::MatchStarted { match_id: 1 }));
}

// ── Typestate connection ──────────────────────────────────────────────────────

#[test]
fn connection_typestate_happy_path() {
    use nexus_net::connection::Connection;

    let conn = Connection::new();
    let conn = conn.begin_handshake();
    let conn = conn.complete();
    let conn = conn.authenticate(42, 9999);
    let conn = conn.join_lobby();
    let conn = conn.enter_match(7);
    assert_eq!(conn.current_match_id(), 7);
    let conn = conn.finish_match();
    let _conn = conn.disconnect();
}

#[test]
fn connection_failed_handshake_returns_disconnected() {
    use nexus_net::connection::Connection;

    let conn = Connection::new();
    let conn = conn.begin_handshake();
    let _conn = conn.fail(); // back to Disconnected<_>
}

// ── Proptest ──────────────────────────────────────────────────────────────────

proptest! {
    /// The combat-action codec is lossless for all attack directions.
    #[test]
    fn combat_action_roundtrip(
        dir in prop::sample::select(vec![AttackDir::Overhead, AttackDir::Left, AttackDir::Right])
    ) {
        let msg = ClientMessage::CombatAction { action: CombatAction::Attack { dir } };
        let bytes = MessageCodec::encode_client(&msg).unwrap();
        let decoded = MessageCodec::decode_client(&bytes).unwrap();
        match decoded {
            ClientMessage::CombatAction { action: CombatAction::Attack { dir: decoded_dir } } => {
                prop_assert_eq!(format!("{dir:?}"), format!("{decoded_dir:?}"));
            }
            _ => prop_assert!(false, "should decode as CombatAction::Attack"),
        }
    }

    /// Ping roundtrip preserves the sequence number.
    #[test]
    fn ping_seq_preserved(seq in 0u32..u32::MAX) {
        let msg = ClientMessage::Ping { seq };
        let bytes = MessageCodec::encode_client(&msg).unwrap();
        let decoded = MessageCodec::decode_client(&bytes).unwrap();
        match decoded {
            ClientMessage::Ping { seq: decoded_seq } => {
                prop_assert_eq!(seq, decoded_seq);
            }
            _ => prop_assert!(false, "expected Ping"),
        }
    }

    /// Matchmaking never pairs players more than 200 rating apart.
    #[test]
    fn matchmaking_rating_invariant(
        r1 in 800u32..2000u32,
        r2 in 800u32..2000u32,
    ) {
        let mut queue = MatchmakingQueue::new();
        let p1 = make_entry(1, "A", "s1", r1);
        let p2 = make_entry(2, "B", "s2", r2);
        queue.enqueue(p1);
        let result = queue.enqueue(p2);

        let gap = (r1 as i64 - r2 as i64).unsigned_abs() as u32;
        if gap <= 200 {
            prop_assert!(result.is_some(), "should match within 200 rating");
        } else {
            prop_assert!(result.is_none(), "should not match beyond 200 rating");
        }
    }
}
