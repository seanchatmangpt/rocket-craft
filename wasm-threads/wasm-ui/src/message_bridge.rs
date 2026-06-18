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

/// Stateful message dispatcher — converts incoming JSON to HudData and tracks entity positions
pub struct MessageBridge {
    pub messages_processed: u64,
    pub last_tick: u64,
    pub entity_positions: std::collections::HashMap<u32, (f32, f32)>,
    pub last_player_health: u32,
    pub last_player_health_max: u32,
    pub last_score: u64,
}

impl MessageBridge {
    pub fn new() -> Self {
        Self {
            messages_processed: 0,
            last_tick: 0,
            entity_positions: std::collections::HashMap::new(),
            last_player_health: 0,
            last_player_health_max: 100,
            last_score: 0,
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
                let health = player_health.unwrap_or(0);
                let health_max = player_health_max.unwrap_or(100);
                self.last_player_health = health;
                self.last_player_health_max = health_max;
                self.last_score = player_score;
                Some(HudData {
                    player_health: health,
                    player_health_max: health_max,
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
            } => {
                self.last_player_health = 0;
                self.last_score = winner_score;
                Some(HudData {
                    player_health: 0,
                    player_health_max: self.last_player_health_max,
                    score: winner_score,
                    entity_count: 0,
                    game_tick: total_ticks,
                    fps: 0.0,
                    messages_per_second: 0.0,
                })
            }
            GameToUiMessage::EntityMoved { entity_id, x, y } => {
                self.entity_positions.insert(entity_id, (x, y));
                Some(HudData {
                    player_health: self.last_player_health,
                    player_health_max: self.last_player_health_max,
                    score: self.last_score,
                    entity_count: self.entity_positions.len(),
                    game_tick: self.last_tick,
                    fps: 0.0,
                    messages_per_second: 0.0,
                })
            }
            GameToUiMessage::EntityDied { entity_id } => {
                self.entity_positions.remove(&entity_id);
                Some(HudData {
                    player_health: self.last_player_health,
                    player_health_max: self.last_player_health_max,
                    score: self.last_score,
                    entity_count: self.entity_positions.len(),
                    game_tick: self.last_tick,
                    fps: 0.0,
                    messages_per_second: 0.0,
                })
            }
        }
    }

    pub fn serialize_to_game(&self, msg: &UiToGameMessage) -> Option<String> {
        serde_json::to_string(msg).ok()
    }

    pub fn send_ping(&self, seq: u32) -> String {
        let msg = UiToGameMessage::Ping { seq };
        serde_json::to_string(&msg).unwrap_or_default()
    }
}

impl Default for MessageBridge {
    fn default() -> Self {
        Self::new()
    }
}
