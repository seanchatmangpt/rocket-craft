//! The `MorphologyLaw` trait (the `Law` in `Machine<Law, Phase>`) and the concrete
//! [`BipedalMetricEnvelopeLaw`] encoding ontology 110 (anatomy + part bands), 116
//! (shield tiers) and 117 (skeletal body-height denominator).

use crate::axis::{Axis, OrientationFinding};
use crate::mech::MeasuredMech;
use crate::part::{MeasuredPart, PartClass};
use crate::refusal::{RefusalCode, Standing};
use crate::shield::{ArchetypeException, classify_shield};
use crate::units::{Band, Ratio};
use alloc::vec::Vec;

/// The full result of validating a mech against a law.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LawOutcome {
    /// Computed standing (never hardcoded).
    pub standing: Standing,
    /// All refusals raised (sorted/stable order).
    pub refusals: Vec<RefusalCode>,
    /// Orientation finding when an up-axis disagreement is present.
    pub orientation: Option<OrientationFinding>,
}

/// The law trait. A law decides a [`LawOutcome`] for a measured mech, purely.
pub trait MorphologyLaw {
    /// Stable law identifier (e.g. ontology binding id).
    fn id(&self) -> &str;
    /// Validate a mech; pure and deterministic.
    fn validate(&self, mech: &MeasuredMech) -> LawOutcome;
}

/// Per-class ratio band lookup, mirrored EXACTLY from ontology 116
/// `law:*HeightBand` `bandMinRatio`/`bandMaxRatio` literals (the authoritative
/// graph law). These are the same bands enforced by the SHACL
/// `law:PartHeightBandShape` and cross-checked by
/// `scripts/verify_metric_morphology.py::PART_BANDS`. Do NOT widen them: a wider
/// band here would ADMIT a part the graph law REFUSES.
fn class_band(class: PartClass) -> Option<Band> {
    let lit = match class {
        // law:HeadHeightBand  (MechaCrown)
        PartClass::MechaCrown => (0.0800, 0.1500),
        // law:TorsoHeightBand (TorsoSegment)
        PartClass::TorsoSegment => (0.3000, 0.4500),
        // law:LegHeightBand / law:LimbHeightBand (BipedalLeg / BipedalLimb)
        PartClass::BipedalLeg => (0.5500, 0.8500),
        // law:WingSpanBand vertical ratio band (bandMinRatio/bandMaxRatio)
        PartClass::WingArray => (0.4000, 0.9000),
        // law:WeaponHeightBand (MechaWeapon)
        PartClass::MechaWeapon => (0.1000, 0.6000),
        // Shields are governed by the tier law, not a single class band.
        // Accessories/Unclassified have no band.
        PartClass::Shield | PartClass::Accessory | PartClass::Unclassified => return None,
    };
    Band::new(lit.0, lit.1).ok()
}

/// The bipedal metric envelope law.
#[derive(Debug, Default, Clone, Copy)]
pub struct BipedalMetricEnvelopeLaw {
    /// Optional archetype unlocking the exception shield band. `None` means a
    /// shield in the exception band is refused (no default proportion).
    pub archetype: Option<ArchetypeException>,
}

impl BipedalMetricEnvelopeLaw {
    /// New law with no archetype exception.
    pub fn new() -> Self {
        Self { archetype: None }
    }

    /// New law that grants `archetype` (unlocks the exception shield band).
    pub fn with_archetype(archetype: ArchetypeException) -> Self {
        Self {
            archetype: Some(archetype),
        }
    }

    /// The law-chosen vertical axis: the declared up-axis (authoritative-by-spec).
    fn vertical(&self, mech: &MeasuredMech) -> Axis {
        mech.declared_up.axis()
    }
}

impl MorphologyLaw for BipedalMetricEnvelopeLaw {
    fn id(&self) -> &str {
        "rf:110_116_117_bipedal_metric_envelope"
    }

    fn validate(&self, mech: &MeasuredMech) -> LawOutcome {
        let mut refusals: Vec<RefusalCode> = Vec::new();
        let vert = self.vertical(mech);

        // --- Measurability law (R5/gap-3): rotated/unclassified parts block. ---
        for p in &mech.parts {
            if p.is_unmeasurable() {
                refusals.push(RefusalCode::UnmeasurablePart {
                    part: p.id.clone(),
                });
            }
        }

        // --- Per-part ratio bands (116) + shield tier law. ---
        let bh = mech.body_height;
        for p in &mech.parts {
            if p.is_unmeasurable() {
                continue; // already flagged; bbox is an incomplete measurement.
            }
            let span = p.bbox.span(vert).abs();
            let part_h = match crate::units::Meters::new(span) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let ratio = match Ratio::of(part_h, bh) {
                Ok(r) => r,
                Err(_) => continue,
            };
            match p.class {
                PartClass::Shield => {
                    if let Err(code) = classify_shield(ratio, self.archetype) {
                        refusals.push(code);
                    }
                }
                _ => {
                    if let Some(band) = class_band(p.class) {
                        if !band.contains(ratio) {
                            refusals.push(RefusalCode::RefusePartHeightBand {
                                part: p.id.clone(),
                                ratio,
                                band,
                            });
                        }
                    }
                }
            }
        }

        // --- Anatomy ordering (110): head above torso; legs below pelvis/torso. ---
        if let Some(anatomy) = check_anatomy(mech, vert) {
            refusals.push(anatomy);
        }

        // --- Orientation law: declared vs actual stacking axis. ---
        let orientation = Some(OrientationFinding {
            declared: mech.declared_up,
            actual: mech.actual_up,
        });
        let orient_defect = orientation.map(|o| o.is_defect()).unwrap_or(false);
        if orient_defect {
            refusals.push(RefusalCode::OrientationDefect);
        }

        let standing = compute_standing(&refusals);
        LawOutcome {
            standing,
            refusals,
            orientation,
        }
    }
}

