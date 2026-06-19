use thiserror::Error;

/// All refusal reasons that the verifier can emit.
/// Every refusal carries a `RefusalReason`. No panics in non-test code paths
/// except genuinely unreachable states with a comment.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RefusalReason {
    #[error("invalid authority class {class} for field {field}")]
    InvalidAuthorityClass { field: String, class: u8 },

    #[error("missing socket required by {dependent}")]
    MissingSocket { dependent: String },

    #[error("motion clearance violation: {detail}")]
    MotionClearanceViolation { detail: String },

    #[error("skin occludes required feature: {feature}")]
    SkinOccludesRequiredFeature { feature: String },

    #[error("LOD demoted CROWN feature without authority reason: {feature}")]
    LodDemotedCrownFeature { feature: String },

    #[error("prediction attempted to overwrite admitted authority state")]
    PredictionAuthorityMutation,

    #[error("SIMD output diverges from scalar for input index {index}")]
    SimdScalarDivergence { index: usize },

    #[error("receipt chain broken at sequence {sequence}: expected {expected}, got {actual}")]
    ReceiptChainBroken {
        sequence: u64,
        expected: String,
        actual: String,
    },

    #[error("orphan projection row {row_id} has no source receipt")]
    OrphanProjectionRow { row_id: String },

    #[error("POWL step missing from trace: {step}")]
    MissingPowlStep { step: String },

    #[error("geometry validation failed: {detail}")]
    GeometryValidationFailed { detail: String },

    #[error("LOD input refused: {detail}")]
    LodRefused { detail: String },
}

/// An Jidoka event record produced whenever a defect is detected and surfaced.
#[derive(Debug, Clone)]
pub struct JidokaEvent {
    pub defect_class: String,
    pub surface: String,
    pub expected_law: String,
    pub observed_failure: String,
    pub residual: String,
    pub repair_candidate: Option<String>,
    pub repair_applied: bool,
    pub receipt: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_authority_class_display() {
        let e = RefusalReason::InvalidAuthorityClass { field: "lod".into(), class: 99 };
        assert!(e.to_string().contains("99"));
        assert!(e.to_string().contains("lod"));
    }

    #[test]
    fn missing_socket_display() {
        let e = RefusalReason::MissingSocket { dependent: "FireWeapon".into() };
        assert!(e.to_string().contains("FireWeapon"));
    }

    #[test]
    fn motion_clearance_violation_display() {
        let e = RefusalReason::MotionClearanceViolation { detail: "too hot".into() };
        assert!(e.to_string().contains("too hot"));
    }

    #[test]
    fn prediction_authority_mutation_display() {
        let e = RefusalReason::PredictionAuthorityMutation;
        let s = e.to_string();
        assert!(s.contains("overwrite") || s.contains("authority"));
    }

    #[test]
    fn simd_scalar_divergence_display() {
        let e = RefusalReason::SimdScalarDivergence { index: 7 };
        assert!(e.to_string().contains("7"));
    }

    #[test]
    fn receipt_chain_broken_display() {
        let e = RefusalReason::ReceiptChainBroken {
            sequence: 42, expected: "abc".into(), actual: "xyz".into(),
        };
        let s = e.to_string();
        assert!(s.contains("42") && s.contains("abc") && s.contains("xyz"));
    }

    #[test]
    fn orphan_projection_row_display() {
        let e = RefusalReason::OrphanProjectionRow { row_id: "row-9".into() };
        assert!(e.to_string().contains("row-9"));
    }

    #[test]
    fn geometry_validation_failed_display() {
        let e = RefusalReason::GeometryValidationFailed { detail: "bad bounds".into() };
        assert!(e.to_string().contains("bad bounds"));
    }

    #[test]
    fn jidoka_event_fields_are_accessible() {
        let ev = JidokaEvent {
            defect_class: "C1".into(), surface: "geometry".into(),
            expected_law: "AABB-valid".into(), observed_failure: "min>max".into(),
            residual: "reject".into(), repair_candidate: Some("flip-bounds".into()),
            repair_applied: false, receipt: None,
        };
        assert_eq!(ev.defect_class, "C1");
        assert!(!ev.repair_applied);
    }

    #[test]
    fn refusal_reason_variants_are_clone_and_partialeq() {
        let a = RefusalReason::MissingSocket { dependent: "X".into() };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
