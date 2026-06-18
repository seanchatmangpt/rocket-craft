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