/// Head bbox-min must be above torso bbox-max along the vertical axis; any
/// bipedal leg bbox-max must be below the torso bbox-min. Returns
/// [`RefusalCode::AnatomyParadox`] on any violation.
fn check_anatomy(mech: &MeasuredMech, vert: Axis) -> Option<RefusalCode> {
    let by_class = |c: PartClass| -> Vec<&MeasuredPart> {
        mech.parts.iter().filter(|p| p.class == c).collect()
    };
    let heads = by_class(PartClass::MechaCrown);
    let torsos = by_class(PartClass::TorsoSegment);
    let legs = by_class(PartClass::BipedalLeg);

    // head.min must be >= torso.max (head sits above torso).
    for h in &heads {
        let (h_min, _) = h.bbox.extent_along(vert);
        for t in &torsos {
            let (_, t_max) = t.bbox.extent_along(vert);
            if h_min < t_max {
                return Some(RefusalCode::AnatomyParadox);
            }
        }
    }
    // leg.max must be <= torso.min (legs sit below torso/pelvis).
    for l in &legs {
        let (_, l_max) = l.bbox.extent_along(vert);
        for t in &torsos {
            let (t_min, _) = t.bbox.extent_along(vert);
            if l_max > t_min {
                return Some(RefusalCode::AnatomyParadox);
            }
        }
    }
    None
}

/// Compute standing from the refusal set. ANATOMY_PARADOX or a bad shield
/// proportion is a structural rejection (`Refused`); pure off-band/orientation/
/// measurability findings keep the asset structurally alive (`PartialAlive`).
fn compute_standing(refusals: &[RefusalCode]) -> Standing {
    if refusals.is_empty() {
        return Standing::Admitted;
    }
    let structural = refusals.iter().any(|c| {
        matches!(
            c,
            RefusalCode::AnatomyParadox | RefusalCode::RefuseDefaultShieldProportion
        )
    });
    if structural {
        Standing::Refused {
            codes: refusals.to_vec(),
        }
    } else {
        Standing::PartialAlive {
            live_refusals: refusals.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{Bbox, UpAxis};
    use crate::mech::MeasuredMechBuilder;
    use crate::part::PartId;
    use crate::units::Meters;

    fn part(id: &str, class: PartClass, zmin: f64, zmax: f64) -> MeasuredPart {
        let m = |v: f64| Meters::new(v).unwrap();
        MeasuredPart::new(
            PartId::new(id),
            class,
            Bbox::new([m(0.0), m(0.0), m(zmin)], [m(1.0), m(1.0), m(zmax)]),
            false,
        )
    }

    fn in_band_mech() -> MeasuredMech {
        // body_height 18. Archetype bands (116): head 0.08-0.15, torso 0.30-0.45,
        // leg 0.55-0.85. head span 2.34 -> 0.130; torso span 6.66 -> 0.370; leg
        // span 11.34 -> 0.630. Anatomy: head(30-32.34).min >= torso(20-26.66).max;
        // leg(0-11.34).max <= torso.min(20).
        MeasuredMechBuilder::new()
            .part(part("SM_Head", PartClass::MechaCrown, 30.0, 32.34))
            .part(part("SM_Torso", PartClass::TorsoSegment, 20.0, 26.66))
            .part(part("SM_Limb_Left", PartClass::BipedalLeg, 0.0, 11.34))
            .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
            .body_height(Meters::new(18.0).unwrap())
            .build()
            .unwrap()
    }

    #[test]
    fn in_band_admits() {
        let law = BipedalMetricEnvelopeLaw::new();
        let out = law.validate(&in_band_mech());
        assert_eq!(out.standing, Standing::Admitted, "{:?}", out.refusals);
    }

    #[test]
    fn anatomy_paradox_refuses() {
        // head below torso.
        let mech = MeasuredMechBuilder::new()
            .part(part("SM_Head", PartClass::MechaCrown, 0.0, 2.0))
            .part(part("SM_Torso", PartClass::TorsoSegment, 6.0, 12.84))
            .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
            .body_height(Meters::new(18.0).unwrap())
            .build()
            .unwrap();
        let out = BipedalMetricEnvelopeLaw::new().validate(&mech);
        assert!(matches!(out.standing, Standing::Refused { .. }));
        assert!(out.refusals.contains(&RefusalCode::AnatomyParadox));
    }
}
