use std::collections::{HashMap, HashSet};

/// A lightweight entity handle — just a newtype over u32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Entity(pub u32);

/// Marker trait for ECS components.
pub trait Component: 'static + Send + Sync {}

// ── Concrete components ──────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}
impl Component for Velocity {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}
impl Component for Health {}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn apply_damage(&mut self, dmg: u32) {
        self.current = self.current.saturating_sub(dmg);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current == 0
    }

    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Attack {
    pub damage: u32,
    pub range: f32,
    pub cooldown_ms: u64,
}
impl Component for Attack {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub name: String,
    pub score: u64,
}
impl Component for Player {}

// ── World ────────────────────────────────────────────────────────────────────

/// The ECS world — owns all entities and their component storage.
pub struct World {
    next_entity: u32,
    positions: HashMap<Entity, Position>,
    velocities: HashMap<Entity, Velocity>,
    healths: HashMap<Entity, Health>,
    attacks: HashMap<Entity, Attack>,
    players: HashMap<Entity, Player>,
    alive: HashSet<Entity>,
    pub current_time_ms: u64,
    pub attack_cooldowns: HashMap<Entity, u64>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            positions: HashMap::new(),
            velocities: HashMap::new(),
            healths: HashMap::new(),
            attacks: HashMap::new(),
            players: HashMap::new(),
            alive: HashSet::new(),
            current_time_ms: 0,
            attack_cooldowns: HashMap::new(),
        }
    }

    // ── Entity lifecycle ─────────────────────────────────────────────────────

    pub fn spawn(&mut self) -> Entity {
        let e = Entity(self.next_entity);
        self.next_entity += 1;
        self.alive.insert(e);
        e
    }

    pub fn despawn(&mut self, e: Entity) {
        self.alive.remove(&e);
        self.positions.remove(&e);
        self.velocities.remove(&e);
        self.healths.remove(&e);
        self.attacks.remove(&e);
        self.players.remove(&e);
        self.attack_cooldowns.remove(&e);
    }

    pub fn entity_count(&self) -> usize {
        self.alive.len()
    }

    pub fn is_alive(&self, e: Entity) -> bool {
        self.alive.contains(&e)
    }

    // ── Position ─────────────────────────────────────────────────────────────

    pub fn add_position(&mut self, e: Entity, c: Position) {
        self.positions.insert(e, c);
    }

    pub fn get_position(&self, e: Entity) -> Option<&Position> {
        self.positions.get(&e)
    }

    pub fn get_position_mut(&mut self, e: Entity) -> Option<&mut Position> {
        self.positions.get_mut(&e)
    }

    // ── Velocity ─────────────────────────────────────────────────────────────

    pub fn add_velocity(&mut self, e: Entity, c: Velocity) {
        self.velocities.insert(e, c);
    }

    pub fn get_velocity(&self, e: Entity) -> Option<&Velocity> {
        self.velocities.get(&e)
    }

    pub fn get_velocity_mut(&mut self, e: Entity) -> Option<&mut Velocity> {
        self.velocities.get_mut(&e)
    }

    // ── Health ───────────────────────────────────────────────────────────────

    pub fn add_health(&mut self, e: Entity, c: Health) {
        self.healths.insert(e, c);
    }

    pub fn get_health(&self, e: Entity) -> Option<&Health> {
        self.healths.get(&e)
    }

    pub fn get_health_mut(&mut self, e: Entity) -> Option<&mut Health> {
        self.healths.get_mut(&e)
    }

    // ── Attack ───────────────────────────────────────────────────────────────

    pub fn add_attack(&mut self, e: Entity, c: Attack) {
        self.attacks.insert(e, c);
    }

    pub fn get_attack(&self, e: Entity) -> Option<&Attack> {
        self.attacks.get(&e)
    }

    pub fn get_attack_mut(&mut self, e: Entity) -> Option<&mut Attack> {
        self.attacks.get_mut(&e)
    }

    // ── Player ───────────────────────────────────────────────────────────────

    pub fn add_player(&mut self, e: Entity, c: Player) {
        self.players.insert(e, c);
    }

    pub fn get_player(&self, e: Entity) -> Option<&Player> {
        self.players.get(&e)
    }

    pub fn get_player_mut(&mut self, e: Entity) -> Option<&mut Player> {
        self.players.get_mut(&e)
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    pub fn entities_with_position(&self) -> impl Iterator<Item = Entity> + '_ {
        self.positions
            .keys()
            .filter(|e| self.alive.contains(e))
            .copied()
    }

    pub fn entities_with_health(&self) -> impl Iterator<Item = Entity> + '_ {
        self.healths
            .keys()
            .filter(|e| self.alive.contains(e))
            .copied()
    }

    pub fn entities_with_velocity(&self) -> impl Iterator<Item = Entity> + '_ {
        self.velocities
            .keys()
            .filter(|e| self.alive.contains(e))
            .copied()
    }

    pub fn entities_alive(&self) -> impl Iterator<Item = Entity> + '_ {
        self.alive.iter().copied()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
