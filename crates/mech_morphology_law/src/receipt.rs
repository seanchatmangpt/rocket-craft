//! BLAKE3 replay receipts, mirroring the 162-entry prev_hash-linked chain.
//!
//! `verify_fresh` re-derives the input hash from the live mech's canonical form, so
//! a STALE leaf (defect R4) is detectable: a receipt whose recomputed `input_hash`
//! diverges from the live mech cannot back an admission.

#[cfg(feature = "blake3hash")]
use crate::canonical::{canonical_mech, canonical_outcome};
#[cfg(feature = "blake3hash")]
use crate::law::LawOutcome;
#[cfg(feature = "blake3hash")]
use crate::mech::MeasuredMech;
use crate::refusal::Standing;
use alloc::string::String;
use core::fmt;

/// A 32-byte BLAKE3 digest, hex-encoded for stable serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    /// Hash arbitrary bytes (only when the `blake3hash` feature is enabled).
    #[cfg(feature = "blake3hash")]
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Blake3Hash(blake3::hash(bytes).to_hex().to_string())
    }

    /// Borrow the hex digest.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A staleness verdict: the receipt's recorded input hash no longer matches the
/// live mech.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Staleness {
    /// Hash recorded in the receipt.
    pub recorded: Blake3Hash,
    /// Hash recomputed from the live mech.
    pub recomputed: Blake3Hash,
}

impl fmt::Display for Staleness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "STALE_RECEIPT: recorded {} != live {}",
            self.recorded, self.recomputed
        )
    }
}

/// A single replay receipt in the chain.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplayReceipt {
    /// Opaque run id.
    pub run_id: String,
    /// Law identifier that produced the outcome.
    pub law_id: String,
    /// Hash of the canonical input mech.
    pub input_hash: Blake3Hash,
    /// Hash of the canonical law outcome.
    pub outcome_hash: Blake3Hash,
    /// Predecessor link hash; `None` at genesis.
    pub prev_hash: Option<Blake3Hash>,
    /// Standing recorded by the law at receipt time.
    pub standing: Standing,
}

impl ReplayReceipt {
    /// Build a receipt from a live mech + outcome (requires the hash feature).
    #[cfg(feature = "blake3hash")]
    pub fn build(
        run_id: impl Into<String>,
        law_id: impl Into<String>,
        mech: &MeasuredMech,
        outcome: &LawOutcome,
        prev: Option<&ReplayReceipt>,
    ) -> Self {
        let input_hash = Blake3Hash::of_bytes(canonical_mech(mech).as_bytes());
        let outcome_hash = Blake3Hash::of_bytes(canonical_outcome(outcome).as_bytes());
        ReplayReceipt {
            run_id: run_id.into(),
            law_id: law_id.into(),
            input_hash,
            outcome_hash,
            prev_hash: prev.map(|p| chain(p.prev_hash.as_ref(), p)),
            standing: outcome.standing.clone(),
        }
    }

    /// Re-derive the input hash from a live mech; refuse on divergence (R4).
    #[cfg(feature = "blake3hash")]
    pub fn verify_fresh(&self, live: &MeasuredMech) -> Result<(), Staleness> {
        let recomputed = Blake3Hash::of_bytes(canonical_mech(live).as_bytes());
        if recomputed == self.input_hash {
            Ok(())
        } else {
            Err(Staleness {
                recorded: self.input_hash.clone(),
                recomputed,
            })
        }
    }
}

/// Compute the link hash for a receipt: `blake3(outcome_hash || prev_hash)`,
/// mirroring the deployed chain. Genesis (`prev = None`) hashes the outcome alone.
#[cfg(feature = "blake3hash")]
pub fn chain(prev: Option<&Blake3Hash>, receipt: &ReplayReceipt) -> Blake3Hash {
    let mut buf = String::new();
    buf.push_str(receipt.outcome_hash.as_str());
    buf.push('|');
    if let Some(p) = prev {
        buf.push_str(p.as_str());
    }
    Blake3Hash::of_bytes(buf.as_bytes())
}

#[cfg(test)]
#[cfg(feature = "blake3hash")]
mod tests {
    use super::*;
    use crate::axis::{Axis, Bbox, UpAxis};
    use crate::law::{BipedalMetricEnvelopeLaw, MorphologyLaw};
    use crate::mech::MeasuredMechBuilder;
    use crate::part::{MeasuredPart, PartClass, PartId};
    use crate::units::Meters;

    fn mech(zmax: f64) -> MeasuredMech {
        let m = |v: f64| Meters::new(v).unwrap();
        MeasuredMechBuilder::new()
            .part(MeasuredPart::new(
                PartId::new("SM_Torso"),
                PartClass::TorsoSegment,
                Bbox::new([m(0.0), m(0.0), m(6.0)], [m(1.0), m(1.0), m(zmax)]),
                false,
            ))
            .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
            .body_height(m(18.0))
            .build()
            .unwrap()
    }

    #[test]
    fn chain_links_and_detects_mutation() {
        let law = BipedalMetricEnvelopeLaw::new();
        let a = mech(12.84);
        let oa = law.validate(&a);
        let r0 = ReplayReceipt::build("run-0", law.id(), &a, &oa, None);
        let r1 = ReplayReceipt::build("run-1", law.id(), &a, &oa, Some(&r0));
        assert!(r0.prev_hash.is_none());
        assert_eq!(r1.prev_hash, Some(chain(None, &r0)));

        // Fresh against same mech -> ok; mutated mech -> stale.
        assert!(r0.verify_fresh(&a).is_ok());
        let b = mech(13.0);
        assert!(r0.verify_fresh(&b).is_err());
    }
}
