//! # MUD Core Types Module
//!
//! Provides the primary enumerations and structures representing combat directions,
//! states, magic types, and stats used by Siris character sessions.

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

/// A player stat that can be allocated via a point-buy command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Health,
    Attack,
    Defense,
    Magic,
}

#[derive(Debug)]
pub struct StatParseError(pub String);

impl std::fmt::Display for StatParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown stat '{}': expected health, attack, defense, or magic",
            self.0
        )
    }
}

impl std::error::Error for StatParseError {}

impl std::str::FromStr for Stat {
    type Err = StatParseError;
    /// Parse a string slice into a `Stat` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ib4_core::types::Stat;
    ///
    /// assert_eq!(Stat::from_str("hp").unwrap(), Stat::Health);
    /// assert_eq!(Stat::from_str("atk").unwrap(), Stat::Attack);
    /// assert!(Stat::from_str("invalid").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "health" | "hp" => Ok(Stat::Health),
            "attack" | "atk" => Ok(Stat::Attack),
            "defense" | "def" => Ok(Stat::Defense),
            "magic" | "mag" => Ok(Stat::Magic),
            other => Err(StatParseError(other.to_string())),
        }
    }
}
