use hecs::World;
use crate::systems::*;

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
