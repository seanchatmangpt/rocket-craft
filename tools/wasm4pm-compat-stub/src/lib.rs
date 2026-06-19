pub mod state {
    pub struct Raw;
    pub struct Admitted;
}

pub mod witness {
    pub struct Ocel20;
    pub struct Wasm4pmBridge;
}

pub mod evidence {
    use crate::state::{Admitted, Raw};
    use std::marker::PhantomData;

    pub struct Evidence<T, S, W> {
        pub value: T,
        pub(crate) _state: PhantomData<S>,
        pub(crate) _witness: PhantomData<W>,
    }

    impl<T, W> Evidence<T, Raw, W> {
        pub fn raw(value: T) -> Self {
            Self {
                value,
                _state: PhantomData,
                _witness: PhantomData,
            }
        }
    }

    impl<T, W> Evidence<T, Admitted, W> {
        pub fn into_inner(self) -> T {
            self.value
        }
    }
}

pub mod admission {
    use crate::evidence::Evidence;
    use crate::state::{Admitted, Raw};
    use std::marker::PhantomData;

    pub struct Admission<T, W> {
        pub value: T,
        _witness: PhantomData<W>,
    }

    impl<T, W> Admission<T, W> {
        pub fn new(value: T) -> Self {
            Self {
                value,
                _witness: PhantomData,
            }
        }

        pub fn into_evidence(self) -> Evidence<T, Admitted, W> {
            Evidence {
                value: self.value,
                _state: PhantomData,
                _witness: PhantomData,
            }
        }
    }

    pub trait Admit {
        type Raw;
        type Admitted;
        type Reason;
        type Witness;

        fn admit(
            raw: Evidence<Self::Raw, Raw, Self::Witness>,
        ) -> Result<Admission<Self::Admitted, Self::Witness>, Refusal<Self::Reason, Self::Witness>>;
    }

    pub struct Refusal<R, W> {
        pub reason: R,
        _witness: PhantomData<W>,
    }

    impl<R, W> Refusal<R, W> {
        pub fn new(reason: R) -> Self {
            Self {
                reason,
                _witness: PhantomData,
            }
        }
    }
}

pub mod engine_bridge {
    #[derive(Debug, Clone, PartialEq)]
    pub enum GraduationReason {
        NeedsDiscovery,
        NeedsConformanceExecution,
        NeedsReceipts,
        NeedsReplay,
        RebuildingProcessMiningLocally,
    }

    #[derive(Debug, Clone)]
    pub struct GraduationCandidate {
        pub reason: GraduationReason,
        pub subject: String,
        pub evidence_ref: String,
    }

    impl GraduationCandidate {
        pub fn new(
            reason: GraduationReason,
            subject: impl Into<String>,
            evidence_ref: impl Into<String>,
        ) -> Self {
            Self {
                reason,
                subject: subject.into(),
                evidence_ref: evidence_ref.into(),
            }
        }

        pub fn is_grounded(&self) -> bool {
            !self.evidence_ref.is_empty()
        }
    }

    pub trait GraduateToWasm4pm {
        fn candidate(&self) -> GraduationCandidate;
    }
}

pub mod ocel {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OCELRelationship {
        pub object_id: String,
        pub qualifier: String,
    }

