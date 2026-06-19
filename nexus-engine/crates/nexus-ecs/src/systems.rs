use crate::components::*;
use hecs::{Entity, World};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use nexus_types::{AttackDir, GundamSeries, MagicType};

    // ── system_tick_ai ────────────────────────────────────────────────────────

    #[test]
    fn tick_ai_increments_turns_in_behavior() {
        let mut world = World::new();
        world.spawn((AiState {
            current_behavior: AiBehavior::Idle,
            turns_in_behavior: 0,
            announced_attack: None,
            phase: 1,
        },));
        system_tick_ai(&mut world);
        let (_, ai) = world.query_mut::<(Entity, &AiState)>().into_iter().next().unwrap();
        assert_eq!(ai.turns_in_behavior, 1);
    }

    #[test]
    fn tick_ai_transitions_idle_to_attacking_after_3_turns() {
        let mut world = World::new();
        world.spawn((AiState {
            current_behavior: AiBehavior::Idle,
            turns_in_behavior: 2, // one more tick → threshold 3
            announced_attack: None,
            phase: 1,
        },));
        system_tick_ai(&mut world);
        let (_, ai) = world.query_mut::<(Entity, &AiState)>().into_iter().next().unwrap();
        assert_eq!(ai.current_behavior, AiBehavior::Attacking);
        assert_eq!(ai.turns_in_behavior, 0, "counter must reset on transition");
    }

    #[test]
    fn tick_ai_does_not_change_non_idle_behavior() {
        let mut world = World::new();
        world.spawn((AiState {
            current_behavior: AiBehavior::Attacking,
            turns_in_behavior: 10,
            announced_attack: None,
            phase: 1,
        },));
        system_tick_ai(&mut world);
        let (_, ai) = world.query_mut::<(Entity, &AiState)>().into_iter().next().unwrap();
        assert_eq!(ai.current_behavior, AiBehavior::Attacking);
    }

    // ── system_tick_trans_am ──────────────────────────────────────────────────

    #[test]
    fn tick_trans_am_decrements_remaining_turns() {
        let mut world = World::new();
        world.spawn((MobileSuit {
            suit_id: "RX-93".into(),
            series: GundamSeries::UniversalCentury,
            special_ability: SpecialAbility::FinFunnels,
            is_trans_am_active: true,
            trans_am_turns_remaining: 3,
            nt_d_active: false,
        },));
        system_tick_trans_am(&mut world);
        let (_, suit) = world.query_mut::<(Entity, &MobileSuit)>().into_iter().next().unwrap();
        assert_eq!(suit.trans_am_turns_remaining, 2);
    }

    #[test]
    fn tick_trans_am_deactivates_when_turns_reach_zero() {
        let mut world = World::new();
        world.spawn((MobileSuit {
            suit_id: "RX-93".into(),
            series: GundamSeries::UniversalCentury,
            special_ability: SpecialAbility::FinFunnels,
            is_trans_am_active: true,
            trans_am_turns_remaining: 0,
            nt_d_active: false,
        },));
        system_tick_trans_am(&mut world);
        let (_, suit) = world.query_mut::<(Entity, &MobileSuit)>().into_iter().next().unwrap();
        assert!(!suit.is_trans_am_active);
    }

    #[test]
    fn tick_trans_am_ignores_inactive_suits() {
        let mut world = World::new();
        world.spawn((MobileSuit {
            suit_id: "RX-78".into(),
            series: GundamSeries::UniversalCentury,
            special_ability: SpecialAbility::None,
            is_trans_am_active: false,
            trans_am_turns_remaining: 5,
            nt_d_active: false,
        },));
        system_tick_trans_am(&mut world);
        let (_, suit) = world.query_mut::<(Entity, &MobileSuit)>().into_iter().next().unwrap();
        // turns should not be decremented when not active
        assert_eq!(suit.trans_am_turns_remaining, 5);
    }

    // ── system_tick_combos ────────────────────────────────────────────────────

    #[test]
    fn tick_combos_increments_idle_turns() {
        let mut world = World::new();
        world.spawn((ComboState { depth: 2, idle_turns: 0, max_depth: 5 },));
        system_tick_combos(&mut world, 3);
        let (_, combo) = world.query_mut::<(Entity, &ComboState)>().into_iter().next().unwrap();
        assert_eq!(combo.idle_turns, 1);
    }

    #[test]
    fn tick_combos_resets_depth_when_threshold_exceeded() {
        let mut world = World::new();
        world.spawn((ComboState { depth: 3, idle_turns: 3, max_depth: 5 },));
        system_tick_combos(&mut world, 3);
        let (_, combo) = world.query_mut::<(Entity, &ComboState)>().into_iter().next().unwrap();
        assert_eq!(combo.depth, 0, "depth must reset when idle_turns > threshold");
    }

    #[test]
    fn tick_combos_does_not_reset_before_threshold() {
        let mut world = World::new();
        world.spawn((ComboState { depth: 3, idle_turns: 2, max_depth: 5 },));
        system_tick_combos(&mut world, 3);
        let (_, combo) = world.query_mut::<(Entity, &ComboState)>().into_iter().next().unwrap();
        assert_eq!(combo.depth, 3, "depth must survive when still within threshold");
    }

    // ── system_move_projectiles ───────────────────────────────────────────────

    #[test]
    fn move_projectiles_updates_position_by_velocity_times_dt() {
        let mut world = World::new();
        world.spawn((
            Position { x: 0.0, y: 0.0, z: 0.0 },
            Velocity { dx: 10.0, dy: 5.0, dz: 0.0 },
        ));
        system_move_projectiles(&mut world, 0.1);
        let (_, pos) = world.query_mut::<(Entity, &Position)>().into_iter().next().unwrap();
        assert!((pos.x - 1.0).abs() < 0.001);
        assert!((pos.y - 0.5).abs() < 0.001);
    }

    // ── system_tick_projectiles ───────────────────────────────────────────────

    #[test]
    fn tick_projectiles_decrements_lifetime() {
        let mut world = World::new();
        world.spawn((Projectile { owner_id: 1, damage: 25.0, lifetime_remaining: 2.0, magic_type: MagicType::Fire },));
        system_tick_projectiles(&mut world, 0.5);
        let (_, proj) = world.query_mut::<(Entity, &Projectile)>().into_iter().next().unwrap();
        assert!((proj.lifetime_remaining - 1.5).abs() < 0.001);
    }

    #[test]
    fn tick_projectiles_despawns_expired_entities() {
        let mut world = World::new();
        world.spawn((Projectile { owner_id: 1, damage: 25.0, lifetime_remaining: 0.1, magic_type: MagicType::Fire },));
        system_tick_projectiles(&mut world, 0.2); // 0.1 - 0.2 = -0.1 ≤ 0 → despawn
        assert_eq!(world.len(), 0);
    }
}
