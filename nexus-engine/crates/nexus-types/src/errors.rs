/// Domain-level type validation errors.
use thiserror::Error;

/// Errors arising from invalid primitive type construction.
#[derive(Debug, Error)]
pub enum TypeError {
    /// Health values must be non-negative.
    #[error("health cannot be negative: got {0}")]
    NegativeHealth(f32),

    /// Gold addition would overflow a u32.
    #[error("gold overflow: adding {added} to {current} exceeds u32::MAX")]
    GoldOverflow { current: u32, added: u32 },

    /// Time dilation factor must be within the legal gameplay range.
    #[error("invalid time dilation {0}: must be in range 0.1..=3.0")]
    InvalidTimeDilation(f32),
}

/// Errors arising from illegal game-state transitions or missing data.
#[derive(Debug, Error)]
pub enum GameError {
    /// An unrecognised attack direction was supplied.
    #[error("invalid attack direction")]
    InvalidAttackDir,

    /// An operation requiring a present enemy was performed with none.
    #[error("no enemy present")]
    NoEnemy,

    /// A combat-only operation was invoked outside combat.
    #[error("not in combat")]
    NotInCombat,

    /// The player's gold balance is below the required amount.
    #[error("insufficient gold: need {need}, have {have}")]
    InsufficientGold { need: u32, have: u32 },

    /// The player's inventory has no free slot for the new item.
    #[error("inventory full")]
    InventoryFull,

    /// The requested item does not exist in the current scope.
    #[error("item not found: {0}")]
    ItemNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TypeError display ─────────────────────────────────────────────────────

    #[test]
    fn negative_health_display_includes_value() {
        let e = TypeError::NegativeHealth(-5.0);
        assert!(format!("{e}").contains("-5"));
    }

    #[test]
    fn gold_overflow_display_includes_both_amounts() {
        let e = TypeError::GoldOverflow { current: 100, added: u32::MAX };
        let s = format!("{e}");
        assert!(s.contains("100"), "should mention current");
        assert!(s.contains("overflow"), "should mention overflow");
    }

    #[test]
    fn invalid_time_dilation_display_includes_value() {
        let e = TypeError::InvalidTimeDilation(99.9);
        assert!(format!("{e}").contains("99.9") || format!("{e}").contains("99"));
    }

    #[test]
    fn type_errors_are_debug() {
        let _ = format!("{:?}", TypeError::NegativeHealth(-1.0));
        let _ = format!("{:?}", TypeError::GoldOverflow { current: 0, added: 0 });
        let _ = format!("{:?}", TypeError::InvalidTimeDilation(0.0));
    }

    // ── GameError display ─────────────────────────────────────────────────────

    #[test]
    fn invalid_attack_dir_display() {
        assert!(format!("{}", GameError::InvalidAttackDir).contains("invalid attack"));
    }

    #[test]
    fn no_enemy_display() {
        assert!(format!("{}", GameError::NoEnemy).contains("enemy"));
    }

    #[test]
    fn not_in_combat_display() {
        assert!(format!("{}", GameError::NotInCombat).contains("combat"));
    }

    #[test]
    fn insufficient_gold_display_shows_amounts() {
        let e = GameError::InsufficientGold { need: 500, have: 300 };
        let s = format!("{e}");
        assert!(s.contains("500"), "need amount missing");
        assert!(s.contains("300"), "have amount missing");
    }

    #[test]
    fn inventory_full_display() {
        assert!(format!("{}", GameError::InventoryFull).contains("full"));
    }

    #[test]
    fn item_not_found_includes_name() {
        let e = GameError::ItemNotFound("Beam Saber".to_string());
        assert!(format!("{e}").contains("Beam Saber"));
    }

    #[test]
    fn game_errors_are_debug() {
        let _ = format!("{:?}", GameError::InvalidAttackDir);
        let _ = format!("{:?}", GameError::NoEnemy);
        let _ = format!("{:?}", GameError::InsufficientGold { need: 1, have: 0 });
    }
}
