pub use nexus_types::{AttackDir, GundamSeries, MagicType};
use serde::{Deserialize, Serialize};

// === Transform Components ===
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub qx: f32,
    pub qy: f32,
    pub qz: f32,
    pub qw: f32,
}

impl Rotation {
    pub fn identity() -> Self {
        Rotation {
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
            qw: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Scale {
    fn default() -> Self {
        Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

// === Combat Components ===
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Health { current: max, max }
    }
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }
    pub fn hp_percent(&self) -> f32 {
        self.current / self.max
    }
    pub fn take_damage(&mut self, dmg: f32) {
        self.current = (self.current - dmg).max(0.0);
    }
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackPower {
    pub base: f32,
    pub bonus: f32,
}

impl AttackPower {
    pub fn total(&self) -> f32 {
        self.base + self.bonus
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Defense {
    pub base: f32,
    pub bonus: f32,
}

impl Defense {
    pub fn total(&self) -> f32 {
        self.base + self.bonus
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ComboState {
    pub depth: u32,
    pub idle_turns: u32,
    pub max_depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QipScars {
    pub stacks: u32,
}

// === Identity Components ===
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyId(pub u64);

// === Mobile Suit Components ===
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MobileSuit {
    pub suit_id: String,
    pub series: GundamSeries,
    pub special_ability: SpecialAbility,
    pub is_trans_am_active: bool,
    pub trans_am_turns_remaining: u32,
    pub nt_d_active: bool,
}

// GundamSeries imported from nexus_types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialAbility {
    FinFunnels,         // Nu Gundam: auto-parry assists
    NtD,                // Unicorn: NT-D burst 30s
    TransAm,            // 00: combo overdrive at depth 4+
    ZeroSystem,         // Wing: berserker mode
    AlayaVijnana,       // IBO: precision parry -10HP
    StrikerPack,        // SEED: mid-fight equipment swap
    GundFormat,         // WfM: HP-cost mechanics
    MoonlightButterfly, // Turn A: reality reset
    None,
}

// === AI / Behavior Components ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiControlled; // marker component

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerControlled; // marker component

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiState {
    pub current_behavior: AiBehavior,
    pub turns_in_behavior: u32,
    pub announced_attack: Option<AttackDir>,
    pub phase: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiBehavior {
    Idle,
    Attacking,
    Defending,
    SpecialAttack,
    Fleeing,
}

// === Economy Components ===
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Gold(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Level(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Experience(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bloodline(pub u32);

// === Visual/Rendering Components ===
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshRef(pub String); // asset path to mesh

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialRef(pub String); // asset path to material

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Visible(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    pub radius: f32,
} // sphere collider (simplified)

// === Projectile Components ===
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    pub owner_id: u64,
    pub damage: f32,
    pub lifetime_remaining: f32,
    pub magic_type: MagicType,
}

// MagicType imported from nexus_types
