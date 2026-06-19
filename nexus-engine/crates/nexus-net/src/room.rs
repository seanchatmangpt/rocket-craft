//! Game room — server-side per-match state and event broadcasting.
//!
//! Each `GameRoom` owns the canonical HP values, the turn counter, and a
//! `tokio::sync::broadcast` channel so every subscribed connection handler
//! receives the same event stream without locking the room for reads.

use tokio::sync::broadcast;

use nexus_types::{Damage, Hp};

use crate::protocol::{CombatAction, CombatOutcome};

// ── Types ─────────────────────────────────────────────────────────────────────

/// One player's record inside a game room.
#[derive(Debug, Clone)]
pub struct RoomPlayer {
    pub player_id: u64,
    pub name: String,
    pub suit_id: String,
    pub hp: Hp,
    pub max_hp: Hp,
    pub attack: Damage,
    pub magic: Damage,
    pub combo_depth: u32,
}

/// High-level lifecycle state of the room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomState {
    WaitingForBothPlayers,
    Active,
    Ended,
}

/// Events broadcast to all subscribers of a room.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ServerRoomEvent {
    MatchStarted { match_id: u64 },
    TurnRereaddressed { action: String, result: String },
    MatchEnded { winner_id: u64 },
}

// ── GameRoom ──────────────────────────────────────────────────────────────────

/// Server-side authoritative state for one duel match.
pub struct GameRoom {
    pub match_id: u64,
    pub player1: RoomPlayer,
    pub player2: RoomPlayer,
    pub turn_number: u32,
    pub is_player1_turn: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub state: RoomState,
    /// Broadcast channel — `send` from server tasks, `subscribe` from handlers.
    tx: broadcast::Sender<ServerRoomEvent>,
}

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors that can occur when applying a combat action to a room.
#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("match is not active")]
    MatchNotActive,
    #[error("not this player's turn")]
    NotPlayersTurn,
    #[error("player not found in match")]
    PlayerNotFound,
}

// ── Implementation ────────────────────────────────────────────────────────────

impl GameRoom {
    /// Create a new room in the `WaitingForBothPlayers` state.
    pub fn new(match_id: u64, player1: RoomPlayer, player2: RoomPlayer) -> Self {
        let (tx, _) = broadcast::channel(32);
        GameRoom {
            match_id,
            player1,
            player2,
            turn_number: 0,
            is_player1_turn: true,
            started_at: chrono::Utc::now(),
            state: RoomState::WaitingForBothPlayers,
            tx,
        }
    }

