use crate::components::*;
use hecs::{Entity, World as HecsWorld};

pub struct GameWorld {
    pub inner: HecsWorld,
}

impl GameWorld {
    pub fn new() -> Self {
        GameWorld {
            inner: HecsWorld::new(),
        }
    }

    /// Spawn a player entity with all required components
    pub fn spawn_player(&mut self, player_id: u64, name: &str, hp: f32, gold: u32) -> Entity {
        // hecs supports tuples up to 15 components — split spawn + insert
        let entity = self.inner.spawn((
            PlayerId(player_id),
            Name(name.to_string()),
            Health::new(hp),
            Mana {
                current: 100.0,
                max: 100.0,
            },
            AttackPower {
                base: 30.0,
                bonus: 0.0,
            },
            Defense {
                base: 10.0,
                bonus: 0.0,
            },
            ComboState {
                depth: 0,
                idle_turns: 0,
                max_depth: 5,
            },
            QipScars { stacks: 0 },
            Gold(gold),
            Level(1),
            Experience(0),
            Bloodline(0),
            PlayerControlled,
            Position::default(),
            Rotation::identity(),
        ));
        self.inner
            .insert(entity, (Scale::default(), Visible(true)))
            .ok();
        entity
    }

    /// Spawn an enemy titan entity
    pub fn spawn_enemy(
        &mut self,
        enemy_id: u64,
        name: &str,
        hp: f32,
        attack: f32,
        phase: u8,
    ) -> Entity {
        self.inner.spawn((
            EnemyId(enemy_id),
            Name(name.to_string()),
            Health::new(hp),
            AttackPower {
                base: attack,
                bonus: 0.0,
            },
            Defense {
                base: 5.0,
                bonus: 0.0,
            },
            AiControlled,
            AiState {
                current_behavior: AiBehavior::Idle,
                turns_in_behavior: 0,
                announced_attack: None,
                phase,
            },
            Position::default(),
            Rotation::identity(),
            Scale::default(),
            Visible(true),
            Collider { radius: 1.5 },
        ))
    }

    /// Spawn a projectile
    pub fn spawn_projectile(
        &mut self,
        owner_id: u64,
        pos: Position,
        vel: Velocity,
        dmg: f32,
        magic: MagicType,
    ) -> Entity {
        self.inner.spawn((
            Projectile {
                owner_id,
                damage: dmg,
                lifetime_remaining: 3.0,
                magic_type: magic,
            },
            pos,
            vel,
            Collider { radius: 0.2 },
            Visible(true),
        ))
    }

    /// Spawn a mobile suit overlay on an existing player entity
    pub fn attach_suit(
        &mut self,
        entity: Entity,
        suit_id: String,
        series: GundamSeries,
        ability: SpecialAbility,
    ) {
        self.inner
            .insert(
                entity,
                (MobileSuit {
                    suit_id,
                    series,
                    special_ability: ability,
                    is_trans_am_active: false,
                    trans_am_turns_remaining: 0,
                    nt_d_active: false,
                },),
            )
            .ok();
    }

    pub fn entity_count(&self) -> u32 {
        self.inner.len()
    }

    /// Despawn dead entities
    pub fn despawn_dead(&mut self) -> Vec<Entity> {
        let dead: Vec<Entity> = self
            .inner
            .query::<(Entity, &Health)>()
            .iter()
            .filter(|(_entity, health)| !health.is_alive())
            .map(|(entity, _health)| entity)
            .collect();
        for e in &dead {
            self.inner.despawn(*e).ok();
        }
        dead
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── spawn_player ──────────────────────────────────────────────────────────

    #[test]
    fn spawn_player_increments_entity_count() {
        let mut world = GameWorld::new();
        assert_eq!(world.entity_count(), 0);
        world.spawn_player(1, "Heero", 500.0, 1000);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn two_spawn_player_calls_produce_distinct_entities() {
        let mut world = GameWorld::new();
        let e1 = world.spawn_player(1, "Heero", 500.0, 1000);
        let e2 = world.spawn_player(2, "Zechs", 500.0, 1000);
        assert_ne!(e1, e2);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn spawned_player_has_correct_gold() {
        let mut world = GameWorld::new();
        let e = world.spawn_player(1, "Heero", 500.0, 999);
        let gold = world.inner.get::<&Gold>(e).unwrap();
        assert_eq!(gold.0, 999);
    }

    #[test]
    fn spawned_player_has_full_health() {
        let mut world = GameWorld::new();
        let e = world.spawn_player(1, "Heero", 500.0, 0);
        let hp = world.inner.get::<&Health>(e).unwrap();
        assert!(hp.is_alive());
        assert_eq!(hp.current, 500.0);
    }

    // ── spawn_enemy ───────────────────────────────────────────────────────────

    #[test]
    fn spawn_enemy_increments_entity_count() {
        let mut world = GameWorld::new();
        world.spawn_enemy(99, "Titan", 300.0, 30.0, 1);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn player_and_enemy_coexist() {
        let mut world = GameWorld::new();
        world.spawn_player(1, "Heero", 500.0, 0);
        world.spawn_enemy(1, "Titan", 300.0, 30.0, 1);
        assert_eq!(world.entity_count(), 2);
    }

    // ── despawn_dead ──────────────────────────────────────────────────────────

    #[test]
    fn despawn_dead_removes_entities_with_zero_hp() {
        let mut world = GameWorld::new();
        let e = world.spawn_player(1, "Heero", 1.0, 0);
        // Kill the entity by setting hp to 0 directly
        world.inner.get::<&mut Health>(e).unwrap().take_damage(999.0);
        let dead = world.despawn_dead();
        assert_eq!(dead.len(), 1);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn despawn_dead_leaves_alive_entities_untouched() {
        let mut world = GameWorld::new();
        world.spawn_player(1, "Heero", 500.0, 0);
        world.spawn_player(2, "Zechs", 500.0, 0);
        let dead = world.despawn_dead();
        assert!(dead.is_empty());
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn despawn_dead_removes_only_dead_entities() {
        let mut world = GameWorld::new();
        let dead_player = world.spawn_player(1, "Heero", 1.0, 0);
        world.spawn_player(2, "Zechs", 500.0, 0);
        world.inner.get::<&mut Health>(dead_player).unwrap().take_damage(999.0);
        let dead = world.despawn_dead();
        assert_eq!(dead.len(), 1);
        assert_eq!(world.entity_count(), 1, "only the dead entity is removed");
    }

    // ── entity_count ─────────────────────────────────────────────────────────

    #[test]
    fn new_world_has_zero_entities() {
        assert_eq!(GameWorld::new().entity_count(), 0);
    }
}
