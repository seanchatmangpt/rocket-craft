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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── ECS ──────────────────────────────────────────────────────────────────

    #[test]
    fn spawn_increases_entity_count() {
        let mut world = World::new();
        assert_eq!(world.entity_count(), 0);
        world.spawn();
        assert_eq!(world.entity_count(), 1);
        world.spawn();
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn despawn_decreases_entity_count() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e);
        assert_eq!(world.entity_count(), 0);
        assert!(!world.is_alive(e));
    }

    #[test]
    fn component_roundtrip() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_health(e, Health::new(100));
        let hp = world.get_health(e).unwrap();
        assert_eq!(hp.current, 100);
        assert_eq!(hp.max, 100);
    }

    #[test]
    fn despawned_entity_components_removed() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_health(e, Health::new(50));
        world.add_position(e, Position { x: 1.0, y: 2.0 });
        world.despawn(e);
        assert!(world.get_health(e).is_none());
        assert!(world.get_position(e).is_none());
    }

    // ── Falsification: damage must depend on attack stat ──────────────────────

    #[test]
    fn damage_scales_with_attack_stat() {
        let mut world = World::new();
        let attacker = world.spawn();
        let target1 = world.spawn();
        let target2 = world.spawn();
        world.add_health(target1, Health::new(1000));
        world.add_health(target2, Health::new(1000));

        world.add_attack(
            attacker,
            Attack {
                damage: 10,
                range: 1.0,
                cooldown_ms: 0,
            },
        );
        let dmg1 = CombatSystem::apply_damage(&mut world, attacker, target1);

        *world.get_attack_mut(attacker).unwrap() = Attack {
            damage: 50,
            range: 1.0,
            cooldown_ms: 0,
        };
        let dmg2 = CombatSystem::apply_damage(&mut world, attacker, target2);

        assert_ne!(dmg1, dmg2, "damage must depend on attack stat");
        assert_eq!(dmg1, 10);
        assert_eq!(dmg2, 50);
    }

    #[test]
    fn zero_attack_deals_zero_damage() {
        let mut world = World::new();
        let attacker = world.spawn();
        let target = world.spawn();
        world.add_health(target, Health::new(100));
        world.add_attack(
            attacker,
            Attack {
                damage: 0,
                range: 1.0,
                cooldown_ms: 0,
            },
        );
        let dmg = CombatSystem::apply_damage(&mut world, attacker, target);
        assert_eq!(dmg, 0);
        assert_eq!(world.get_health(target).unwrap().current, 100);
    }

    #[test]
    fn no_attack_component_deals_zero_damage() {
        let mut world = World::new();
        let attacker = world.spawn(); // no Attack component
        let target = world.spawn();
        world.add_health(target, Health::new(100));
        let dmg = CombatSystem::apply_damage(&mut world, attacker, target);
        assert_eq!(dmg, 0);
    }

    // ── Health clamping ───────────────────────────────────────────────────────

    #[test]
    fn health_cannot_go_below_zero() {
        let mut hp = Health::new(10);
        hp.apply_damage(1000);
        assert_eq!(hp.current, 0);
        assert!(hp.is_dead());
    }

    #[test]
    fn health_cannot_exceed_max() {
        let mut hp = Health::new(100);
        hp.heal(10000);
        assert_eq!(hp.current, 100);
    }

    #[test]
    fn health_percentage_at_full() {
        let hp = Health::new(100);
        assert!((hp.percentage() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn health_percentage_at_zero() {
        let mut hp = Health::new(100);
        hp.apply_damage(100);
        assert!((hp.percentage() - 0.0).abs() < f32::EPSILON);
    }

    // ── CombatSystem cleanup ──────────────────────────────────────────────────

    #[test]
    fn dead_entities_are_removed_by_cleanup() {
        let mut world = World::new();
        let attacker = world.spawn();
        let target = world.spawn();
        world.add_health(target, Health::new(1)); // 1 HP
        world.add_attack(
            attacker,
            Attack {
                damage: 10,
                range: 1.0,
                cooldown_ms: 0,
            },
        );
        CombatSystem::apply_damage(&mut world, attacker, target);
        assert!(world.get_health(target).unwrap().is_dead());
        CombatSystem::run_cleanup(&mut world);
        assert!(!world.is_alive(target));
        assert_eq!(world.entity_count(), 1); // only attacker
    }

    // ── Physics ───────────────────────────────────────────────────────────────

    #[test]
    fn physics_moves_entity_by_velocity() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_position(e, Position { x: 0.0, y: 0.0 });
        world.add_velocity(e, Velocity { dx: 10.0, dy: 0.0 });
        PhysicsSystem::run(&mut world, 1000); // 1 second
        let pos = world.get_position(e).unwrap();
        assert!((pos.x - 10.0).abs() < 0.001);
        assert!((pos.y - 0.0).abs() < 0.001);
    }

    /// Falsification: position must depend on velocity, not be constant.
    #[test]
    fn position_depends_on_velocity() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.add_position(e1, Position { x: 0.0, y: 0.0 });
        world.add_position(e2, Position { x: 0.0, y: 0.0 });
        world.add_velocity(e1, Velocity { dx: 5.0, dy: 0.0 });
        world.add_velocity(e2, Velocity { dx: 20.0, dy: 0.0 });
        PhysicsSystem::run(&mut world, 1000);
        let x1 = world.get_position(e1).unwrap().x;
        let x2 = world.get_position(e2).unwrap().x;
        assert_ne!(x1, x2, "different velocities must produce different positions");
    }

    #[test]
    fn physics_entity_without_velocity_stays_still() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_position(e, Position { x: 3.0, y: 7.0 });
        // No velocity component
        PhysicsSystem::run(&mut world, 1000);
        let pos = world.get_position(e).unwrap();
        assert!((pos.x - 3.0).abs() < 0.001);
        assert!((pos.y - 7.0).abs() < 0.001);
    }

    // ── InputSystem ───────────────────────────────────────────────────────────

    #[test]
    fn input_move_updates_velocity() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_velocity(e, Velocity { dx: 0.0, dy: 0.0 });
        InputSystem::process(
            &mut world,
            PlayerInput::Move {
                entity: e.0,
                dx: 3.5,
                dy: -1.2,
            },
        );
        let vel = world.get_velocity(e).unwrap();
        assert!((vel.dx - 3.5).abs() < 0.001);
        assert!((vel.dy - (-1.2)).abs() < 0.001);
    }

    #[test]
    fn input_attack_reduces_target_health() {
        let mut world = World::new();
        let attacker = world.spawn();
        let target = world.spawn();
        world.add_attack(
            attacker,
            Attack {
                damage: 25,
                range: 1.0,
                cooldown_ms: 0,
            },
        );
        world.add_health(target, Health::new(100));
        InputSystem::process(
            &mut world,
            PlayerInput::Attack {
                attacker: attacker.0,
                target: target.0,
            },
        );
        assert_eq!(world.get_health(target).unwrap().current, 75);
    }

    #[test]
    fn input_use_item_1_heals_entity() {
        let mut world = World::new();
        let e = world.spawn();
        let mut hp = Health::new(100);
        hp.apply_damage(50);
        world.add_health(e, hp);
        InputSystem::process(
            &mut world,
            PlayerInput::UseItem {
                entity: e.0,
                item_id: 1,
            },
        );
        assert_eq!(world.get_health(e).unwrap().current, 60);
    }

    #[test]
    fn input_use_item_unknown_id_does_nothing() {
        let mut world = World::new();
        let e = world.spawn();
        let mut hp = Health::new(100);
        hp.apply_damage(50);
        world.add_health(e, hp);
        InputSystem::process(
            &mut world,
            PlayerInput::UseItem {
                entity: e.0,
                item_id: 99,
            },
        );
        // unknown item — no change
        assert_eq!(world.get_health(e).unwrap().current, 50);
    }

    // ── Game state transitions ────────────────────────────────────────────────

    #[test]
    fn game_state_start_to_running() {
        let state = GameState::<Initializing>::new();
        let running = state.start();
        assert!(running.is_running());
    }

    #[test]
    fn game_state_tick_increments() {
        let mut state = GameState::<Initializing>::new().start();
        assert_eq!(state.tick, 0);
        state.tick(16);
        assert_eq!(state.tick, 1);
        state.tick(16);
        assert_eq!(state.tick, 2);
    }

    #[test]
    fn game_state_elapsed_accumulates() {
        let mut state = GameState::<Initializing>::new().start();
        state.tick(100);
        state.tick(200);
        assert_eq!(state.elapsed_ms, 300);
    }

    #[test]
    fn game_state_pause_resume_cycle() {
        let state = GameState::<Initializing>::new().start();
        let paused = state.pause();
        assert!(paused.is_paused());
        let running = paused.resume();
        assert!(running.is_running());
    }

    #[test]
    fn game_state_running_to_game_over() {
        let state = GameState::<Initializing>::new().start();
        let over = state.game_over();
        assert_eq!(over.total_ticks(), 0);
    }

    #[test]
    fn game_state_game_over_restart() {
        let over = GameState::<Initializing>::new().start().game_over();
        let fresh = over.restart();
        // restart produces a brand-new Initializing state
        let running = fresh.start();
        assert_eq!(running.tick, 0);
    }

    #[test]
    fn game_over_winner_score_from_survivors() {
        let state = GameState::<Initializing>::new();
        let mut running = state.start();
        let e = running.world.spawn();
        running.world.add_health(e, Health::new(100));
        running.world.add_player(
            e,
            Player {
                name: "Hero".into(),
                score: 9999,
            },
        );
        let over = running.game_over();
        assert_eq!(over.winner_score(), 9999);
    }

    // ── Protocol serialisation ────────────────────────────────────────────────

    #[test]
    fn game_to_ui_message_roundtrips_json() {
        let msg = GameToUiMessage::StateUpdate {
            tick: 42,
            entity_count: 5,
            player_health: Some(75),
            player_health_max: Some(100),
            player_score: 1000,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: GameToUiMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            GameToUiMessage::StateUpdate {
                tick,
                player_health,
                ..
            } => {
                assert_eq!(tick, 42);
                assert_eq!(player_health, Some(75));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn game_over_message_roundtrips_json() {
        let msg = GameToUiMessage::GameOver {
            winner_score: 42000,
            total_ticks: 1800,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: GameToUiMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            GameToUiMessage::GameOver {
                winner_score,
                total_ticks,
            } => {
                assert_eq!(winner_score, 42000);
                assert_eq!(total_ticks, 1800);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn ui_to_game_ping_roundtrips_json() {
        let msg = UiToGameMessage::Ping { seq: 7 };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: UiToGameMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            UiToGameMessage::Ping { seq } => assert_eq!(seq, 7),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn ui_to_game_input_roundtrips_json() {
        let msg = UiToGameMessage::Input(PlayerInput::Move {
            entity: 3,
            dx: 1.5,
            dy: -0.5,
        });
        let json = msg.to_json();
        let decoded = UiToGameMessage::from_json(&json).unwrap();
        match decoded {
            UiToGameMessage::Input(PlayerInput::Move { entity, dx, dy }) => {
                assert_eq!(entity, 3);
                assert!((dx - 1.5).abs() < 0.001);
                assert!((dy - (-0.5)).abs() < 0.001);
            }
            _ => panic!("wrong variant"),
        }
    }

    // ── Property-based tests ──────────────────────────────────────────────────

    proptest! {
        #[test]
        fn health_current_never_exceeds_max(max in 1u32..10000, damage in 0u32..10000) {
            let mut hp = Health::new(max);
            hp.apply_damage(damage);
            prop_assert!(hp.current <= hp.max);
            prop_assert!(hp.percentage() >= 0.0);
            prop_assert!(hp.percentage() <= 1.0);
        }

        #[test]
        fn health_is_dead_iff_current_is_zero(max in 1u32..10000, damage in 0u32..50000) {
            let mut hp = Health::new(max);
            hp.apply_damage(damage);
            let dead = hp.is_dead();
            prop_assert_eq!(dead, hp.current == 0);
        }

        #[test]
        fn entity_count_never_negative_after_operations(
            spawns in 1usize..20,
            despawns in 0usize..20,
        ) {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..spawns).map(|_| world.spawn()).collect();
            let to_despawn = despawns.min(spawns);
            for &e in entities.iter().take(to_despawn) {
                world.despawn(e);
            }
            prop_assert_eq!(world.entity_count(), spawns - to_despawn);
        }

        #[test]
        fn physics_position_is_deterministic(
            x0 in -1000.0f32..1000.0,
            y0 in -1000.0f32..1000.0,
            vx in -100.0f32..100.0,
            vy in -100.0f32..100.0,
            dt in 1u64..5000,
        ) {
            let mut world1 = World::new();
            let mut world2 = World::new();
            let e1 = world1.spawn();
            let e2 = world2.spawn();
            world1.add_position(e1, Position { x: x0, y: y0 });
            world2.add_position(e2, Position { x: x0, y: y0 });
            world1.add_velocity(e1, Velocity { dx: vx, dy: vy });
            world2.add_velocity(e2, Velocity { dx: vx, dy: vy });
            PhysicsSystem::run(&mut world1, dt);
            PhysicsSystem::run(&mut world2, dt);
            let p1 = world1.get_position(e1).unwrap();
            let p2 = world2.get_position(e2).unwrap();
            prop_assert!((p1.x - p2.x).abs() < 0.001);
            prop_assert!((p1.y - p2.y).abs() < 0.001);
        }

        #[test]
        fn damage_applied_reduces_health_by_exact_amount(
            max_hp in 1u32..10000,
            damage in 1u32..1000,
        ) {
            // Only interesting when damage < max_hp (otherwise clamped).
            prop_assume!(damage < max_hp);
            let mut hp = Health::new(max_hp);
            hp.apply_damage(damage);
            prop_assert_eq!(hp.current, max_hp - damage);
        }

        #[test]
        fn heal_after_damage_bounded_by_max(
            max_hp in 1u32..10000,
            damage in 1u32..10000,
            heal in 0u32..10000,
        ) {
            let mut hp = Health::new(max_hp);
            hp.apply_damage(damage);
            hp.heal(heal);
            prop_assert!(hp.current <= hp.max);
        }
    }
}
