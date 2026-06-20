use crate::systems::InputCommand;

/// Messages sent from the game-logic worker to the UI worker.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GameToUiMessage {
    StateUpdate {
        tick: u64,
        entity_count: usize,
        player_health: Option<u32>,
        player_health_max: Option<u32>,
        player_score: u64,
    },
    GameOver {
        winner_score: u64,
        total_ticks: u64,
    },
    EntityMoved {
        entity_id: u32,
        x: f32,
        y: f32,
    },
    EntityDied {
        entity_id: u32,
    },
}

/// Messages sent from the UI worker to the game-logic worker.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UiToGameMessage {
    Input(InputCommand),
    Pause,
    Resume,
    Restart,
    Ping { seq: u32 },
}

/// Replies from the game-logic worker that aren't UI state updates.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FromGameMessage {
    Pong { seq: u32 },
}

impl GameToUiMessage {
    /// Serialize this message to a JSON string ready for `postMessage`.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from a JSON string received from `postMessage`.
    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

impl UiToGameMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::InputCommand;

    // ── GameToUiMessage roundtrip ─────────────────────────────────────────────

    #[test]
    fn state_update_roundtrips_json() {
        let msg = GameToUiMessage::StateUpdate {
            tick: 42, entity_count: 5, player_health: Some(80),
            player_health_max: Some(100), player_score: 1000,
        };
        let json = msg.to_json();
        let back = GameToUiMessage::from_json(&json).unwrap();
        if let GameToUiMessage::StateUpdate { tick, player_score, .. } = back {
            assert_eq!(tick, 42);
            assert_eq!(player_score, 1000);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn game_over_roundtrips_json() {
        let msg = GameToUiMessage::GameOver { winner_score: 9999, total_ticks: 300 };
        let back = GameToUiMessage::from_json(&msg.to_json()).unwrap();
        if let GameToUiMessage::GameOver { winner_score, .. } = back {
            assert_eq!(winner_score, 9999);
        } else { panic!("wrong variant"); }
    }

    #[test]
    fn from_json_returns_none_on_garbage() {
        assert!(GameToUiMessage::from_json("not-json").is_none());
    }

    // ── UiToGameMessage roundtrip ─────────────────────────────────────────────

    #[test]
    fn ping_roundtrips_json() {
        let msg = UiToGameMessage::Ping { seq: 7 };
        let back = UiToGameMessage::from_json(&msg.to_json()).unwrap();
        if let UiToGameMessage::Ping { seq } = back {
            assert_eq!(seq, 7);
        } else { panic!("wrong variant"); }
    }

    #[test]
    fn input_command_roundtrips_through_ui_to_game() {
        let msg = UiToGameMessage::Input(InputCommand::Move { entity: 1, dx: 1.0, dy: 0.0 });
        let back = UiToGameMessage::from_json(&msg.to_json()).unwrap();
        assert!(matches!(back, UiToGameMessage::Input(InputCommand::Move { .. })));
    }

    #[test]
    fn pause_resume_restart_roundtrip() {
        for msg in [UiToGameMessage::Pause, UiToGameMessage::Resume, UiToGameMessage::Restart] {
            let back = UiToGameMessage::from_json(&msg.to_json());
            assert!(back.is_some());
        }
    }
}
