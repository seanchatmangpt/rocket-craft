//! Part taxonomy. An unclassifiable part becomes [`PartClass::Unclassified`] and
//! forces a refusal — never a silent omission (closes gap-3). A rotated part keeps
//! `has_rotation = true` so measurement-incompleteness is typed, not dropped.

use crate::axis::Bbox;
use alloc::string::String;

/// The morphological class of a part, mirroring ontology 116 part bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PartClass {
    /// Head / crown (`SM_Head`).
    MechaCrown,
    /// Torso segment (`SM_Torso`).
    TorsoSegment,
    /// Bipedal leg / limb (`SM_Limb_*`).
    BipedalLeg,
    /// Wing array (`SM_WingArray_*`).
    WingArray,
    /// Weapon (`SM_Blade_*`).
    MechaWeapon,
    /// Shield (tier-governed).
    Shield,
    /// Cosmetic accessory (excluded from the body-height denominator).
    Accessory,
    /// Could not be classified from the schema — forces a refusal.
    Unclassified,
}

impl PartClass {
    /// Stable token for canonical serialization.
    pub fn token(self) -> &'static str {
        match self {
            PartClass::MechaCrown => "MechaCrown",
            PartClass::TorsoSegment => "TorsoSegment",
            PartClass::BipedalLeg => "BipedalLeg",
            PartClass::WingArray => "WingArray",
            PartClass::MechaWeapon => "MechaWeapon",
            PartClass::Shield => "Shield",
            PartClass::Accessory => "Accessory",
            PartClass::Unclassified => "Unclassified",
        }
    }

    /// Whether this class participates in the skeletal body-height reference
    /// (head-top to leg-bottom). Accessories/shields/wings/weapons do not.
    pub fn is_skeletal(self) -> bool {
        matches!(
            self,
            PartClass::MechaCrown | PartClass::TorsoSegment | PartClass::BipedalLeg
        )
    }
}

/// A part identifier newtype.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PartId(pub String);

impl PartId {
    /// Construct from anything string-like.
    pub fn new(s: impl Into<String>) -> Self {
        PartId(s.into())
    }

    /// Borrow the underlying id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A measured part: identity, class, bbox, and a rotation flag.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MeasuredPart {
    /// Part identity.
    pub id: PartId,
    /// Classified morphology.
    pub class: PartClass,
    /// Axis-aligned bounding box.
    pub bbox: Bbox,
    /// True when the part carries an unhandled rotation / `xformOpOrder`
    /// (e.g. `SM_Blade_L/R`), making its bbox an incomplete measurement.
    pub has_rotation: bool,
}

impl MeasuredPart {
    /// Construct a measured part.
    pub fn new(id: PartId, class: PartClass, bbox: Bbox, has_rotation: bool) -> Self {
        MeasuredPart {
            id,
            class,
            bbox,
            has_rotation,
        }
    }

    /// True when the part cannot be lawfully measured (rotated or unclassified)
    /// and therefore blocks admission.
    pub fn is_unmeasurable(&self) -> bool {
        self.has_rotation || self.class == PartClass::Unclassified
    }
}
