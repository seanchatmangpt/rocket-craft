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

#[cfg(test)]
mod tests {
    use super::*;

    fn state_update_json(tick: u64, health: u32, score: u64) -> String {
        format!(
            r#"{{"StateUpdate":{{"tick":{tick},"entity_count":3,"player_health":{health},"player_health_max":100,"player_score":{score}}}}}"#
        )
    }

    #[test]
    fn new_bridge_starts_zeroed() {
        let b = MessageBridge::new();
        assert_eq!(b.messages_processed, 0);
        assert_eq!(b.last_tick, 0);
        assert!(b.entity_positions.is_empty());
    }

    #[test]
    fn process_invalid_json_returns_none_no_increment() {
        let mut b = MessageBridge::new();
        assert!(b.process("not-json").is_none());
        assert_eq!(b.messages_processed, 0);
    }

    #[test]
    fn process_state_update_increments_counter_and_updates_fields() {
        let mut b = MessageBridge::new();
        let hud = b.process(&state_update_json(42, 80, 999)).unwrap();
        assert_eq!(b.messages_processed, 1);
        assert_eq!(b.last_tick, 42);
        assert_eq!(b.last_player_health, 80);
        assert_eq!(b.last_score, 999);
        assert_eq!(hud.player_health, 80);
        assert_eq!(hud.score, 999);
        assert_eq!(hud.game_tick, 42);
    }

    #[test]
    fn process_entity_moved_updates_position_map() {
        let mut b = MessageBridge::new();
        let json = r#"{"EntityMoved":{"entity_id":7,"x":1.5,"y":2.5}}"#;
        b.process(json).unwrap();
        assert_eq!(b.entity_positions.get(&7), Some(&(1.5, 2.5)));
    }

    #[test]
    fn process_entity_died_removes_from_position_map() {
        let mut b = MessageBridge::new();
        b.process(r#"{"EntityMoved":{"entity_id":7,"x":1.0,"y":2.0}}"#).unwrap();
        b.process(r#"{"EntityDied":{"entity_id":7}}"#).unwrap();
        assert!(!b.entity_positions.contains_key(&7));
    }

    #[test]
    fn process_game_over_zeroes_player_health() {
        let mut b = MessageBridge::new();
        b.process(&state_update_json(1, 50, 0)).unwrap();
        let hud = b.process(r#"{"GameOver":{"winner_score":9999,"total_ticks":100}}"#).unwrap();
        assert_eq!(b.last_player_health, 0);
        assert_eq!(hud.player_health, 0);
        assert_eq!(hud.score, 9999);
    }

    #[test]
    fn serialize_to_game_produces_valid_json() {
        let b = MessageBridge::new();
        let json = b.serialize_to_game(&UiToGameMessage::Pause).unwrap();
        assert!(json.contains("Pause"));
    }

    #[test]
    fn send_ping_encodes_seq() {
        let b = MessageBridge::new();
        let json = b.send_ping(42);
        assert!(json.contains("42"));
        assert!(json.contains("Ping"));
    }
}
