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

#[cfg(test)]
mod tests {
    use super::*;

    fn admitted_row(id: &str, receipt: &str) -> ProjectionRow {
        ProjectionRow {
            projection_id: id.into(),
            source_powl_step: "STEP_1".into(),
            source_receipt: receipt.into(),
            object_id: "obj-1".into(),
            projection_type: ProjectionType::SetMeshVariant,
            authority_inputs: vec!["damage_class".into()],
            semantic_lod_class: "Primary".into(),
            ue4_target_surface: "SM_Frame".into(),
            admission_status: AdmissionStatus::Admitted,
        }
    }

    // ── ProjectionRow::validate ───────────────────────────────────────────────

    #[test]
    fn admitted_row_with_receipt_passes() {
        assert!(admitted_row("r1", "hash-abc").validate().is_ok());
    }

    #[test]
    fn admitted_row_without_receipt_is_orphan() {
        let row = admitted_row("r2", ""); // empty receipt
        assert!(matches!(
            row.validate().unwrap_err(),
            RefusalReason::OrphanProjectionRow { .. }
        ));
    }

    #[test]
    fn refused_row_without_receipt_is_ok() {
        let mut row = admitted_row("r3", "");
        row.admission_status = AdmissionStatus::Refused;
        assert!(row.validate().is_ok());
    }

    #[test]
    fn residual_row_without_receipt_is_ok() {
        let mut row = admitted_row("r4", "");
        row.admission_status = AdmissionStatus::Residual;
        assert!(row.validate().is_ok());
    }

    #[test]
    fn crown_row_without_authority_inputs_fails() {
        let mut row = admitted_row("r5", "hash-def");
        row.semantic_lod_class = "Crown".into();
        row.authority_inputs.clear();
        assert!(matches!(
            row.validate().unwrap_err(),
            RefusalReason::OrphanProjectionRow { .. }
        ));
    }

    #[test]
    fn crown_row_with_authority_inputs_passes() {
        let mut row = admitted_row("r6", "hash-ghi");
        row.semantic_lod_class = "Crown".into();
        assert!(row.validate().is_ok()); // already has authority_inputs
    }

    // ── validate_manifest ─────────────────────────────────────────────────────

    #[test]
    fn manifest_with_no_failures_returns_empty_vec() {
        let rows = vec![
            admitted_row("r1", "hash-1"),
            admitted_row("r2", "hash-2"),
        ];
        assert!(validate_manifest(&rows).is_empty());
    }

    #[test]
    fn manifest_collects_all_failures() {
        let rows = vec![
            admitted_row("r1", "hash-1"),   // ok
            admitted_row("r2", ""),          // fail: orphan
            admitted_row("r3", "hash-3"),   // ok
            admitted_row("r4", ""),          // fail: orphan
        ];
        let failures = validate_manifest(&rows);
        assert_eq!(failures.len(), 2);
        assert!(failures.iter().any(|(id, _)| id == "r2"));
        assert!(failures.iter().any(|(id, _)| id == "r4"));
    }

    #[test]
    fn empty_manifest_returns_empty_vec() {
        assert!(validate_manifest(&[]).is_empty());
    }
}
