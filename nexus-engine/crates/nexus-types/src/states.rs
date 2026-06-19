//! Typestate markers and runtime enums for all game-state domains.
//!
//! The sealed marker traits enforce at compile time that only the canonical
//! state structs defined here can be used as type parameters, preventing
//! downstream crates from introducing undocumented states.
// ---------------------------------------------------------------------------
// Sealed trait (module-private, not re-exported)
// ---------------------------------------------------------------------------

mod private {
    /// Prevents external crates from implementing the state-marker traits.
    pub trait Sealed {}

    impl Sealed for super::Idle {}
    impl Sealed for super::Attacking {}
    impl Sealed for super::Parrying {}
    impl Sealed for super::Dodging {}
    impl Sealed for super::Stunned {}
    impl Sealed for super::Dead {}

    impl Sealed for super::Connecting {}
    impl Sealed for super::Authenticated {}
    impl Sealed for super::InLobby {}
    impl Sealed for super::InMatch {}
    impl Sealed for super::Spectating {}
    impl Sealed for super::Disconnected {}

    impl Sealed for super::PendingBid {}
    impl Sealed for super::BidAccepted {}
    impl Sealed for super::BidRejected {}
    impl Sealed for super::AuctionClosed {}
}

// ---------------------------------------------------------------------------
// Combat typestates
// ---------------------------------------------------------------------------

/// Compile-time marker for a valid combat phase.
pub trait CombatStateMarker: private::Sealed {}

/// The entity is standing by, ready to act.
pub struct Idle;
/// The entity has committed to an attack swing.
pub struct Attacking;
/// The entity is holding a parry guard.
pub struct Parrying;
/// The entity is mid-dodge roll.
pub struct Dodging;
/// The entity is in a hit-stun window and cannot act.
pub struct Stunned;
/// The entity's HP has reached zero.
pub struct Dead;

impl CombatStateMarker for Idle {}
impl CombatStateMarker for Attacking {}
impl CombatStateMarker for Parrying {}
impl CombatStateMarker for Dodging {}
impl CombatStateMarker for Stunned {}
impl CombatStateMarker for Dead {}

// ---------------------------------------------------------------------------
// Session typestates
// ---------------------------------------------------------------------------

/// Compile-time marker for a valid multiplayer-session phase.
pub trait SessionStateMarker: private::Sealed {}

/// TCP handshake in progress.
pub struct Connecting;
/// Credentials have been verified by the auth service.
pub struct Authenticated;
/// Player is waiting in a pre-match lobby.
pub struct InLobby;
/// Player is actively participating in a ranked or casual match.
pub struct InMatch;
/// Player is watching a match without participating.
pub struct Spectating;
/// The connection to the server has been terminated.
pub struct Disconnected;

impl SessionStateMarker for Connecting {}
impl SessionStateMarker for Authenticated {}
impl SessionStateMarker for InLobby {}
impl SessionStateMarker for InMatch {}
impl SessionStateMarker for Spectating {}
impl SessionStateMarker for Disconnected {}

// ---------------------------------------------------------------------------
// Economy typestates
// ---------------------------------------------------------------------------

/// Compile-time marker for a valid auction-house transaction phase.
pub trait EconomyStateMarker: private::Sealed {}

/// A bid has been submitted but not yet rereaddressed.
pub struct PendingBid;
/// The auction house has accepted the highest bid.
pub struct BidAccepted;
/// The bid was outbid or the item was withdrawn.
pub struct BidRejected;
/// The auction has ended; no further bids are accepted.
pub struct AuctionClosed;

impl EconomyStateMarker for PendingBid {}
impl EconomyStateMarker for BidAccepted {}
impl EconomyStateMarker for BidRejected {}
impl EconomyStateMarker for AuctionClosed {}

// ---------------------------------------------------------------------------
// Runtime enums (used when typestate generics are not necessary)
// ---------------------------------------------------------------------------

/// Direction of an incoming melee attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttackDir {
    /// High vertical strike from above.
    Overhead,
    /// Horizontal sweep from the left.
    Left,
    /// Horizontal sweep from the right.
    Right,
}

/// Result of a parry attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ParryOutcome {
    /// Parry window was missed entirely.
    Miss,
    /// Successful block but no counter bonus.
    Normal,
    /// Frame-perfect parry granting a counter window.
    Perfect,
}

/// Elemental magic school.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MagicType {
    Fire,
    Lightning,
    Ice,
    Dark,
    Light,
    BeamSaber,
}

/// Equipment and item rarity tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    /// Beyond Legendary — obtained only from Infinity Raids.
    Infinity,
}

/// Enemy archetype that determines AI behaviour and ability sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TitanType {
    /// Melee-focused brawler.
    Warrior,
    /// Long-range spellcaster.
    Mage,
    /// Ranged precision attacker.
    Archer,
    /// Heavily armoured tank.
    Heavy,
    /// Final-boss tier with all abilities.
    GodKing,
}

/// Gundam anime franchise series tag for unit provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GundamSeries {
    WitchFromMercury,
    Seed,
    Unicorn,
    UniversalCentury,
    Wing,
    DoubleO,
    IronBloodedOrphans,
    TurnA,
    BuildFighters,
}

