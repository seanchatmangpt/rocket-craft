//! `nexus-types` — foundational primitive types for the Gundam Nexus Rust formal model.
//!
//! Every other crate in the `nexus-engine` workspace depends on this crate.
//! It deliberately has no dependencies on the rest of the workspace so that
//! it can be compiled first and in isolation.
//!
//! # Module layout
//!
//! | Module      | Purpose                                                   |
//! |-------------|-----------------------------------------------------------|
//! | [`units`]   | Phantom-typed numeric units (`Hp`, `Gold`, `Damage`, …)  |
//! | [`math`]    | 3-D math type aliases and `Transform`                    |
//! | [`states`]  | Typestate markers and runtime enums                       |
//! | [`ids`]     | Strongly-typed entity IDs                                 |
//! | [`errors`]  | Domain error types (`TypeError`, `GameError`)            |

pub mod errors;
pub mod ids;
pub mod math;
pub mod states;
pub mod units;
pub mod tps;

pub use tps::{
    StateVector, Part, μ, const_validate_part, MechAssembly, TpsValidationError, TpsAssemblyReceipt,
};


// Convenience re-exports so downstream crates can write `nexus_types::Hp` etc.
pub use errors::{GameError, TypeError};
pub use ids::{AuctionId, EnemyId, ItemId, PlayerId, SessionId, TransactionId, TypedId};
pub use math::{Aabb, Mat4, Quat, Transform, Vec2, Vec3};
pub use states::{
    AttackDir, AuctionClosed, Authenticated, BidAccepted, BidRejected, Connecting, CombatStateMarker,
    Dead, Disconnected, Dodging, EconomyStateMarker, GachaRarity, Idle, InLobby, InMatch,
    MagicType, ParryOutcome, PendingBid, Rarity, Series, GundamSeries, SessionStateMarker, Spectating, Stunned,
    TitanType, Attacking, Parrying,
};
pub use units::{
    Armor, ComboMultiplier, Damage, Gold, Hp, Mana, TimeDilation, Typed, Xp,
};
