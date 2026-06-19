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
