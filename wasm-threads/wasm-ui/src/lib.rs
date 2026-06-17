pub mod hud;
pub mod message_bridge;
pub mod renderer;
pub mod ui_state;

pub use hud::{render_frame, HudData};
pub use message_bridge::{GameToUiMessage, InputCommand, MessageBridge, UiToGameMessage};
pub use renderer::{DrawCall, Renderer, TestRenderer};
#[cfg(target_arch = "wasm32")]
pub use renderer::CanvasRenderer;
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
        self.frame += 1;
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

    pub fn send_ping(&self, seq: u32) -> String {
        self.bridge.send_ping(seq)
    }
}
