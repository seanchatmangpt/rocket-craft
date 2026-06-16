use crate::hud::HudData;

/// Mirrors the GameToUiMessage enum from wasm-game-logic (duplicated to avoid cross-crate dep)
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UiToGameMessage {
    Input(InputCommand),
    Pause,
    Resume,
    Restart,
    Ping { seq: u32 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputCommand {
    Move { entity: u32, dx: f32, dy: f32 },
    Attack { attacker: u32, target: u32 },
    UseItem { entity: u32, item_id: u32 },
}

/// Stateless message dispatcher — converts incoming JSON to HudData
pub struct MessageBridge {
    pub messages_processed: u64,
    pub last_tick: u64,
}

impl MessageBridge {
    pub fn new() -> Self {
        Self {
            messages_processed: 0,
            last_tick: 0,
        }
    }

    pub fn process(&mut self, json: &str) -> Option<HudData> {
        let msg: GameToUiMessage = serde_json::from_str(json).ok()?;
        self.messages_processed += 1;
        match msg {
            GameToUiMessage::StateUpdate {
                tick,
                entity_count,
                player_health,
                player_health_max,
                player_score,
            } => {
                self.last_tick = tick;
                Some(HudData {
                    player_health: player_health.unwrap_or(0),
                    player_health_max: player_health_max.unwrap_or(100),
                    score: player_score,
                    entity_count,
                    game_tick: tick,
                    fps: 0.0, // computed elsewhere
                    messages_per_second: 0.0,
                })
            }
            GameToUiMessage::GameOver {
                winner_score,
                total_ticks,
            } => Some(HudData {
                player_health: 0,
                player_health_max: 100,
                score: winner_score,
                entity_count: 0,
                game_tick: total_ticks,
                fps: 0.0,
                messages_per_second: 0.0,
            }),
            _ => None,
        }
    }

    pub fn serialize_to_game(&self, msg: &UiToGameMessage) -> Option<String> {
        serde_json::to_string(msg).ok()
    }
}

impl Default for MessageBridge {
    fn default() -> Self {
        Self::new()
    }
}
