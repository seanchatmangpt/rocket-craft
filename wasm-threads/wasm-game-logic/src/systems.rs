use crate::ecs::{Entity, World};

// ── Physics ──────────────────────────────────────────────────────────────────

pub struct PhysicsSystem;

impl PhysicsSystem {
    /// Integrate all (position, velocity) pairs by `delta_ms` milliseconds.
    pub fn run(world: &mut World, delta_ms: u64) {
        let dt = delta_ms as f32 / 1000.0;
        // Collect first to avoid the borrow-checker complaining about split borrows.
        let entities: Vec<Entity> = world.entities_with_position().collect();
        for e in entities {
            if let Some(vel) = world.get_velocity(e).cloned() {
                if let Some(pos) = world.get_position_mut(e) {
                    pos.x += vel.dx * dt;
                    pos.y += vel.dy * dt;
                }
            }
        }
    }
}

// ── Combat ───────────────────────────────────────────────────────────────────

pub struct CombatSystem;

impl CombatSystem {
    /// Apply damage from `attacker` to `target`. Returns actual damage dealt.
    pub fn apply_damage(world: &mut World, attacker: Entity, target: Entity) -> u32 {
        let attack = match world.get_attack(attacker) {
            Some(a) => a.clone(),
            None => return 0,
        };
        let dmg = attack.damage;
        if dmg == 0 {
            return 0;
        }

        // Enforce range check (defaulting to 0.0 distance if position components are missing)
        if let (Some(ap), Some(tp)) = (world.get_position(attacker), world.get_position(target)) {
            let dx = ap.x - tp.x;
            let dy = ap.y - tp.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > attack.range {
                return 0;
            }
        }

        // Enforce cooldown check
        let current_time = world.current_time_ms;
        if let Some(&allowed_time) = world.attack_cooldowns.get(&attacker) {
            if current_time < allowed_time {
                return 0;
            }
        }

        if let Some(hp) = world.get_health_mut(target) {
            hp.apply_damage(dmg);
            if attack.cooldown_ms > 0 {
                world.attack_cooldowns.insert(attacker, current_time + attack.cooldown_ms);
            }
            dmg
        } else {
            0
        }
    }

    /// Remove entities whose health has hit zero.
    pub fn run_cleanup(world: &mut World) {
        let dead: Vec<Entity> = world
            .entities_with_health()
            .filter(|&e| world.get_health(e).map(|h| h.is_dead()).unwrap_or(false))
            .collect();
        for e in dead {
            world.despawn(e);
        }
    }
}

// ── Input ────────────────────────────────────────────────────────────────────

pub struct InputSystem;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputCommand {
    Move { entity: u32, dx: f32, dy: f32 },
    Attack { attacker: u32, target: u32 },
    UseItem { entity: u32, item_id: u32 },
}

impl InputSystem {
    pub fn process(world: &mut World, input: InputCommand) {
        match input {
            InputCommand::Move { entity, dx, dy } => {
                let e = Entity(entity);
                if let Some(vel) = world.get_velocity_mut(e) {
                    vel.dx = dx;
                    vel.dy = dy;
                }
            }
            InputCommand::Attack { attacker, target } => {
                CombatSystem::apply_damage(world, Entity(attacker), Entity(target));
            }
            InputCommand::UseItem { entity, item_id } => {
                // item_id == 1 → heal 10 HP
                if item_id == 1 {
                    if let Some(hp) = world.get_health_mut(Entity(entity)) {
                        hp.heal(10);
                    }
                }
            }
        }
    }
}

// ── Score ─────────────────────────────────────────────────────────────────────

pub struct ScoreSystem;

impl ScoreSystem {
    /// Award `points` to the player attached to `entity`.
    pub fn award(world: &mut World, entity: Entity, points: u64) {
        if let Some(player) = world.get_player_mut(entity) {
            player.score += points;
        }
    }

    /// Return the highest score among all alive players, or 0 if none.
    pub fn leader_score(world: &World) -> u64 {
        world
            .entities_alive()
            .filter_map(|e| world.get_player(e).map(|p| p.score))
            .max()
            .unwrap_or(0)
    }
}

