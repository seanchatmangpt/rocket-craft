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

/// A bid has been submitted but not yet resolved.
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
pub enum Series {
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
