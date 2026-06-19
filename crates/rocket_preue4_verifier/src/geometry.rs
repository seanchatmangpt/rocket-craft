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