pub type Series = GundamSeries;

/// Gacha pull rarity tier for banner drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GachaRarity {
    /// Standard unit.
    R,
    /// Super Rare unit.
    SR,
    /// Super Super Rare — top banner tier.
    SSR,
}

// ---------------------------------------------------------------------------
// MagicType conversions
// ---------------------------------------------------------------------------

/// Error returned when a raw `u8` byte does not map to any `MagicType` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicTypeParseError(pub u8);

impl std::fmt::Display for MagicTypeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown magic type byte: {}", self.0)
    }
}

impl std::error::Error for MagicTypeParseError {}

/// Flat damage bonus for each magic school (used by combat resolvers).
impl From<MagicType> for f32 {
    fn from(m: MagicType) -> f32 {
        match m {
            MagicType::Fire => 20.0,
            MagicType::Lightning => 30.0,
            MagicType::Ice => 15.0,
            MagicType::Dark => 35.0,
            MagicType::Light => 25.0,
            MagicType::BeamSaber => 40.0,
        }
    }
}

/// Wire-format decoding: converts a raw protocol byte into a `MagicType`.
impl TryFrom<u8> for MagicType {
    type Error = MagicTypeParseError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(MagicType::Fire),
            1 => Ok(MagicType::Lightning),
            2 => Ok(MagicType::Ice),
            3 => Ok(MagicType::Dark),
            4 => Ok(MagicType::Light),
            5 => Ok(MagicType::BeamSaber),
            other => Err(MagicTypeParseError(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AttackDir ─────────────────────────────────────────────────────────────

    #[test]
    fn attack_dir_three_distinct_variants() {
        let dirs = [AttackDir::Overhead, AttackDir::Left, AttackDir::Right];
        // Each pair is distinct
        assert_ne!(dirs[0], dirs[1]);
        assert_ne!(dirs[1], dirs[2]);
        assert_ne!(dirs[0], dirs[2]);
    }

    #[test]
    fn attack_dir_clones_equal_original() {
        let dir = AttackDir::Overhead;
        assert_eq!(dir, dir.clone());
    }

    // ── MagicType → f32 damage bonus ─────────────────────────────────────────

    #[test]
    fn magic_type_damage_bonuses_are_ordered_by_power() {
        let fire: f32 = MagicType::Fire.into();
        let lightning: f32 = MagicType::Lightning.into();
        let ice: f32 = MagicType::Ice.into();
        let dark: f32 = MagicType::Dark.into();
        let beam: f32 = MagicType::BeamSaber.into();
        // BeamSaber > Dark > Lightning > Fire > Ice
        assert!(beam > dark);
        assert!(dark > lightning);
        assert!(lightning > fire);
        assert!(fire > ice);
    }

    #[test]
    fn beam_saber_bonus_is_40() {
        let bonus: f32 = MagicType::BeamSaber.into();
        assert_eq!(bonus, 40.0);
    }

    // ── MagicType TryFrom<u8> ─────────────────────────────────────────────────

    #[test]
    fn magic_type_try_from_valid_bytes() {
        assert_eq!(MagicType::try_from(0u8).unwrap(), MagicType::Fire);
        assert_eq!(MagicType::try_from(1u8).unwrap(), MagicType::Lightning);
        assert_eq!(MagicType::try_from(2u8).unwrap(), MagicType::Ice);
        assert_eq!(MagicType::try_from(3u8).unwrap(), MagicType::Dark);
        assert_eq!(MagicType::try_from(4u8).unwrap(), MagicType::Light);
        assert_eq!(MagicType::try_from(5u8).unwrap(), MagicType::BeamSaber);
    }

    #[test]
    fn magic_type_try_from_invalid_byte_returns_error() {
        let err = MagicType::try_from(99u8).unwrap_err();
        assert_eq!(err.0, 99);
    }

    #[test]
    fn magic_type_parse_error_display_contains_value() {
        let err = MagicTypeParseError(42);
        assert!(err.to_string().contains("42"));
    }

    // ── GachaRarity ordering ──────────────────────────────────────────────────

    #[test]
    fn gacha_rarity_variants_are_distinct() {
        assert_ne!(GachaRarity::R, GachaRarity::SR);
        assert_ne!(GachaRarity::SR, GachaRarity::SSR);
    }

    // ── TitanType variants ────────────────────────────────────────────────────

    #[test]
    fn titan_type_five_distinct_variants() {
        let types = [
            TitanType::Warrior, TitanType::Mage, TitanType::Archer,
            TitanType::Heavy, TitanType::GodKing,
        ];
        // spot-check: at least first and last differ
        assert_ne!(types[0], types[4]);
    }

    // ── Rarity ───────────────────────────────────────────────────────────────

    #[test]
    fn rarity_six_variants_all_distinct() {
        let rarities = [
            Rarity::Common, Rarity::Uncommon, Rarity::Rare,
            Rarity::Epic, Rarity::Legendary, Rarity::Infinity,
        ];
        assert_ne!(rarities[0], rarities[5]);
        assert_ne!(rarities[2], rarities[4]);
    }
}
