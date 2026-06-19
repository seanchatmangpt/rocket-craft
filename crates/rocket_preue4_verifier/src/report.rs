//! GC-MECHBIRTH-002: Machine-readable report generation.
//!
//! Produces a structured `VerifierReport` that can be serialized to JSON.
//! The report captures all gate results, jidoka events, residuals, and final status.

use crate::error::JidokaEvent;
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
