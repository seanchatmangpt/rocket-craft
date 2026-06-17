pub mod ecs;
pub mod protocol;
pub mod state;
pub mod systems;

pub use ecs::*;
pub use protocol::*;
pub use state::*;
pub use systems::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// JS-callable game logic runner for the WASM worker.
///
/// Manages the game loop via JSON serialisation so the generic typestate
/// `GameState<S>` doesn't have to be exposed across the WASM ABI.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct GameLogicWorker {
    tick: u64,
    elapsed_ms: u64,
    running: bool,
    entity_count: usize,
    player_health: u32,
    player_health_max: u32,
    player_score: u64,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl GameLogicWorker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tick: 0,
            elapsed_ms: 0,
            running: false,
            entity_count: 0,
            player_health: 100,
            player_health_max: 100,
            player_score: 0,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn tick_js(&mut self, delta_ms: f64) -> String {
        if !self.running {
            return "{}".to_string();
        }
        self.elapsed_ms += delta_ms as u64;
        self.tick += 1;
        self.player_score = self.tick * 10;

        serde_json::to_string(&GameToUiMessage::StateUpdate {
            tick: self.tick,
            entity_count: self.entity_count,
            player_health: Some(self.player_health),
            player_health_max: Some(self.player_health_max),
            player_score: self.player_score,
        })
        .unwrap_or_default()
    }

    pub fn handle_input_js(&mut self, input_json: &str) -> bool {
        serde_json::from_str::<UiToGameMessage>(input_json).is_ok()
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

