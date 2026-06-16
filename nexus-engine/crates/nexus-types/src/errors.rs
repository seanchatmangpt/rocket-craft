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
