use hecs::{World as HecsWorld, Entity};
use crate::components::*;

pub struct GameWorld {
    pub inner: HecsWorld,
}

impl GameWorld {
    pub fn new() -> Self { GameWorld { inner: HecsWorld::new() } }

    /// Spawn a player entity with all required components
    pub fn spawn_player(&mut self, player_id: u64, name: &str, hp: f32, gold: u32) -> Entity {
        // hecs supports tuples up to 15 components — split spawn + insert
        let entity = self.inner.spawn((
            PlayerId(player_id),
            Name(name.to_string()),
            Health::new(hp),
            Mana { current: 100.0, max: 100.0 },
            AttackPower { base: 30.0, bonus: 0.0 },
            Defense { base: 10.0, bonus: 0.0 },
            ComboState { depth: 0, idle_turns: 0, max_depth: 5 },
            QipScars { stacks: 0 },
            Gold(gold),
            Level(1),
            Experience(0),
            Bloodline(0),
            PlayerControlled,
            Position::default(),
            Rotation::identity(),
        ));
        self.inner.insert(entity, (Scale::default(), Visible(true))).ok();
        entity
    }

    /// Spawn an enemy titan entity
    pub fn spawn_enemy(&mut self, enemy_id: u64, name: &str, hp: f32, attack: f32, phase: u8) -> Entity {
        self.inner.spawn((
            EnemyId(enemy_id),
            Name(name.to_string()),
            Health::new(hp),
            AttackPower { base: attack, bonus: 0.0 },
            Defense { base: 5.0, bonus: 0.0 },
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
