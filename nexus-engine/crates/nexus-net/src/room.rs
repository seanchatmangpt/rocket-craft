//! Game room — server-side per-match state and event broadcasting.
//!
//! Each `GameRoom` owns the canonical HP values, the turn counter, and a
//! `tokio::sync::broadcast` channel so every subscribed connection handler
//! receives the same event stream without locking the room for reads.

use tokio::sync::broadcast;

use crate::protocol::{CombatAction, CombatOutcome};

// ── Magic type bonus ──────────────────────────────────────────────────────────

/// Returns the flat damage bonus for each magic type.
///
/// Mapping by raw `u8` value (as carried in `CombatAction::CastMagic`):
/// - 0 → Fire        (+20 – intense but common)
/// - 1 → Lightning   (+30 – fast and piercing)
/// - 2 → Ice         (+15 – reliable, no special bonus)
/// - 3 → Dark        (+35 – high-risk, high-reward)
/// - 4 → Light       (+25 – balanced holy element)
/// - _  → 10 (unknown/future types get a small base bonus)
fn magic_type_multiplier(magic_type: u8) -> f32 {
    match magic_type {
        0 => 20.0, // Fire
        1 => 30.0, // Lightning
        2 => 15.0, // Ice
        3 => 35.0, // Dark
        4 => 25.0, // Light
        _ => 10.0, // Unknown future type
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// One player's record inside a game room.
#[derive(Debug, Clone)]
pub struct RoomPlayer {
    pub player_id: u64,
    pub name: String,
    pub suit_id: String,
    pub hp: f32,
    pub max_hp: f32,
    pub attack: f32,
    pub magic: f32,
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
    TurnResolved { action: String, result: String },
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
                let (attacker, defender) = if is_p1_acting {
                    (&mut self.player1 as *mut RoomPlayer, &mut self.player2 as *mut RoomPlayer)
                } else {
                    (&mut self.player2 as *mut RoomPlayer, &mut self.player1 as *mut RoomPlayer)
                };
                // SAFETY: attacker and defender are distinct fields of self.
                let (attacker, defender) = unsafe { (&mut *attacker, &mut *defender) };

                let base_dmg = 25.0_f32;
                let combo_bonus = attacker.combo_depth as f32 * 2.5;
                let dmg = base_dmg + combo_bonus;

                defender.hp = (defender.hp - dmg).max(0.0);
                attacker.combo_depth += 1;

                let new_hp = defender.hp;
                let combo_depth = attacker.combo_depth;

                if new_hp <= 0.0 {
                    let player_id = defender.player_id;
                    CombatOutcome::PlayerDied { player_id }
                } else {
                    CombatOutcome::Hit { damage: dmg, new_hp, combo_depth }
                }
            }
            CombatAction::Parry { .. } => {
                // Reset the acting player's combo — a parry is a defensive reset.
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
                let (attacker, defender) = if is_p1_acting {
                    (&mut self.player1 as *mut RoomPlayer, &mut self.player2 as *mut RoomPlayer)
                } else {
                    (&mut self.player2 as *mut RoomPlayer, &mut self.player1 as *mut RoomPlayer)
                };
                let (attacker, defender) = unsafe { (&mut *attacker, &mut *defender) };

                // Special abilities scale with the attacker's base attack and the
                // ability tier (ability_id 0 = basic, higher = stronger).
                let dmg = attacker.attack * 2.0 + ability_id as f32 * 5.0;
                defender.hp = (defender.hp - dmg).max(0.0);
                attacker.combo_depth = 0; // special resets combo

                let new_hp = defender.hp;
                if new_hp <= 0.0 {
                    CombatOutcome::PlayerDied { player_id: defender.player_id }
                } else {
                    CombatOutcome::Hit { damage: dmg, new_hp, combo_depth: 0 }
                }
            }
            CombatAction::CastMagic { magic_type } => {
                let (attacker, defender) = if is_p1_acting {
                    (&mut self.player1 as *mut RoomPlayer, &mut self.player2 as *mut RoomPlayer)
                } else {
                    (&mut self.player2 as *mut RoomPlayer, &mut self.player1 as *mut RoomPlayer)
                };
                let (attacker, defender) = unsafe { (&mut *attacker, &mut *defender) };

                // Magic damage scales with the attacker's magic stat plus a
                // type-specific bonus that reflects elemental potency.
                // magic_type mapping: 0=Fire, 1=Lightning, 2=Ice, 3=Dark, 4=Light
                let type_bonus = magic_type_multiplier(magic_type);
                let dmg = attacker.magic * 1.5 + type_bonus;
                defender.hp = (defender.hp - dmg).max(0.0);
                attacker.combo_depth = 0; // magic cast resets combo

                let new_hp = defender.hp;
                if new_hp <= 0.0 {
                    CombatOutcome::PlayerDied { player_id: defender.player_id }
                } else {
                    CombatOutcome::Hit { damage: dmg, new_hp, combo_depth: 0 }
                }
            }
        };

        // ── Advance turn ─────────────────────────────────────────────────────
        self.turn_number += 1;
        self.is_player1_turn = !self.is_player1_turn;

        // ── Win condition ────────────────────────────────────────────────────
        if self.player1.hp <= 0.0 || self.player2.hp <= 0.0 {
            self.state = RoomState::Ended;
            let winner_id = if self.player1.hp > 0.0 {
                self.player1.player_id
            } else {
                self.player2.player_id
            };
            // Ignore send errors — there may be no subscribers yet in tests.
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
            player1_hp: self.player1.hp,
            player2_hp: self.player2.hp,
            player1_combo: self.player1.combo_depth,
            player2_combo: self.player2.combo_depth,
            turn_number: self.turn_number,
            is_player1_turn: self.is_player1_turn,
        }
    }
}