// Re-export Health so tests can use it through systems
pub use crate::ecs::{Attack, Position};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{Health, Player, Velocity};

    fn make_world() -> World {
        World::new()
    }

    // ── PhysicsSystem ─────────────────────────────────────────────────────────

    #[test]
    fn physics_integrates_velocity_into_position() {
        let mut w = make_world();
        let e = w.spawn();
        w.add_position(e, Position { x: 0.0, y: 0.0 });
        w.add_velocity(e, Velocity { dx: 10.0, dy: 0.0 });
        // 1000ms = 1s → x should increase by 10.0
        PhysicsSystem::run(&mut w, 1000);
        let pos = w.get_position(e).unwrap();
        assert!((pos.x - 10.0).abs() < 1e-4, "x should be 10.0, got {}", pos.x);
        assert!(pos.y.abs() < 1e-4);
    }

    #[test]
    fn physics_no_velocity_leaves_position_unchanged() {
        let mut w = make_world();
        let e = w.spawn();
        w.add_position(e, Position { x: 5.0, y: 7.0 });
        // no velocity added
        PhysicsSystem::run(&mut w, 1000);
        let pos = w.get_position(e).unwrap();
        assert!((pos.x - 5.0).abs() < 1e-4);
    }

    // ── CombatSystem ─────────────────────────────────────────────────────────

    #[test]
    fn apply_damage_reduces_target_health() {
        let mut w = make_world();
        let attacker = w.spawn();
        let target = w.spawn();
        w.add_position(attacker, Position { x: 0.0, y: 0.0 });
        w.add_position(target, Position { x: 1.0, y: 0.0 });
        w.add_attack(attacker, Attack { damage: 25, range: 100.0, cooldown_ms: 0 });
        w.add_health(target, Health::new(100));

        let dealt = CombatSystem::apply_damage(&mut w, attacker, target);
        assert_eq!(dealt, 25);
        assert_eq!(w.get_health(target).unwrap().current, 75);
    }

    #[test]
    fn apply_damage_returns_zero_when_target_out_of_range() {
        let mut w = make_world();
        let attacker = w.spawn();
        let target = w.spawn();
        w.add_position(attacker, Position { x: 0.0, y: 0.0 });
        w.add_position(target, Position { x: 200.0, y: 0.0 });
        w.add_attack(attacker, Attack { damage: 10, range: 5.0, cooldown_ms: 0 });
        w.add_health(target, Health::new(100));

        let dealt = CombatSystem::apply_damage(&mut w, attacker, target);
        assert_eq!(dealt, 0, "out-of-range attack must deal 0 damage");
        assert_eq!(w.get_health(target).unwrap().current, 100);
    }

    #[test]
    fn apply_damage_returns_zero_with_no_attack_component() {
        let mut w = make_world();
        let attacker = w.spawn();
        let target = w.spawn();
        w.add_health(target, Health::new(50));
        assert_eq!(CombatSystem::apply_damage(&mut w, attacker, target), 0);
    }

    #[test]
    fn run_cleanup_removes_dead_entities() {
        let mut w = make_world();
        let e = w.spawn();
        let mut hp = Health::new(10);
        hp.apply_damage(10); // kill it
        w.add_health(e, hp);
        CombatSystem::run_cleanup(&mut w);
        assert!(!w.is_alive(e));
    }

    // ── InputSystem ───────────────────────────────────────────────────────────

    #[test]
    fn input_move_updates_velocity() {
        let mut w = make_world();
        let e = w.spawn();
        w.add_velocity(e, Velocity { dx: 0.0, dy: 0.0 });
        InputSystem::process(&mut w, InputCommand::Move { entity: e.0, dx: 3.0, dy: 4.0 });
        let vel = w.get_velocity(e).unwrap();
        assert!((vel.dx - 3.0).abs() < 1e-6);
        assert!((vel.dy - 4.0).abs() < 1e-6);
    }

    #[test]
    fn input_use_item_1_heals_entity() {
        let mut w = make_world();
        let e = w.spawn();
        let mut hp = Health::new(100);
        hp.apply_damage(30);
        w.add_health(e, hp);
        InputSystem::process(&mut w, InputCommand::UseItem { entity: e.0, item_id: 1 });
        assert_eq!(w.get_health(e).unwrap().current, 80);
    }

    // ── ScoreSystem ───────────────────────────────────────────────────────────

    #[test]
    fn award_adds_points_to_player_score() {
        let mut w = make_world();
        let e = w.spawn();
        w.add_player(e, Player { name: "p1".into(), score: 0 });
        ScoreSystem::award(&mut w, e, 500);
        assert_eq!(w.get_player(e).unwrap().score, 500);
    }

    #[test]
    fn leader_score_returns_max_among_players() {
        let mut w = make_world();
        let a = w.spawn();
        let b = w.spawn();
        w.add_health(a, Health::new(10));
        w.add_health(b, Health::new(10));
        w.add_player(a, Player { name: "a".into(), score: 100 });
        w.add_player(b, Player { name: "b".into(), score: 999 });
        assert_eq!(ScoreSystem::leader_score(&w), 999);
    }

    #[test]
    fn leader_score_is_zero_with_no_players() {
        let w = make_world();
        assert_eq!(ScoreSystem::leader_score(&w), 0);
    }
}
