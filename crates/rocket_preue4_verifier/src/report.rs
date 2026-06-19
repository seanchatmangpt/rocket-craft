//! GC-MECHBIRTH-002: Machine-readable report generation.
//!
//! Produces a structured `VerifierReport` that can be serialized to JSON.
//! The report captures all gate results, jidoka events, residuals, and final status.

use crate::verifier::{GateResult, PipelineResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRecord {
    pub gate: String,
    pub result: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JidokaRecord {
    pub defect_class: String,
    pub surface: String,
    pub expected_law: String,
    pub observed_failure: String,
    pub residual: String,
    pub repair_candidate: Option<String>,
    pub repair_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierReport {
    pub milestone: String,
    pub scoped_status: String,
    pub final_status: String,
    pub gates: Vec<GateRecord>,
    pub jidoka_events: Vec<JidokaRecord>,
    pub residuals: Vec<String>,
    pub inputs: Vec<String>,
    pub generated_artifacts: Vec<String>,
}

impl VerifierReport {
    /// Build a report from a completed pipeline result.
    pub fn from_pipeline(
        milestone: String,
        result: &PipelineResult,
        inputs: Vec<String>,
        generated_artifacts: Vec<String>,
        static_residuals: Vec<String>,
    ) -> Self {
        let gates = result
            .gates
            .iter()
            .map(|(name, r)| match r {
                GateResult::Pass => GateRecord {
                    gate: name.clone(),
                    result: "PASS".into(),
                    detail: None,
                },
                GateResult::Fail(d) => GateRecord {
                    gate: name.clone(),
                    result: "FAIL".into(),
                    detail: Some(d.clone()),
                },
                GateResult::Residual(d) => GateRecord {
                    gate: name.clone(),
                    result: "RESIDUAL".into(),
                    detail: Some(d.clone()),
                },
            })
            .collect();

        let jidoka_events = result
            .jidoka_events
            .iter()
            .map(|ev| JidokaRecord {
                defect_class: ev.defect_class.clone(),
                surface: ev.surface.clone(),
                expected_law: ev.expected_law.clone(),
                observed_failure: ev.observed_failure.clone(),
                residual: ev.residual.clone(),
                repair_candidate: ev.repair_candidate.clone(),
                repair_applied: ev.repair_applied,
            })
            .collect();

        Self {
            milestone,
            scoped_status: result.scoped_status().to_string(),
            final_status: result.final_status.clone(),
            gates,
            jidoka_events,
            residuals: static_residuals,
            inputs,
            generated_artifacts,
        }
    }

    /// Serialize to pretty-printed JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("VerifierReport serialization must not fail")
    }
}

/// Produce a default GC-MECHBIRTH-002 report with known static residuals.
pub fn mechbirth_002_residuals() -> Vec<String> {
    vec![
        "ggen_artifact_lowering: hand-generated surrogates, not ggen-manufactured".into(),
        "ue4_projection: no rendered surface".into(),
        "signing_layer: tamper-evident, not cryptographically signed".into(),
        "simdE_ffi: C SIMDe FFI kernel deferred to GC-MECHBIRTH-003".into(),
        "stress_1M: 1M cell stress test deferred pending dev machine capacity check".into(),
        "wasm4pm_playground: @wasm4pm/testing build blocked by nuxt postinstall".into(),
    ]
}

/// Produce a default GC-GUNDAM-FACTORY-001 report with known static residuals.
pub fn gundam_factory_001_residuals() -> Vec<String> {
    vec![
        "ue4_projection: no rendered surface — deferred to HTML5/UE4 pipeline".into(),
        "visual_delta: no Playwright browser session in pre-UE4 scope".into(),
        "signing_layer: tamper-evident, not cryptographically signed".into(),
        "stress_1M: 1M cell stress test deferred pending dev machine capacity check".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JidokaEvent;
    use crate::verifier::{GateResult, PipelineResult};

    fn passing_pipeline() -> PipelineResult {
        PipelineResult {
            gates: vec![
                ("GATE_0".into(), GateResult::Pass),
                ("GATE_1".into(), GateResult::Pass),
            ],
            jidoka_events: vec![],
            final_status: "ALIVE_UNDER_SCOPE".into(),
        }
    }

    fn failing_pipeline() -> PipelineResult {
        PipelineResult {
            gates: vec![
                ("GATE_0".into(), GateResult::Pass),
                ("GATE_1".into(), GateResult::Fail("mismatch".into())),
                ("GATE_2".into(), GateResult::Residual("deferred to 003".into())),
            ],
            jidoka_events: vec![JidokaEvent {
                defect_class: "AUTHORITY_LENGTHS".into(),
                surface: "authority_state".into(),
                expected_law: "equal lengths".into(),
                observed_failure: "heat=3, stress=4".into(),
                residual: String::new(),
                repair_candidate: Some("resize".into()),
                repair_applied: false,
                receipt: None,
            }],
            final_status: "PARTIAL_ALIVE_CANDIDATE".into(),
        }
    }

    // ── VerifierReport::from_pipeline ─────────────────────────────────────────

    #[test]
    fn from_pipeline_captures_milestone_and_final_status() {
        let pr = passing_pipeline();
        let report = VerifierReport::from_pipeline(
            "GC-MECHBIRTH-002".into(),
            &pr,
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(report.milestone, "GC-MECHBIRTH-002");
        assert_eq!(report.final_status, "ALIVE_UNDER_SCOPE");
    }

    #[test]
    fn from_pipeline_scoped_status_is_alive_when_all_pass() {
        let pr = passing_pipeline();
        let report = VerifierReport::from_pipeline("M".into(), &pr, vec![], vec![], vec![]);
        assert_eq!(report.scoped_status, "ALIVE_UNDER_SCOPE");
    }

    #[test]
    fn from_pipeline_scoped_status_is_partial_when_any_fail() {
        let pr = failing_pipeline();
        let report = VerifierReport::from_pipeline("M".into(), &pr, vec![], vec![], vec![]);
        assert_eq!(report.scoped_status, "PARTIAL_ALIVE_CANDIDATE");
    }

    #[test]
    fn from_pipeline_maps_gate_results() {
        let pr = failing_pipeline();
        let report = VerifierReport::from_pipeline("M".into(), &pr, vec![], vec![], vec![]);
        assert_eq!(report.gates.len(), 3);
        assert_eq!(report.gates[0].result, "PASS");
        assert_eq!(report.gates[0].detail, None);
        assert_eq!(report.gates[1].result, "FAIL");
        assert_eq!(report.gates[1].detail.as_deref(), Some("mismatch"));
        assert_eq!(report.gates[2].result, "RESIDUAL");
    }

    #[test]
    fn from_pipeline_maps_jidoka_events() {
        let pr = failing_pipeline();
        let report = VerifierReport::from_pipeline("M".into(), &pr, vec![], vec![], vec![]);
        assert_eq!(report.jidoka_events.len(), 1);
        assert_eq!(report.jidoka_events[0].defect_class, "AUTHORITY_LENGTHS");
        assert!(!report.jidoka_events[0].repair_applied);
    }

    #[test]
    fn from_pipeline_records_inputs_artifacts_residuals() {
        let pr = passing_pipeline();
        let report = VerifierReport::from_pipeline(
            "M".into(),
            &pr,
            vec!["input_a.json".into()],
            vec!["out.bin".into()],
            vec!["residual_note".into()],
        );
        assert_eq!(report.inputs, vec!["input_a.json"]);
        assert_eq!(report.generated_artifacts, vec!["out.bin"]);
        assert_eq!(report.residuals, vec!["residual_note"]);
    }

    // ── to_json ───────────────────────────────────────────────────────────────

    #[test]
    fn to_json_produces_valid_json_with_milestone_field() {
        let pr = passing_pipeline();
        let report =
            VerifierReport::from_pipeline("GC-002".into(), &pr, vec![], vec![], vec![]);
        let json = report.to_json();
        assert!(json.contains("\"milestone\""));
        assert!(json.contains("GC-002"));
        // parse to confirm well-formed JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed["milestone"], "GC-002");
    }

    // ── mechbirth_002_residuals ───────────────────────────────────────────────

    #[test]
    fn mechbirth_002_residuals_non_empty() {
        assert!(!mechbirth_002_residuals().is_empty());
    }

    #[test]
    fn mechbirth_002_residuals_contains_simd_note() {
        let r = mechbirth_002_residuals();
        assert!(r.iter().any(|s| s.contains("simdE_ffi")));
    }
}
