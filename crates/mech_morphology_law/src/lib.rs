//! # `mech_morphology_law`
//!
//! Pre-UE4 hero-asset **metric-morphology GRAPH LAW** — a pure, deterministic,
//! `no_std`-capable core that enforces ontology laws **110** (anatomy ordering +
//! per-part ratio bands), **116** (shield tiers) and **117** (body-height
//! denominator / reference-fabric binding) *before* any render happens.
//!
//! ## Doctrine — CONTINUE_REPAIR / CLAIM_HOLD
//!
//! This crate never grants admission by exclusion. A mech is `Admitted` only when
//! [`Standing::Admitted`] is *computed* by the law with **zero live refusals** and
//! backed by a **fresh, non-stale** chained receipt. Any live refusal yields
//! [`Standing::PartialAlive`] and a runtime [`ClaimHold`] — the honest "keep
//! repairing upstream" verdict, never a silent ADMITTED.
//!
//! ## GRAPH-LAW pre-render invariant
//!
//! Ratios are **ratio-of-body-height** (unit-free), so they survive absolute-scale
//! error. `body_height` is the **law-defined skeletal reference** (head-top to
//! leg-bottom), not the accessory-inclusive union.
//!
//! ## Typestate
//!
//! Admission flows through a [`Machine`] whose phase is a zero-sized
//! `PhantomData<P>` marker, so illegal transitions are *compile* errors:
//! `Measured -> Validated -> Admitted`.
//!
//! ```rust
//! use mech_morphology_law::prelude::*;
//!
//! // A canonical, in-band bipedal mech admits. body_height = 18; archetype bands
//! // from ontology 116: head 0.08-0.15, torso 0.30-0.45, leg 0.55-0.85. Here head
//! // span 2.34 -> 0.130, torso 6.66 -> 0.370, leg 11.34 -> 0.630; head above torso
//! // above legs.
//! let mech = MeasuredMechBuilder::new()
//!     .body_height(Meters::new(18.0).unwrap())
//!     .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
//!     .part(part_at("SM_Head", PartClass::MechaCrown, 30.0, 32.34))
//!     .part(part_at("SM_Torso", PartClass::TorsoSegment, 20.0, 26.66))
//!     .part(part_at("SM_Limb_Left", PartClass::BipedalLeg, 0.0, 11.34))
//!     .build()
//!     .unwrap();
//!
//! let machine = Machine::<BipedalMetricEnvelopeLaw, Measured>::new(mech);
//! let validated = machine.validate();
//! assert_eq!(validated.standing(), &Standing::Admitted);
//! assert!(validated.admit().is_ok());
//!
//! // helper used above
//! fn part_at(id: &str, class: PartClass, zmin: f64, zmax: f64) -> MeasuredPart {
//!     MeasuredPart::new(
//!         PartId::new(id),
//!         class,
//!         Bbox::new(
//!             [Meters::new(0.0).unwrap(), Meters::new(0.0).unwrap(), Meters::new(zmin).unwrap()],
//!             [Meters::new(1.0).unwrap(), Meters::new(1.0).unwrap(), Meters::new(zmax).unwrap()],
//!         ),
//!         false,
//!     )
//! }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod axis;
pub mod canonical;
pub mod law;
pub mod machine;
pub mod mech;
pub mod part;
pub mod receipt;
pub mod refusal;
pub mod shield;
pub mod units;

pub use axis::{Axis, Bbox, OrientationFinding, UpAxis};
pub use law::{BipedalMetricEnvelopeLaw, LawOutcome, MorphologyLaw};
pub use machine::{Admitted, ClaimHold, Machine, Measured, Validated};
pub use mech::{MeasuredMech, MeasuredMechBuilder};
pub use part::{MeasuredPart, PartClass, PartId};
pub use receipt::{Blake3Hash, ReplayReceipt, Staleness};
#[cfg(feature = "blake3hash")]
pub use receipt::chain;
pub use refusal::{MorphologyError, RefusalCode, Standing};
pub use shield::{ArchetypeException, ShieldTier, classify_shield};
pub use units::{Band, Meters, MetersPerUnit, MetricError, Ratio};

/// Curated single-import surface.
pub mod prelude {
    pub use crate::axis::{Axis, Bbox, OrientationFinding, UpAxis};
    pub use crate::law::{BipedalMetricEnvelopeLaw, LawOutcome, MorphologyLaw};
    pub use crate::machine::{Admitted, ClaimHold, Machine, Measured, Validated};
    pub use crate::mech::{MeasuredMech, MeasuredMechBuilder};
    pub use crate::part::{MeasuredPart, PartClass, PartId};
    pub use crate::receipt::{Blake3Hash, ReplayReceipt, Staleness};
    #[cfg(feature = "blake3hash")]
    pub use crate::receipt::chain;
    pub use crate::refusal::{MorphologyError, RefusalCode, Standing};
    pub use crate::shield::{ArchetypeException, ShieldTier, classify_shield};
    pub use crate::units::{Band, Meters, MetersPerUnit, MetricError, Ratio};
}
