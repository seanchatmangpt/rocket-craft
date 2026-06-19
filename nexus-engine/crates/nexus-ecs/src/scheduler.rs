use crate::systems::*;
use hecs::World;

pub struct SystemScheduler {
    pub dt: f32,
    pub combo_reset_threshold: u32,
}

impl SystemScheduler {
    pub fn new(dt: f32) -> Self {
        SystemScheduler {
            dt,
            combo_reset_threshold: 2,
        }
    }

    /// Run one full game tick: all systems in order
    pub fn tick(&self, world: &mut World) {
        system_tick_ai(world);
        system_tick_trans_am(world);
        system_tick_combos(world, self.combo_reset_threshold);
        system_move_projectiles(world, self.dt);
        system_tick_projectiles(world, self.dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use hecs::{Entity, World};
    use nexus_types::MagicType;

    #[test]
    fn scheduler_default_combo_reset_threshold_is_2() {
        let s = SystemScheduler::new(0.016);
        assert_eq!(s.combo_reset_threshold, 2);
    }

    #[test]
    fn tick_runs_all_systems_without_panic_on_empty_world() {
        let s = SystemScheduler::new(0.016);
        let mut world = World::new();
        s.tick(&mut world); // must not panic
    }

    #[test]
    fn tick_integrates_projectile_position_by_dt() {
        let dt = 0.1_f32;
        let s = SystemScheduler::new(dt);
        let mut world = World::new();
        world.spawn((
            Position { x: 0.0, y: 0.0, z: 0.0 },
            Velocity { dx: 10.0, dy: 0.0, dz: 0.0 },
        ));
        s.tick(&mut world);
        let (_, pos) = world.query_mut::<(Entity, &Position)>().into_iter().next().unwrap();
        assert!((pos.x - 1.0).abs() < 0.001, "x must advance by v*dt = 1.0, got {}", pos.x);
    }

    #[test]
    fn tick_despawns_expired_projectile() {
        let s = SystemScheduler::new(1.0); // large dt
        let mut world = World::new();
        world.spawn((Projectile { owner_id: 1, damage: 10.0, lifetime_remaining: 0.5, magic_type: MagicType::Fire },));
        s.tick(&mut world);
        assert_eq!(world.len(), 0, "expired projectile must be despawned");
    }

    #[test]
    fn tick_resets_idle_combo_after_threshold() {
        let s = SystemScheduler::new(0.016);
        let mut world = World::new();
        // combo_reset_threshold = 2; after 3 ticks idle_turns exceeds threshold
        world.spawn((ComboState { depth: 5, idle_turns: 2, max_depth: 5 },));
        s.tick(&mut world); // idle_turns becomes 3 > threshold 2 → depth resets
        let (_, c) = world.query_mut::<(Entity, &ComboState)>().into_iter().next().unwrap();
        assert_eq!(c.depth, 0, "combo depth must reset when idle_turns exceeds threshold");
    }
}
