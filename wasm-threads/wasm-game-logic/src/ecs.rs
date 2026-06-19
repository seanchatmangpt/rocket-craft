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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Health ────────────────────────────────────────────────────────────────

    #[test]
    fn health_new_sets_current_to_max() {
        let h = Health::new(100);
        assert_eq!(h.current, 100);
        assert_eq!(h.max, 100);
    }

    #[test]
    fn health_apply_damage_subtracts() {
        let mut h = Health::new(100);
        h.apply_damage(30);
        assert_eq!(h.current, 70);
    }

    #[test]
    fn health_apply_damage_saturates_at_zero() {
        let mut h = Health::new(50);
        h.apply_damage(999);
        assert_eq!(h.current, 0);
        assert!(h.is_dead());
    }

    #[test]
    fn health_heal_clamps_to_max() {
        let mut h = Health::new(100);
        h.apply_damage(40);
        h.heal(200);
        assert_eq!(h.current, 100);
    }

    #[test]
    fn health_percentage_full_is_one() {
        let h = Health::new(100);
        assert!((h.percentage() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn health_percentage_zero_max_is_zero() {
        let h = Health { current: 0, max: 0 };
        assert_eq!(h.percentage(), 0.0);
    }

    // ── World spawn/despawn ───────────────────────────────────────────────────

    #[test]
    fn world_spawn_increments_entity_count() {
        let mut w = World::new();
        let e = w.spawn();
        assert_eq!(w.entity_count(), 1);
        assert!(w.is_alive(e));
    }

    #[test]
    fn world_spawn_ids_are_unique() {
        let mut w = World::new();
        let a = w.spawn();
        let b = w.spawn();
        assert_ne!(a, b);
    }

    #[test]
    fn world_despawn_removes_entity() {
        let mut w = World::new();
        let e = w.spawn();
        w.despawn(e);
        assert!(!w.is_alive(e));
        assert_eq!(w.entity_count(), 0);
    }

    #[test]
    fn despawn_removes_components() {
        let mut w = World::new();
        let e = w.spawn();
        w.add_position(e, Position { x: 1.0, y: 2.0 });
        w.despawn(e);
        assert!(w.get_position(e).is_none());
    }

    // ── Component add/get ─────────────────────────────────────────────────────

    #[test]
    fn add_and_get_position() {
        let mut w = World::new();
        let e = w.spawn();
        w.add_position(e, Position { x: 3.0, y: 4.0 });
        let pos = w.get_position(e).unwrap();
        assert!((pos.x - 3.0).abs() < 1e-6);
        assert!((pos.y - 4.0).abs() < 1e-6);
    }

    #[test]
    fn get_position_missing_returns_none() {
        let w = World::new();
        let e = Entity(42);
        assert!(w.get_position(e).is_none());
    }

    #[test]
    fn add_and_get_health() {
        let mut w = World::new();
        let e = w.spawn();
        w.add_health(e, Health::new(80));
        let h = w.get_health(e).unwrap();
        assert_eq!(h.max, 80);
    }

    #[test]
    fn entities_with_position_iterates_all() {
        let mut w = World::new();
        let e1 = w.spawn();
        let e2 = w.spawn();
        w.add_position(e1, Position { x: 0.0, y: 0.0 });
        w.add_position(e2, Position { x: 1.0, y: 1.0 });
        let count = w.entities_with_position().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn world_default_equals_new() {
        let w: World = Default::default();
        assert_eq!(w.entity_count(), 0);
    }
}
