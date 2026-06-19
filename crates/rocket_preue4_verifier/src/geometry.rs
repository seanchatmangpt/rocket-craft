use crate::error::RefusalReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartFamily {
    Frame,
    Shoulder,
    Arm,
    Leg,
    Socket,
    ArmorPanel,
    WeaponMount,
    CoolingVent,
}

#[derive(Debug, Clone)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    pub fn is_valid(&self) -> bool {
        self.min[0] <= self.max[0] && self.min[1] <= self.max[1] && self.min[2] <= self.max[2]
    }
}

#[derive(Debug, Clone)]
pub struct SocketMount {
    pub socket_id: String,
    pub mount_point: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct ClearanceZone {
    pub zone_id: String,
    pub bounds: Aabb,
}

#[derive(Debug, Clone)]
pub struct SemanticFeature {
    pub feature_id: String,
    pub required_for_lod: crate::semantic_lod::LodClass,
}

#[derive(Debug, Clone)]
pub struct GeometryEnvelope {
    pub part_id: String,
    pub family: PartFamily,
    pub bounds: Aabb,
    pub sockets: Vec<SocketMount>,
    pub clearance_zones: Vec<ClearanceZone>,
    pub lod_required_features: Vec<SemanticFeature>,
}

impl GeometryEnvelope {
    pub fn validate(&self) -> Result<(), RefusalReason> {
        if !self.bounds.is_valid() {
            return Err(RefusalReason::GeometryValidationFailed {
                detail: format!("invalid bounds for part {}", self.part_id),
            });
        }
        // WeaponMount requires at least one socket.
        if self.family == PartFamily::WeaponMount && self.sockets.is_empty() {
            return Err(RefusalReason::MissingSocket {
                dependent: self.part_id.clone(),
            });
        }
        // ArmorPanel with Crown LOD features must declare clearance zones.
        if self.family == PartFamily::ArmorPanel {
            let has_crown_features = self
                .lod_required_features
                .iter()
                .any(|f| f.required_for_lod == crate::semantic_lod::LodClass::Crown);
            if has_crown_features && self.clearance_zones.is_empty() {
                return Err(RefusalReason::MotionClearanceViolation {
                    detail: format!(
                        "armor panel {} has CROWN features but no clearance zones",
                        self.part_id
                    ),
                });
            }
        }
        Ok(())
    }
}

/// Validate a full mech geometry assembly.
/// Returns a list of `(part_id, RefusalReason)` for every invalid part.
pub fn validate_assembly(parts: &[GeometryEnvelope]) -> Vec<(String, RefusalReason)> {
    let mut failures = Vec::new();
    for part in parts {
        if let Err(e) = part.validate() {
            failures.push((part.part_id.clone(), e));
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_lod::LodClass;

    fn valid_aabb() -> Aabb {
        Aabb { min: [0.0, 0.0, 0.0], max: [1.0, 1.0, 1.0] }
    }

    fn invalid_aabb() -> Aabb {
        Aabb { min: [1.0, 0.0, 0.0], max: [0.0, 1.0, 1.0] } // min.x > max.x
    }

    fn base_envelope(family: PartFamily) -> GeometryEnvelope {
        GeometryEnvelope {
            part_id: "test-part".into(),
            family,
            bounds: valid_aabb(),
            sockets: Vec::new(),
            clearance_zones: Vec::new(),
            lod_required_features: Vec::new(),
        }
    }

    // ── Aabb::is_valid ────────────────────────────────────────────────────────

    #[test]
    fn aabb_is_valid_when_min_le_max() {
        assert!(valid_aabb().is_valid());
    }

    #[test]
    fn aabb_is_invalid_when_min_gt_max_on_x() {
        assert!(!invalid_aabb().is_valid());
    }

    #[test]
    fn aabb_degenerate_zero_size_is_valid() {
        let a = Aabb { min: [1.0, 1.0, 1.0], max: [1.0, 1.0, 1.0] };
        assert!(a.is_valid());
    }

    // ── GeometryEnvelope::validate ────────────────────────────────────────────

    #[test]
    fn valid_frame_passes_validation() {
        let e = base_envelope(PartFamily::Frame);
        assert!(e.validate().is_ok());
    }

    #[test]
    fn invalid_bounds_returns_geometry_error() {
        let mut e = base_envelope(PartFamily::Frame);
        e.bounds = invalid_aabb();
        let err = e.validate().unwrap_err();
        assert!(matches!(err, crate::error::RefusalReason::GeometryValidationFailed { .. }));
    }

    #[test]
    fn weapon_mount_with_no_sockets_returns_error() {
        let e = base_envelope(PartFamily::WeaponMount);
        assert!(matches!(
            e.validate().unwrap_err(),
            crate::error::RefusalReason::MissingSocket { .. }
        ));
    }

    #[test]
    fn weapon_mount_with_socket_passes() {
        let mut e = base_envelope(PartFamily::WeaponMount);
        e.sockets.push(SocketMount {
            socket_id: "S0".into(),
            mount_point: [0.0, 0.0, 0.0],
        });
        assert!(e.validate().is_ok());
    }

    #[test]
    fn armor_panel_with_crown_feature_and_no_clearance_returns_error() {
        let mut e = base_envelope(PartFamily::ArmorPanel);
        e.lod_required_features.push(SemanticFeature {
            feature_id: "wing-edge".into(),
            required_for_lod: LodClass::Crown,
        });
        // no clearance zones → should fail
        assert!(matches!(
            e.validate().unwrap_err(),
            crate::error::RefusalReason::MotionClearanceViolation { .. }
        ));
    }

    #[test]
    fn armor_panel_with_crown_feature_and_clearance_zone_passes() {
        let mut e = base_envelope(PartFamily::ArmorPanel);
        e.lod_required_features.push(SemanticFeature {
            feature_id: "wing-edge".into(),
            required_for_lod: LodClass::Crown,
        });
        e.clearance_zones.push(ClearanceZone {
            zone_id: "Z0".into(),
            bounds: valid_aabb(),
        });
        assert!(e.validate().is_ok());
    }

    #[test]
    fn armor_panel_with_non_crown_features_needs_no_clearance() {
        let mut e = base_envelope(PartFamily::ArmorPanel);
        e.lod_required_features.push(SemanticFeature {
            feature_id: "wing-detail".into(),
            required_for_lod: LodClass::Secondary,
        });
        // no clearance zones and non-CROWN feature → should pass
        assert!(e.validate().is_ok());
    }

    // ── PartFamily variants ───────────────────────────────────────────────────

    #[test]
    fn part_family_variants_are_distinct() {
        assert_ne!(PartFamily::Frame, PartFamily::Shoulder);
        assert_ne!(PartFamily::WeaponMount, PartFamily::ArmorPanel);
    }
}
