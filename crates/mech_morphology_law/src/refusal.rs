//! Typed verdict / refusal vocabulary. `Display` maps each code to the exact wire
//! string used by `scripts/verify_metric_morphology.py`.

use crate::part::PartId;
use crate::units::{Band, Ratio};
use alloc::vec::Vec;
use core::fmt;

/// A single law refusal.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RefusalCode {
    /// Anatomy ordering violated (head below torso, leg above pelvis, etc.).
    AnatomyParadox,
    /// A shield in the exception band without an explicit archetype, or a shield
    /// ratio outside every tier band.
    RefuseDefaultShieldProportion,
    /// A part's ratio-of-body-height fell outside its class band.
    RefusePartHeightBand {
        /// Offending part.
        part: PartId,
        /// Measured ratio.
        ratio: Ratio,
        /// Expected band.
        band: Band,
    },
    /// Declared up-axis disagrees with the actual stacking axis.
    OrientationDefect,
    /// A part is rotated/unclassified and therefore not lawfully measurable.
    UnmeasurablePart {
        /// Offending part.
        part: PartId,
    },
}

impl fmt::Display for RefusalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefusalCode::AnatomyParadox => f.write_str("ANATOMY_PARADOX"),
            RefusalCode::RefuseDefaultShieldProportion => {
                f.write_str("REFUSE_DEFAULT_SHIELD_PROPORTION")
            }
            RefusalCode::RefusePartHeightBand { part, ratio, band } => write!(
                f,
                "REFUSE_PART_HEIGHT_BAND({}): ratio {:.4} outside [{:.2},{:.2}]",
                part.as_str(),
                ratio.get(),
                band.lo.get(),
                band.hi.get()
            ),
            RefusalCode::OrientationDefect => f.write_str("ORIENTATION_DEFECT"),
            RefusalCode::UnmeasurablePart { part } => {
                write!(f, "UNMEASURABLE_PART({})", part.as_str())
            }
        }
    }
}

/// The computed standing of a mech. Never hardcoded — always produced by the law.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Standing {
    /// Zero live refusals; eligible for admission.
    Admitted,
    /// Real defects present but the asset is structurally alive (CONTINUE_REPAIR).
    PartialAlive {
        /// The live refusals keeping it out of admission.
        live_refusals: Vec<RefusalCode>,
    },
    /// The asset is rejected outright (e.g. anatomy paradox + bad shield).
    Refused {
        /// All refusal codes.
        codes: Vec<RefusalCode>,
    },
    /// Standing could not be determined.
    Unknown,
}

/// Top-level typed error surface.
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MorphologyError {
    /// A metric construction error escaped to the law boundary.
    #[cfg_attr(feature = "std", error("metric error: {0:?}"))]
    Metric(crate::units::MetricError),
    /// The builder was given an empty or duplicate-id mech.
    #[cfg_attr(feature = "std", error("invalid mech construction: {0}"))]
    Build(&'static str),
}

#[cfg(not(feature = "std"))]
impl fmt::Display for MorphologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MorphologyError::Metric(e) => write!(f, "metric error: {:?}", e),
            MorphologyError::Build(m) => write!(f, "invalid mech construction: {}", m),
        }
    }
}

impl From<crate::units::MetricError> for MorphologyError {
    fn from(e: crate::units::MetricError) -> Self {
        MorphologyError::Metric(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn wire_strings_match_python() {
        assert_eq!(RefusalCode::AnatomyParadox.to_string(), "ANATOMY_PARADOX");
        assert_eq!(
            RefusalCode::RefuseDefaultShieldProportion.to_string(),
            "REFUSE_DEFAULT_SHIELD_PROPORTION"
        );
        let c = RefusalCode::RefusePartHeightBand {
            part: PartId::new("SM_Head"),
            ratio: Ratio::new(0.0710).unwrap(),
            band: Band::new(0.08, 0.15).unwrap(),
        };
        assert_eq!(
            c.to_string(),
            "REFUSE_PART_HEIGHT_BAND(SM_Head): ratio 0.0710 outside [0.08,0.15]"
        );
    }
}
