use hecs::World;
use crate::components::*;

/// System: apply time dilation to all projectile lifetimes
pub fn system_tick_projectiles(world: &mut World, dt: f32) {
    let mut expired = Vec::new();
    for (entity, (proj, _pos, _vel)) in
        world.query_mut::<(&mut Projectile, &Position, &Velocity)>()
    {
        proj.lifetime_remaining -= dt;
        if proj.lifetime_remaining <= 0.0 {
            expired.push(entity);
        }
    }
    for e in expired {
        world.despawn(e).ok();
    }
}

/// System: move projectiles by velocity
pub fn system_move_projectiles(world: &mut World, dt: f32) {
    for (_entity, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.x += vel.dx * dt;
        pos.y += vel.dy * dt;
        pos.z += vel.dz * dt;
    }
}

/// System: tick AI behavior — increment turns_in_behavior, occasionally switch
pub fn system_tick_ai(world: &mut World) {
    for (_entity, ai) in world.query_mut::<&mut AiState>() {
        ai.turns_in_behavior += 1;
        // Simple behavior switch after 3 turns idle
        if ai.current_behavior == AiBehavior::Idle && ai.turns_in_behavior >= 3 {
            ai.current_behavior = AiBehavior::Attacking;
            ai.turns_in_behavior = 0;
        }
    }
}

/// System: tick Trans-Am duration
pub fn system_tick_trans_am(world: &mut World) {
    for (_entity, suit) in world.query_mut::<&mut MobileSuit>() {
        if suit.is_trans_am_active {
            if suit.trans_am_turns_remaining == 0 {
                suit.is_trans_am_active = false;
            } else {
                suit.trans_am_turns_remaining -= 1;
            }
        }
    }
}

/// System: check combo idle timeout (reset_threshold turns of non-attack = reset)
pub fn system_tick_combos(world: &mut World, reset_threshold: u32) {
    for (_entity, combo) in world.query_mut::<&mut ComboState>() {
        combo.idle_turns += 1;
        if combo.idle_turns >= reset_threshold {
            combo.depth = 0;
            combo.idle_turns = 0;
        }
    }
}

/// Query helper: get all alive enemy entities and their health
pub fn query_alive_enemies(world: &World) -> Vec<(hecs::Entity, f32, f32)> {
    world
        .query::<(&EnemyId, &Health)>()
        .iter()
        .filter(|(_, (_, h))| h.is_alive())
        .map(|(e, (_, h))| (e, h.current, h.max))
        .collect()
}

/// Query helper: get all player positions
pub fn query_player_positions(world: &World) -> Vec<(u64, Position)> {
    world
        .query::<(&PlayerId, &Position)>()
        .iter()
        .map(|(_, (id, pos))| (id.0, *pos))
        .collect()
}
