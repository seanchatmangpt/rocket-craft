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
        let dmg = world.get_attack(attacker).map(|a| a.damage).unwrap_or(0);
        if dmg == 0 {
            return 0;
        }
        if let Some(hp) = world.get_health_mut(target) {
            hp.apply_damage(dmg);
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
