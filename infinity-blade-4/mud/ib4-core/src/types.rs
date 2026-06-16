use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttackDir {
    Overhead,
    Left,
    Right,
}

impl fmt::Display for AttackDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttackDir::Overhead => write!(f, "Overhead"),
            AttackDir::Left => write!(f, "Left"),
            AttackDir::Right => write!(f, "Right"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CombatState {
    Idle,
    Attacking,
    Parrying,
    Dodging,
    Stunned,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MagicType {
    Fire,
    Lightning,
    Ice,
    Dark,
    Light,
}

impl fmt::Display for MagicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagicType::Fire => write!(f, "Fire"),
            MagicType::Lightning => write!(f, "Lightning"),
            MagicType::Ice => write!(f, "Ice"),
            MagicType::Dark => write!(f, "Dark"),
            MagicType::Light => write!(f, "Light"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TitanType {
    Warrior,
    Mage,
    Archer,
    Heavy,
    GodKing,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Infinity,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StatusEffect {
    Burn,
    Stun,
    Freeze,
    Dark,
    Healed,
}