    impl OCELRelationship {
        pub fn new(object_id: impl Into<String>, qualifier: impl Into<String>) -> Self {
            Self {
                object_id: object_id.into(),
                qualifier: qualifier.into(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OCELEvent {
        pub id: String,
        pub event_type: String,
        pub relationships: Vec<OCELRelationship>,
    }

    impl OCELEvent {
        pub fn new(id: impl Into<String>, event_type: &str) -> Self {
            Self {
                id: id.into(),
                event_type: event_type.to_string(),
                relationships: Vec::new(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OCELObject {
        pub id: String,
        pub object_type: String,
    }

    impl OCELObject {
        pub fn new(id: impl Into<String>, object_type: &str) -> Self {
            Self {
                id: id.into(),
                object_type: object_type.to_string(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OCEL {
        pub event_types: Vec<String>,
        pub object_types: Vec<String>,
        pub events: Vec<OCELEvent>,
        pub objects: Vec<OCELObject>,
    }

    pub mod validate {
        use super::OCEL;
        use std::collections::{HashMap, HashSet};

        pub struct ValidationError {
            pub code: String,
            pub message: String,
        }

        pub struct ValidationReport {
            pub valid: bool,
            pub errors: Vec<ValidationError>,
        }

        pub fn validate(ocel: &OCEL, _params: &HashMap<String, String>) -> ValidationReport {
            let mut errors = Vec::new();
            let object_ids: HashSet<&str> = ocel.objects.iter().map(|o| o.id.as_str()).collect();
            for event in &ocel.events {
                for rel in &event.relationships {
                    if !object_ids.contains(rel.object_id.as_str()) {
                        errors.push(ValidationError {
                            code: "E2O_MISSING_TARGET".to_string(),
                            message: format!(
                                "Event {} references unknown object {}",
                                event.id, rel.object_id
                            ),
                        });
                        break;
                    }
                }
            }
            ValidationReport {
                valid: errors.is_empty(),
                errors,
            }
        }
    }
}

pub mod receipt {
    #[derive(Debug, Clone, PartialEq)]
    pub enum ReceiptRefusal {
        MissingSubject,
        UnreplayableClaim,
        InvalidWitness,
        MissingDigest,
        MissingReplayHint,
    }

    pub struct Digest {
        pub value: String,
    }

    impl Digest {
        pub fn new(value: impl Into<String>) -> Self {
            Self {
                value: value.into(),
            }
        }
    }

    pub struct ReplayHint {
        pub value: String,
    }

    impl ReplayHint {
        pub fn new(value: impl Into<String>) -> Self {
            Self {
                value: value.into(),
            }
        }
    }

    pub struct ReceiptEnvelope {
        pub subject: String,
        pub witness: String,
        pub digest: Digest,
        pub replay_hint: ReplayHint,
    }

    impl ReceiptEnvelope {
        pub fn try_from_parts(
            subject: String,
            witness: &str,
            digest: Digest,
            replay_hint: ReplayHint,
        ) -> Result<Self, ReceiptRefusal> {
            if subject.is_empty() {
                return Err(ReceiptRefusal::MissingSubject);
            }
            if witness.is_empty() {
                return Err(ReceiptRefusal::InvalidWitness);
            }
            Ok(Self {
                subject,
                witness: witness.to_string(),
                digest,
                replay_hint,
            })
        }

        pub fn is_well_shaped(&self) -> bool {
            !self.subject.is_empty() && !self.witness.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── Evidence typestate ────────────────────────────────────────────────────

    #[test]
    fn evidence_raw_wraps_value() {
        let e = evidence::Evidence::<i32, state::Raw, witness::Ocel20>::raw(42);
        assert_eq!(e.value, 42);
    }

    #[test]
    fn admission_into_evidence_provides_inner() {
        let adm = admission::Admission::<String, witness::Ocel20>::new("hello".into());
        let ev = adm.into_evidence();
        assert_eq!(ev.into_inner(), "hello");
    }

    // ── GraduationCandidate ───────────────────────────────────────────────────

    #[test]
    fn graduation_candidate_new_stores_fields() {
        let g = engine_bridge::GraduationCandidate::new(
            engine_bridge::GraduationReason::NeedsDiscovery,
            "mech-001",
            "ocel://run-42",
        );
        assert_eq!(g.subject, "mech-001");
        assert_eq!(g.evidence_ref, "ocel://run-42");
    }

    #[test]
    fn graduation_candidate_is_grounded_true_when_nonempty_ref() {
        let g = engine_bridge::GraduationCandidate::new(
            engine_bridge::GraduationReason::NeedsReceipts,
            "sub",
            "ref-001",
        );
        assert!(g.is_grounded());
    }

    #[test]
    fn graduation_candidate_is_grounded_false_when_empty_ref() {
        let g = engine_bridge::GraduationCandidate::new(
            engine_bridge::GraduationReason::NeedsReplay,
            "sub",
            "",
        );
        assert!(!g.is_grounded());
    }

    // ── OCEL validate ─────────────────────────────────────────────────────────

    fn make_ocel_valid() -> ocel::OCEL {
        ocel::OCEL {
            event_types: vec!["BIRTH".into()],
            object_types: vec!["mech".into()],
            events: vec![{
                let mut e = ocel::OCELEvent::new("e1", "BIRTH");
                e.relationships.push(ocel::OCELRelationship::new("o1", "case"));
                e
            }],
            objects: vec![ocel::OCELObject::new("o1", "mech")],
        }
    }

    #[test]
    fn validate_passes_when_all_relationships_reference_known_objects() {
        let ocel = make_ocel_valid();
        let report = ocel::validate::validate(&ocel, &HashMap::new());
        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn validate_fails_when_event_references_missing_object() {
        let mut ocel = make_ocel_valid();
        ocel.events[0].relationships[0].object_id = "ghost".into();
        let report = ocel::validate::validate(&ocel, &HashMap::new());
        assert!(!report.valid);
        assert!(report.errors[0].code == "E2O_MISSING_TARGET");
    }

    // ── ReceiptEnvelope ───────────────────────────────────────────────────────

    #[test]
    fn receipt_envelope_ok_with_valid_parts() {
        let result = receipt::ReceiptEnvelope::try_from_parts(
            "subj".into(),
            "witness",
            receipt::Digest::new("abc"),
            receipt::ReplayHint::new("hint"),
        );
        let env = match result { Ok(e) => e, Err(_) => panic!("expected Ok") };
        assert!(env.is_well_shaped());
    }

    #[test]
    fn receipt_envelope_fails_on_empty_subject() {
        let err = match receipt::ReceiptEnvelope::try_from_parts(
            "".into(),
            "witness",
            receipt::Digest::new("h"),
            receipt::ReplayHint::new("r"),
        ) { Ok(_) => panic!("expected Err"), Err(e) => e };
        assert_eq!(err, receipt::ReceiptRefusal::MissingSubject);
    }

    #[test]
    fn receipt_envelope_fails_on_empty_witness() {
        let err = match receipt::ReceiptEnvelope::try_from_parts(
            "subj".into(),
            "",
            receipt::Digest::new("h"),
            receipt::ReplayHint::new("r"),
        ) { Ok(_) => panic!("expected Err"), Err(e) => e };
        assert_eq!(err, receipt::ReceiptRefusal::InvalidWitness);
    }
}
