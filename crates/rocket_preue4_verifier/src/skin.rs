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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_spec() -> SkinSpec {
        SkinSpec {
            layers: vec![SkinLayer::BaseMaterial],
            damage_class_binding: 0,
            heat_class_binding: 0,
            has_thermal_vent_visible: true,
            sponsor_livery_present: false,
            repair_receipt: None,
        }
    }

    #[test]
    fn minimal_base_material_only_passes() {
        assert!(validate_skin_stack(&minimal_spec()).is_ok());
    }

    #[test]
    fn faction_palette_without_base_material_fails() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::FactionPalette], // no BaseMaterial
            ..minimal_spec()
        };
        assert!(matches!(
            validate_skin_stack(&spec).unwrap_err(),
            RefusalReason::SkinOccludesRequiredFeature { .. }
        ));
    }

    #[test]
    fn faction_palette_with_base_material_passes() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::FactionPalette],
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_ok());
    }

    #[test]
    fn sponsor_livery_without_thermal_zones_fails() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial],
            sponsor_livery_present: true,
            has_thermal_vent_visible: true,
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_err());
    }

    #[test]
    fn sponsor_livery_hiding_thermal_vent_fails() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::ThermalZones],
            sponsor_livery_present: true,
            has_thermal_vent_visible: false, // vent occluded
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_err());
    }

    #[test]
    fn sponsor_livery_with_thermal_zones_and_visible_vent_passes() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::ThermalZones],
            sponsor_livery_present: true,
            has_thermal_vent_visible: true,
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_ok());
    }

    #[test]
    fn damage_masks_without_binding_fails() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::DamageMasks],
            damage_class_binding: 0, // no binding
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_err());
    }

    #[test]
    fn damage_masks_with_binding_passes() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::DamageMasks],
            damage_class_binding: 5,
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_ok());
    }

    #[test]
    fn repair_residue_without_receipt_fails() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::RepairResidue],
            repair_receipt: None,
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_err());
    }

    #[test]
    fn repair_residue_with_receipt_passes() {
        let spec = SkinSpec {
            layers: vec![SkinLayer::BaseMaterial, SkinLayer::RepairResidue],
            repair_receipt: Some("receipt-abc123".into()),
            ..minimal_spec()
        };
        assert!(validate_skin_stack(&spec).is_ok());
    }
}
