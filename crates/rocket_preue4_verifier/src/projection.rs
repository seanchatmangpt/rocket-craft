//! GC-MECHBIRTH-002: Projection Manifest
//! Row-level validation of UE4 surface projection mappings.

use crate::error::RefusalReason;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionType {
    SetMeshVariant,
    SetMaterialMask,
    SetInstanceTransform,
    SetLodClass,
    SetAnimationParam,
    SetVisibilityBatch,
    SetDebugReceiptOverlay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionStatus {
    Admitted,
    Refused,
    Residual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionRow {
    pub projection_id: String,
    pub source_powl_step: String,
    /// must not be empty for admitted rows
    pub source_receipt: String,
    pub object_id: String,
    pub projection_type: ProjectionType,
    pub authority_inputs: Vec<String>,
    pub semantic_lod_class: String,
    pub ue4_target_surface: String,
    pub admission_status: AdmissionStatus,
}

impl ProjectionRow {
    pub fn validate(&self) -> Result<(), RefusalReason> {
        // Every admitted row must have a source receipt
        if self.admission_status == AdmissionStatus::Admitted && self.source_receipt.is_empty() {
            return Err(RefusalReason::OrphanProjectionRow {
                row_id: self.projection_id.clone(),
            });
        }
        // Crown rows must have authority inputs
        if self.semantic_lod_class == "Crown" && self.authority_inputs.is_empty() {
            return Err(RefusalReason::OrphanProjectionRow {
                row_id: format!("{} (Crown missing authority_inputs)", self.projection_id),
            });
        }
        Ok(())
    }
}

/// Validate a full projection manifest.
/// Returns a Vec of (projection_id, RefusalReason) for every invalid row.
pub fn validate_manifest(rows: &[ProjectionRow]) -> Vec<(String, RefusalReason)> {
    let mut failures = Vec::new();
    for row in rows {
        if let Err(e) = row.validate() {
            failures.push((row.projection_id.clone(), e));
        }
    }
    failures
}
