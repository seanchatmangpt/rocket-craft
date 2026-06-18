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
    world: World,
    player_entity: Option<Entity>,
    tick: u64,
    elapsed_ms: u64,
    running: bool,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl GameLogicWorker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            world: World::new(),
            player_entity: None,
            tick: 0,
            elapsed_ms: 0,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        let pe = self.world.spawn();
        self.world.add_health(pe, Health::new(100));
        self.world.add_position(pe, Position { x: 0.0, y: 0.0 });
        self.world.add_velocity(pe, Velocity { dx: 0.0, dy: 0.0 });
        self.world.add_attack(pe, Attack { damage: 10, range: 50.0, cooldown_ms: 500 });
        self.world.add_player(pe, Player { name: "Player1".to_string(), score: 0 });
        self.player_entity = Some(pe);
    }

    pub fn tick_js(&mut self, delta_ms: f64) -> String {
        if !self.running {
            return "{}".to_string();
        }
        self.elapsed_ms += delta_ms as u64;
        self.world.current_time_ms = self.elapsed_ms;
        self.tick += 1;

        PhysicsSystem::run(&mut self.world, delta_ms as u64);
        CombatSystem::run_cleanup(&mut self.world);

        if let Some(pe) = self.player_entity {
            if self.world.is_alive(pe) {
                ScoreSystem::award(&mut self.world, pe, 10);
            }
        }

        let entity_count = self.world.entity_count();

        let (player_health, player_health_max, player_score) =
            match self.player_entity {
                Some(pe) if self.world.is_alive(pe) => {
                    let (h, hm) = self.world.get_health(pe)
                        .map(|h| (h.current, h.max))
                        .unwrap_or((0, 100));
                    let score = self.world.get_player(pe)
                        .map(|p| p.score)
                        .unwrap_or(0);
                    (Some(h), Some(hm), score)
                }
                _ => (Some(0), Some(100), 0),
            };

        serde_json::to_string(&GameToUiMessage::StateUpdate {
            tick: self.tick,
            entity_count,
            player_health,
            player_health_max,
            player_score,
        })
        .unwrap_or_default()
    }

    /// Returns empty string for most messages, JSON Pong for Ping.
    pub fn handle_input_js(&mut self, input_json: &str) -> String {
        let msg = match serde_json::from_str::<UiToGameMessage>(input_json) {
            Ok(m) => m,
            Err(_) => return String::new(),
        };
        match msg {
            UiToGameMessage::Pause => {
                self.running = false;
                String::new()
            }
            UiToGameMessage::Resume => {
                self.running = true;
                String::new()
            }
            UiToGameMessage::Restart => {
                *self = GameLogicWorker::new();
                self.start();
                String::new()
            }
            UiToGameMessage::Ping { seq } => {
                serde_json::to_string(&FromGameMessage::Pong { seq }).unwrap_or_default()
            }
            UiToGameMessage::Input(cmd) => {
                InputSystem::process(&mut self.world, cmd);
                String::new()
            }
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
