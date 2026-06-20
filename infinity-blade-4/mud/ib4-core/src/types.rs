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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ── AttackDir ─────────────────────────────────────────────────────────────

    #[test]
    fn attack_dir_display() {
        assert_eq!(AttackDir::Overhead.to_string(), "Overhead");
        assert_eq!(AttackDir::Left.to_string(), "Left");
        assert_eq!(AttackDir::Right.to_string(), "Right");
    }

    #[test]
    fn attack_dir_clone_and_eq() {
        let d = AttackDir::Left;
        assert_eq!(d.clone(), AttackDir::Left);
        assert_ne!(d, AttackDir::Right);
    }

    // ── CombatState ───────────────────────────────────────────────────────────

    #[test]
    fn combat_states_are_distinct() {
        assert_ne!(CombatState::Idle, CombatState::Attacking);
        assert_ne!(CombatState::Dead, CombatState::Stunned);
    }

    // ── MagicType ─────────────────────────────────────────────────────────────

    #[test]
    fn magic_type_display_all_variants() {
        assert_eq!(MagicType::Fire.to_string(), "Fire");
        assert_eq!(MagicType::Lightning.to_string(), "Lightning");
        assert_eq!(MagicType::Ice.to_string(), "Ice");
        assert_eq!(MagicType::Dark.to_string(), "Dark");
        assert_eq!(MagicType::Light.to_string(), "Light");
    }

    // ── Stat::from_str ────────────────────────────────────────────────────────

    #[test]
    fn stat_from_str_accepts_canonical_names() {
        assert_eq!(Stat::from_str("health").unwrap(), Stat::Health);
        assert_eq!(Stat::from_str("attack").unwrap(), Stat::Attack);
        assert_eq!(Stat::from_str("defense").unwrap(), Stat::Defense);
        assert_eq!(Stat::from_str("magic").unwrap(), Stat::Magic);
    }

    #[test]
    fn stat_from_str_accepts_aliases() {
        assert_eq!(Stat::from_str("hp").unwrap(), Stat::Health);
        assert_eq!(Stat::from_str("atk").unwrap(), Stat::Attack);
        assert_eq!(Stat::from_str("def").unwrap(), Stat::Defense);
        assert_eq!(Stat::from_str("mag").unwrap(), Stat::Magic);
    }

    #[test]
    fn stat_from_str_is_case_insensitive() {
        assert_eq!(Stat::from_str("HEALTH").unwrap(), Stat::Health);
        assert_eq!(Stat::from_str("ATK").unwrap(), Stat::Attack);
    }

    #[test]
    fn stat_from_str_returns_error_for_unknown() {
        assert!(Stat::from_str("invalid").is_err());
        assert!(Stat::from_str("").is_err());
        assert!(Stat::from_str("str").is_err());
    }

    #[test]
    fn stat_parse_error_display_includes_input() {
        let e = Stat::from_str("badstat").unwrap_err();
        assert!(e.to_string().contains("badstat"));
    }

    // ── Rarity / TitanType / StatusEffect ────────────────────────────────────

    #[test]
    fn rarity_variants_are_distinct() {
        assert_ne!(Rarity::Common, Rarity::Legendary);
        assert_ne!(Rarity::Rare, Rarity::Infinity);
    }

    #[test]
    fn titan_type_clone() {
        let t = TitanType::GodKing;
        assert_eq!(t.clone(), TitanType::GodKing);
    }

    #[test]
    fn status_effect_clone_and_eq() {
        assert_eq!(StatusEffect::Burn.clone(), StatusEffect::Burn);
        assert_ne!(StatusEffect::Stun, StatusEffect::Freeze);
    }
}
