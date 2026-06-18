use hecs::{World, Entity};
use crate::components::*;

pub fn system_tick_ai(world: &mut World) {
    for (_entity, ai) in world.query_mut::<(Entity, &mut AiState)>() {
        ai.turns_in_behavior += 1;
        if ai.current_behavior == AiBehavior::Idle && ai.turns_in_behavior >= 3 {
            ai.current_behavior = AiBehavior::Attacking;
            ai.turns_in_behavior = 0;
        }
    }
}

pub fn system_tick_trans_am(world: &mut World) {
    for (_entity, suit) in world.query_mut::<(Entity, &mut MobileSuit)>() {
        if suit.is_trans_am_active {
            if suit.trans_am_turns_remaining > 0 {
                suit.trans_am_turns_remaining -= 1;
            } else {
                suit.is_trans_am_active = false;
            }
        }
    }
}

pub fn system_tick_combos(world: &mut World, threshold: u32) {
    for (_entity, combo) in world.query_mut::<(Entity, &mut ComboState)>() {
        combo.idle_turns += 1;
        if combo.idle_turns > threshold {
            combo.depth = 0;
        }
    }
}

pub fn system_move_projectiles(world: &mut World, dt: f32) {
    for (_entity, pos, vel) in world.query_mut::<(Entity, &mut Position, &Velocity)>() {
        pos.x += vel.dx * dt;
        pos.y += vel.dy * dt;
        pos.z += vel.dz * dt;
    }
}

pub fn system_tick_projectiles(world: &mut World, dt: f32) {
    let mut to_despawn = Vec::new();
    for (entity, proj) in world.query_mut::<(Entity, &mut Projectile)>() {
        proj.lifetime_remaining -= dt;
        if proj.lifetime_remaining <= 0.0 {
            to_despawn.push(entity);
        }
    }
    for entity in to_despawn {
        world.despawn(entity).ok();
    }
}
