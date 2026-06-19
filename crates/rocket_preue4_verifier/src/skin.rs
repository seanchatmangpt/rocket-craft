//! GC-MECHBIRTH-002: Skin/Material Surrogate
//! Validates layered skin stacks for pre-UE4 material binding compliance.

use crate::error::RefusalReason;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkinLayer {
    BaseMaterial = 0,
    FactionPalette = 1,
    SponsorLivery = 2,
    ThermalZones = 3,
    DamageMasks = 4,
    WearMasks = 5,
    RepairResidue = 6,
    SemanticHighlights = 7,
    LodTextureSet = 8,
}

#[derive(Debug, Clone)]
pub struct SkinSpec {
    pub layers: Vec<SkinLayer>,
    pub damage_class_binding: u8,
    pub heat_class_binding: u8,
    pub has_thermal_vent_visible: bool,
    pub sponsor_livery_present: bool,
    pub repair_receipt: Option<String>,
}

pub fn validate_skin_stack(spec: &SkinSpec) -> Result<(), RefusalReason> {
    // BaseMaterial before FactionPalette
    if spec.layers.contains(&SkinLayer::FactionPalette)
        && !spec.layers.contains(&SkinLayer::BaseMaterial)
    {
        return Err(RefusalReason::SkinOccludesRequiredFeature {
            feature: "BaseMaterial_required_before_FactionPalette".into(),
        });
    }
    // ThermalZones before SponsorLivery (readability check)
    if spec.sponsor_livery_present && !spec.layers.contains(&SkinLayer::ThermalZones) {
        return Err(RefusalReason::SkinOccludesRequiredFeature {
            feature: "ThermalZones_required_before_SponsorLivery".into(),
        });
    }
    // SponsorLivery cannot hide thermal vent
    if spec.sponsor_livery_present && !spec.has_thermal_vent_visible {
        return Err(RefusalReason::SkinOccludesRequiredFeature {
            feature: "thermal_vent_hidden_by_sponsor_livery".into(),
        });
    }
    // DamageMasks must bind to damage class (non-zero binding required if damage masks exist)
    if spec.layers.contains(&SkinLayer::DamageMasks) && spec.damage_class_binding == 0 {
        return Err(RefusalReason::SkinOccludesRequiredFeature {
            feature: "DamageMasks_requires_damage_class_binding".into(),
        });
    }
    // RepairResidue requires receipt
    if spec.layers.contains(&SkinLayer::RepairResidue) && spec.repair_receipt.is_none() {
        return Err(RefusalReason::SkinOccludesRequiredFeature {
            feature: "RepairResidue_requires_repair_receipt".into(),
        });
    }
    Ok(())
}