    /// Subscribe to the room event stream.
    ///
    /// Returns a `Receiver` that yields every `ServerRoomEvent` published after
    /// the subscription is created.  Lagged receivers receive an error rather
    /// than losing items silently.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerRoomEvent> {
        self.tx.subscribe()
    }

    /// Apply a combat action submitted by `actor_id`.
    ///
    /// Validates that:
    /// - The room is `Active`.
    /// - It is the submitting player's turn.
    ///
    /// Resolves the action, updates HP, advances the turn counter, checks the
    /// win condition, and (on match end) broadcasts a `MatchEnded` event.
    pub fn apply_action(
        &mut self,
        actor_id: u64,
        action: CombatAction,
    ) -> Result<CombatOutcome, RoomError> {
        if self.state != RoomState::Active {
            return Err(RoomError::MatchNotActive);
        }

        let is_p1_acting = if actor_id == self.player1.player_id {
            true
        } else if actor_id == self.player2.player_id {
            false
        } else {
            return Err(RoomError::PlayerNotFound);
        };

        let is_players_turn =
            (is_p1_acting && self.is_player1_turn) || (!is_p1_acting && !self.is_player1_turn);
        if !is_players_turn {
            return Err(RoomError::NotPlayersTurn);
        }

        // ── Resolve the action ───────────────────────────────────────────────
        let outcome = match action {
            CombatAction::Attack { .. } => {
                let combo_before = if is_p1_acting {
                    self.player1.combo_depth
                } else {
                    self.player2.combo_depth
                };

                let dmg = Damage::new(25.0 + combo_before as f32 * 2.5);

                let (new_hp, defender_id) = if is_p1_acting {
                    self.player2.hp = Hp::new((self.player2.hp.value() - dmg.value()).max(0.0));
                    self.player1.combo_depth = combo_before + 1;
                    (self.player2.hp, self.player2.player_id)
                } else {
                    self.player1.hp = Hp::new((self.player1.hp.value() - dmg.value()).max(0.0));
                    self.player2.combo_depth = combo_before + 1;
                    (self.player1.hp, self.player1.player_id)
                };

                let combo_depth = if is_p1_acting {
                    self.player1.combo_depth
                } else {
                    self.player2.combo_depth
                };

                if new_hp.is_dead() {
                    CombatOutcome::PlayerDied {
                        player_id: defender_id,
                    }
                } else {
                    CombatOutcome::Hit {
                        damage: dmg.value(),
                        new_hp: new_hp.value(),
                        combo_depth,
                    }
                }
            }
            CombatAction::Parry { .. } => {
                if is_p1_acting {
                    self.player1.combo_depth = 0;
                } else {
                    self.player2.combo_depth = 0;
                }
                CombatOutcome::Blocked
            }
            CombatAction::Dodge => {
                if is_p1_acting {
                    self.player1.combo_depth = 0;
                } else {
                    self.player2.combo_depth = 0;
                }
                CombatOutcome::Dodged
            }
            CombatAction::UseSpecial { ability_id } => {
                let attacker_attack = if is_p1_acting {
                    self.player1.attack
                } else {
                    self.player2.attack
                };

                let dmg = Damage::new(attacker_attack.value() * 2.0 + ability_id as f32 * 5.0);

                let (new_hp, defender_id) = if is_p1_acting {
                    self.player2.hp = Hp::new((self.player2.hp.value() - dmg.value()).max(0.0));
                    self.player1.combo_depth = 0;
                    (self.player2.hp, self.player2.player_id)
                } else {
                    self.player1.hp = Hp::new((self.player1.hp.value() - dmg.value()).max(0.0));
                    self.player2.combo_depth = 0;
                    (self.player1.hp, self.player1.player_id)
                };

                if new_hp.is_dead() {
                    CombatOutcome::PlayerDied {
                        player_id: defender_id,
                    }
                } else {
                    CombatOutcome::Hit {
                        damage: dmg.value(),
                        new_hp: new_hp.value(),
                        combo_depth: 0,
                    }
                }
            }
            CombatAction::CastMagic { magic_type } => {
                let attacker_magic = if is_p1_acting {
                    self.player1.magic
                } else {
                    self.player2.magic
                };

                let type_bonus = f32::from(magic_type);
                let dmg = Damage::new(attacker_magic.value() * 1.5 + type_bonus);

                let (new_hp, defender_id) = if is_p1_acting {
                    self.player2.hp = Hp::new((self.player2.hp.value() - dmg.value()).max(0.0));
                    self.player1.combo_depth = 0;
                    (self.player2.hp, self.player2.player_id)
                } else {
                    self.player1.hp = Hp::new((self.player1.hp.value() - dmg.value()).max(0.0));
                    self.player2.combo_depth = 0;
                    (self.player1.hp, self.player1.player_id)
                };

                if new_hp.is_dead() {
                    CombatOutcome::PlayerDied {
                        player_id: defender_id,
                    }
                } else {
                    CombatOutcome::Hit {
                        damage: dmg.value(),
                        new_hp: new_hp.value(),
                        combo_depth: 0,
                    }
                }
            }
        };

        // ── Advance turn ─────────────────────────────────────────────────────
        self.turn_number += 1;
        self.is_player1_turn = !self.is_player1_turn;

        // ── Win condition ────────────────────────────────────────────────────
        if self.player1.hp.is_dead() || self.player2.hp.is_dead() {
            self.state = RoomState::Ended;
            let winner_id = if !self.player1.hp.is_dead() {
                self.player1.player_id
            } else {
                self.player2.player_id
            };
            let _ = self.tx.send(ServerRoomEvent::MatchEnded { winner_id });
        }

        Ok(outcome)
    }

    /// Broadcast an arbitrary room event to all active subscribers.
    pub fn broadcast(&self, event: ServerRoomEvent) {
        let _ = self.tx.send(event);
    }

    /// Snapshot the current match state as a serialisable struct.
    pub fn snapshot(&self) -> crate::protocol::MatchStateSnapshot {
        crate::protocol::MatchStateSnapshot {
            match_id: self.match_id,
            player1_hp: self.player1.hp.value(),
            player2_hp: self.player2.hp.value(),
            player1_combo: self.player1.combo_depth,
            player2_combo: self.player2.combo_depth,
            turn_number: self.turn_number,
            is_player1_turn: self.is_player1_turn,
        }
    }
}
