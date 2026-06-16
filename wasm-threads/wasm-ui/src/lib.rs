pub mod hud;
pub mod message_bridge;
pub mod renderer;
pub mod ui_state;

pub use hud::{render_frame, HudData};
pub use message_bridge::{GameToUiMessage, InputCommand, MessageBridge, UiToGameMessage};
pub use renderer::{DrawCall, Renderer, TestRenderer};
pub use ui_state::{Error, Loading, Ready, Unloaded, UiState};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// JS-callable UI controller
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct UiController {
    bridge: MessageBridge,
    frame: u64,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl UiController {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            bridge: MessageBridge::new(),
            frame: 0,
        }
    }

    /// Call this from the worker's onmessage handler with the JSON data
    pub fn on_game_message(&mut self, json: &str) -> String {
        match self.bridge.process(json) {
            Some(hud) => serde_json::to_string(&hud).unwrap_or_default(),
            None => "{}".to_string(),
        }
    }

    /// Build a JSON message to send to the game logic worker
    pub fn send_move(&self, entity: u32, dx: f32, dy: f32) -> String {
        let msg = UiToGameMessage::Input(InputCommand::Move { entity, dx, dy });
        self.bridge.serialize_to_game(&msg).unwrap_or_default()
    }

    pub fn send_attack(&self, attacker: u32, target: u32) -> String {
        let msg = UiToGameMessage::Input(InputCommand::Attack { attacker, target });
        self.bridge.serialize_to_game(&msg).unwrap_or_default()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame
    }

    pub fn messages_processed(&self) -> u64 {
        self.bridge.messages_processed
    }

    pub fn last_game_tick(&self) -> u64 {
        self.bridge.last_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // --- UI State transitions ---
    #[test]
    fn ui_state_full_happy_path() {
        let s = UiState::<Unloaded>::new();
        let s = s.start_loading();
        let mut s = s.ready();
        s.update_from_game(42, 80, 100, 1500, 10);
        assert_eq!(s.last_game_tick, 42);
        assert_eq!(s.player_health, 80);
        assert_eq!(s.player_score, 1500);
        assert_eq!(s.messages_received, 1);
        s.render_frame();
        assert_eq!(s.frame, 1);
    }

    #[test]
    fn ui_state_error_can_retry() {
        let s = UiState::<Unloaded>::new();
        let s = s.start_loading();
        let s = s.fail("connection refused");
        let _s = s.retry(); // back to Unloaded
    }

    // --- Renderer ---
    #[test]
    fn test_renderer_records_draw_calls() {
        let mut r = TestRenderer::new();
        r.clear();
        r.draw_health_bar(0.0, 0.0, 100.0, 20.0, 0.75);
        r.draw_score(999);
        r.present();
        assert_eq!(r.frame_count(), 1);
        assert_eq!(r.call_count(), 4);
        assert!(r.has_call(&DrawCall::Clear));
        assert!(r.has_call(&DrawCall::Score(999)));
    }

    // Falsification: renderer must record different calls for different inputs
    #[test]
    fn renderer_health_bar_reflects_actual_percentage() {
        let mut r = TestRenderer::new();
        r.draw_health_bar(0.0, 0.0, 100.0, 20.0, 0.5);
        r.draw_health_bar(0.0, 0.0, 100.0, 20.0, 0.9);
        let bars: Vec<_> = r
            .draw_calls()
            .iter()
            .filter_map(|c| {
                if let DrawCall::HealthBar { percentage, .. } = c {
                    Some(*percentage)
                } else {
                    None
                }
            })
            .collect();
        assert_ne!(
            bars[0], bars[1],
            "different health must produce different draw calls"
        );
    }

    // --- HUD ---
    #[test]
    fn hud_color_depends_on_health() {
        let full = HudData {
            player_health: 100,
            player_health_max: 100,
            score: 0,
            entity_count: 0,
            game_tick: 0,
            fps: 0.0,
            messages_per_second: 0.0,
        };
        let low = HudData {
            player_health: 10,
            player_health_max: 100,
            score: 0,
            entity_count: 0,
            game_tick: 0,
            fps: 0.0,
            messages_per_second: 0.0,
        };
        assert_ne!(
            full.health_color(),
            low.health_color(),
            "color must differ by health level"
        );
        assert_eq!(full.health_color(), "#00ff00");
        assert_eq!(low.health_color(), "#ff0000");
    }

    #[test]
    fn hud_critical_health_threshold() {
        let critical = HudData {
            player_health: 20,
            player_health_max: 100,
            score: 0,
            entity_count: 0,
            game_tick: 0,
            fps: 0.0,
            messages_per_second: 0.0,
        };
        let fine = HudData {
            player_health: 80,
            player_health_max: 100,
            score: 0,
            entity_count: 0,
            game_tick: 0,
            fps: 0.0,
            messages_per_second: 0.0,
        };
        assert!(critical.is_critical_health());
        assert!(!fine.is_critical_health());
    }

    #[test]
    fn render_frame_produces_correct_call_sequence() {
        let mut r = TestRenderer::new();
        let hud = HudData {
            player_health: 75,
            player_health_max: 100,
            score: 500,
            entity_count: 3,
            game_tick: 10,
            fps: 60.0,
            messages_per_second: 30.0,
        };
        render_frame(&mut r, &hud);
        assert!(r.has_call(&DrawCall::Clear));
        assert!(r.has_call(&DrawCall::Score(500)));
        assert!(r.has_call(&DrawCall::EntityCount(3)));
        assert!(r.has_call(&DrawCall::Present));
        assert_eq!(r.frame_count(), 1);
    }

    // --- MessageBridge ---
    #[test]
    fn bridge_processes_state_update() {
        let mut bridge = MessageBridge::new();
        let json = r#"{"StateUpdate":{"tick":5,"entity_count":3,"player_health":80,"player_health_max":100,"player_score":2000}}"#;
        let hud = bridge.process(json).unwrap();
        assert_eq!(hud.game_tick, 5);
        assert_eq!(hud.player_health, 80);
        assert_eq!(hud.score, 2000);
        assert_eq!(bridge.messages_processed, 1);
        assert_eq!(bridge.last_tick, 5);
    }

    #[test]
    fn bridge_rejects_invalid_json() {
        let mut bridge = MessageBridge::new();
        let result = bridge.process("not json at all");
        assert!(result.is_none());
        assert_eq!(bridge.messages_processed, 0);
    }

    // Falsification: bridge must reflect different ticks from different messages
    #[test]
    fn bridge_last_tick_reflects_message_content() {
        let mut bridge = MessageBridge::new();
        let json1 = r#"{"StateUpdate":{"tick":10,"entity_count":1,"player_health":null,"player_health_max":null,"player_score":0}}"#;
        let json2 = r#"{"StateUpdate":{"tick":99,"entity_count":1,"player_health":null,"player_health_max":null,"player_score":0}}"#;
        bridge.process(json1);
        assert_eq!(bridge.last_tick, 10);
        bridge.process(json2);
        assert_eq!(bridge.last_tick, 99);
        assert_ne!(10u64, 99u64);
    }

    #[test]
    fn bridge_serializes_input_commands() {
        let bridge = MessageBridge::new();
        let json = bridge
            .serialize_to_game(&UiToGameMessage::Input(InputCommand::Move {
                entity: 1,
                dx: 3.0,
                dy: -2.0,
            }))
            .unwrap();
        assert!(json.contains("Move"));
        let json2 = bridge
            .serialize_to_game(&UiToGameMessage::Pause)
            .unwrap();
        assert!(json2.contains("Pause"));
        assert_ne!(json, json2, "different messages must produce different JSON");
    }

    // Proptest: HUD invariants
    proptest! {
        #[test]
        fn health_percentage_always_in_range(hp in 0u32..10000, max in 1u32..10000) {
            let hud = HudData {
                player_health: hp.min(max),
                player_health_max: max,
                score: 0,
                entity_count: 0,
                game_tick: 0,
                fps: 0.0,
                messages_per_second: 0.0,
            };
            prop_assert!(hud.health_percentage() >= 0.0);
            prop_assert!(hud.health_percentage() <= 1.0);
        }

        #[test]
        fn render_frame_always_calls_clear_and_present(
            hp in 0u32..100,
            score in 0u64..u64::MAX,
            entities in 0usize..1000,
        ) {
            let mut r = TestRenderer::new();
            let hud = HudData {
                player_health: hp,
                player_health_max: 100,
                score,
                entity_count: entities,
                game_tick: 0,
                fps: 0.0,
                messages_per_second: 0.0,
            };
            render_frame(&mut r, &hud);
            prop_assert!(r.has_call(&DrawCall::Clear), "must always clear before drawing");
            prop_assert!(r.has_call(&DrawCall::Present), "must always present after drawing");
        }

        #[test]
        fn bridge_message_count_monotonically_increases(n in 1usize..20) {
            let mut bridge = MessageBridge::new();
            for i in 0..n {
                let json = format!(
                    r#"{{"StateUpdate":{{"tick":{},"entity_count":0,"player_health":null,"player_health_max":null,"player_score":0}}}}"#,
                    i
                );
                bridge.process(&json);
            }
            prop_assert_eq!(bridge.messages_processed, n as u64);
        }
    }
}
