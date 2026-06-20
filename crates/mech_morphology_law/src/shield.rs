//! Shield-tier law (mirror of ontology 116). The `Exception` tier requires an
//! explicit [`ArchetypeException`]; a default/absent archetype refuses.

use crate::refusal::RefusalCode;
use crate::units::{Band, Ratio};

/// Inclusive shield-tier bands as raw `(lo, hi)` literals (single source of truth).
const NORMAL: (f64, f64) = (0.25, 0.35);
const LARGE: (f64, f64) = (0.35, 0.55);
const TOWER: (f64, f64) = (0.55, 0.85);
const EXCEPTION: (f64, f64) = (0.85, 1.10);

#[inline]
fn band_of(lit: (f64, f64)) -> Band {
    // The literals are finite and non-negative, so `Band::new` cannot fail here.
    Band::new(lit.0, lit.1).unwrap_or(Band::ZERO)
}

/// Shield ratio tiers, boundary-inclusive per band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ShieldTier {
    /// `0.25..=0.35`
    Normal,
    /// `0.35..=0.55`
    Large,
    /// `0.55..=0.85`
    Tower,
    /// `0.85..=1.10` — requires an archetype.
    Exception,
}

impl ShieldTier {
    /// The inclusive band for this tier.
    pub fn band(self) -> Band {
        match self {
            ShieldTier::Normal => band_of(NORMAL),
            ShieldTier::Large => band_of(LARGE),
            ShieldTier::Tower => band_of(TOWER),
            ShieldTier::Exception => band_of(EXCEPTION),
        }
    }
}

/// Archetype exceptions that lawfully unlock the `Exception` shield band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArchetypeException {
    /// Tower archetype.
    Tower,
    /// Siege archetype.
    Siege,
    /// Mobile-barricade archetype.
    MobileBarricade,
    /// Transformation-shield archetype.
    TransformationShield,
}

/// Classify a shield ratio into a tier.
///
/// The `Exception` band (`0.85..=1.10`) is only lawful when an
/// [`ArchetypeException`] is present; otherwise this returns
/// [`RefusalCode::RefuseDefaultShieldProportion`].
pub fn classify_shield(
    ratio: Ratio,
    archetype: Option<ArchetypeException>,
) -> Result<ShieldTier, RefusalCode> {
    for tier in [
        ShieldTier::Normal,
        ShieldTier::Large,
        ShieldTier::Tower,
        ShieldTier::Exception,
    ] {
        if tier.band().contains(ratio) {
            if tier == ShieldTier::Exception && archetype.is_none() {
                return Err(RefusalCode::RefuseDefaultShieldProportion);
            }
            return Ok(tier);
        }
    }
    // Outside every tier band (e.g. < 0.25 or > 1.10) -> default-proportion refusal.
    Err(RefusalCode::RefuseDefaultShieldProportion)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(v: f64) -> Ratio {
        Ratio::new(v).unwrap()
    }

    #[test]
    fn tiers_classify() {
        assert_eq!(classify_shield(r(0.30), None), Ok(ShieldTier::Normal));
        assert_eq!(classify_shield(r(0.45), None), Ok(ShieldTier::Large));
        assert_eq!(classify_shield(r(0.70), None), Ok(ShieldTier::Tower));
    }

    #[test]
    fn exception_requires_archetype() {
        assert_eq!(
            classify_shield(r(0.95), None),
            Err(RefusalCode::RefuseDefaultShieldProportion)
        );
        assert_eq!(
            classify_shield(r(0.95), Some(ArchetypeException::Tower)),
            Ok(ShieldTier::Exception)
        );
    }

    #[test]
    fn seams_inclusive() {
        assert_eq!(classify_shield(r(0.35), None), Ok(ShieldTier::Normal));
        assert_eq!(classify_shield(r(0.55), None), Ok(ShieldTier::Large));
        assert_eq!(classify_shield(r(0.85), None), Ok(ShieldTier::Tower));
        assert_eq!(
            classify_shield(r(1.10), Some(ArchetypeException::Siege)),
            Ok(ShieldTier::Exception)
        );
    }

    #[test]
    fn ufo_disc_ratio_refused() {
        // ratio 1.0 with no archetype -> refuse.
        assert_eq!(
            classify_shield(r(1.0), None),
            Err(RefusalCode::RefuseDefaultShieldProportion)
        );
    }
}
