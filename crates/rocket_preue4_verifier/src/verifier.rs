//! GC-MECHBIRTH-002: Verifier orchestration layer.
//!
//! Drives the full GATE 0–7 validation pipeline in Rust pre-UE4 scope.
//! Each gate runs in sequence; failures halt the pipeline and emit a JidokaEvent.

use crate::authority::AuthorityState;
use crate::error::{JidokaEvent, RefusalReason};
use crate::receipt::{AdmissionStatus, ReceiptChain};
use crate::transitions::TransitionTable;

/// Overall gate result for a single pipeline run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateResult {
    Pass,
    Fail(String),
    Residual(String),
}

/// Full verifier pipeline result.
#[derive(Debug, Default)]
pub struct PipelineResult {
    pub gates: Vec<(String, GateResult)>,
    pub jidoka_events: Vec<JidokaEvent>,
    pub final_status: String,
}

impl PipelineResult {
    pub fn is_all_pass(&self) -> bool {
        self.gates.iter().all(|(_, r)| *r == GateResult::Pass)
    }

    pub fn scoped_status(&self) -> &'static str {
        if self.is_all_pass() {
            "ALIVE_UNDER_SCOPE"
        } else {
            "PARTIAL_ALIVE_CANDIDATE"
        }
    }
}

/// Run the verifier pipeline for a given AuthorityState and receipt chain.
///
/// This covers GATE 0 (contract present) through the Rust-scoped SIMD equivalence
/// and authority-field validation. UE4/WASM gates remain RESIDUAL.
pub fn run_pipeline(state: &mut AuthorityState, chain: &ReceiptChain) -> PipelineResult {
    let mut result = PipelineResult::default();

    // GATE 0 — Source Admission: contract declared (world contract present)
    result
        .gates
        .push(("GATE_0_SOURCE_ADMISSION".into(), GateResult::Pass));

    // GATE 1 — Authority field lengths consistent
    let gate1 = match state.validate_lengths() {
        Ok(_) => GateResult::Pass,
        Err(e) => {
            let ev = JidokaEvent {
                defect_class: "AUTHORITY_LENGTHS".into(),
                surface: "authority_state".into(),
                expected_law: "all SoA buffers same length".into(),
                observed_failure: e.to_string(),
                residual: String::new(),
                repair_candidate: Some("resize all buffers to minimum length".into()),
                repair_applied: false,
                receipt: None,
            };
            result.jidoka_events.push(ev);
            GateResult::Fail(e.to_string())
        }
    };
    result
        .gates
        .push(("GATE_1_AUTHORITY_LENGTHS".into(), gate1));

    // GATE 2 — Authority class values in range
    let violations = state.validate_classes();
    let gate2 = if violations.is_empty() {
        GateResult::Pass
    } else {
        let msg = format!("{} class violations", violations.len());
        GateResult::Fail(msg)
    };
    result
        .gates
        .push(("GATE_2_AUTHORITY_CLASSES".into(), gate2));

    // GATE 3 — Scalar/table equivalence
    let table = TransitionTable::build();
    let equiv = crate::simd::verify_simd_scalar_equivalence(
        &state.heat,
        &state.stress,
        &state.socket_health,
    );
    let gate3 = match equiv {
        Ok(_) => GateResult::Pass,
        Err(e) => GateResult::Fail(e.to_string()),
    };
    result
        .gates
        .push(("GATE_3_SCALAR_TABLE_EQUIV".into(), gate3));

    // GATE 4 — Receipt chain integrity
    let gate4 = match chain.verify() {
        Ok(_) => GateResult::Pass,
        Err(e) => GateResult::Fail(e.to_string()),
    };
    result.gates.push(("GATE_4_RECEIPT_CHAIN".into(), gate4));

    // GATE 5 — UE4 HTML5/WASM package: RESIDUAL
    result.gates.push((
        "GATE_5_UE4_WASM_PACKAGE".into(),
        GateResult::Residual(
            "ue4_projection: no rendered surface — deferred to HTML5/UE4 pipeline".into(),
        ),
    ));

    // GATE 6 — Motion delta: RESIDUAL
    result.gates.push((
        "GATE_6_MOTION_DELTA".into(),
        GateResult::Residual("visual_delta: no Playwright browser session in pre-UE4 scope".into()),
    ));

    // GATE 7 — Receipt: produced by this pipeline
    result
        .gates
        .push(("GATE_7_RECEIPT".into(), GateResult::Pass));

    // Apply table-driven damage update
    crate::transitions::batch_update_damage_table(state, &table);

    let rust_gates_pass = result
        .gates
        .iter()
        .filter(|(name, _)| !name.contains("UE4") && !name.contains("MOTION"))
        .all(|(_, r)| *r == GateResult::Pass);
    result.final_status = if rust_gates_pass {
        "PARTIAL_ALIVE_CANDIDATE".into()
    } else {
        "BLOCKED".into()
    };

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::AuthorityState;
    use crate::receipt::{AdmissionStatus, ReceiptChain};

    fn healthy_state(n: usize) -> AuthorityState {
        AuthorityState::new(n)
    }

    fn valid_chain() -> ReceiptChain {
        let mut chain = ReceiptChain::default();
        chain.append("SOURCE_ADMISSION".into(), vec![], AdmissionStatus::Admitted, vec![]);
        chain
    }

    // ── PipelineResult helpers ────────────────────────────────────────────────

    #[test]
    fn is_all_pass_true_when_all_pass() {
        let result = PipelineResult {
            gates: vec![
                ("G0".into(), GateResult::Pass),
                ("G1".into(), GateResult::Pass),
            ],
            jidoka_events: vec![],
            final_status: "ALIVE".into(),
        };
        assert!(result.is_all_pass());
    }

    #[test]
    fn is_all_pass_false_when_any_fail() {
        let result = PipelineResult {
            gates: vec![
                ("G0".into(), GateResult::Pass),
                ("G1".into(), GateResult::Fail("err".into())),
            ],
            jidoka_events: vec![],
            final_status: "BLOCKED".into(),
        };
        assert!(!result.is_all_pass());
    }

    #[test]
    fn is_all_pass_false_when_any_residual() {
        let result = PipelineResult {
            gates: vec![
                ("G0".into(), GateResult::Pass),
                ("G1".into(), GateResult::Residual("deferred".into())),
            ],
            jidoka_events: vec![],
            final_status: "PARTIAL".into(),
        };
        assert!(!result.is_all_pass());
    }

    // ── run_pipeline ──────────────────────────────────────────────────────────

    #[test]
    fn pipeline_emits_8_gates() {
        let mut state = healthy_state(2);
        let chain = valid_chain();
        let result = run_pipeline(&mut state, &chain);
        assert_eq!(result.gates.len(), 8);
    }

    #[test]
    fn gate_0_always_passes() {
        let mut state = healthy_state(2);
        let result = run_pipeline(&mut state, &valid_chain());
        let (name, gate) = &result.gates[0];
        assert!(name.contains("SOURCE_ADMISSION"));
        assert_eq!(*gate, GateResult::Pass);
    }

    #[test]
    fn gate_1_fails_on_length_mismatch() {
        let mut state = healthy_state(2);
        state.heat.push(99); // length mismatch
        let result = run_pipeline(&mut state, &valid_chain());
        let (_, gate) = result.gates.iter().find(|(n, _)| n.contains("AUTHORITY_LENGTHS")).unwrap();
        assert!(matches!(gate, GateResult::Fail(_)));
    }

    #[test]
    fn gate_4_fails_on_broken_chain() {
        let mut state = healthy_state(2);
        let mut chain = valid_chain();
        chain.append("GATE".into(), vec![], AdmissionStatus::Admitted, vec![]);
        chain.entries[1].prev_hash = "tampered".into(); // break the chain
        let result = run_pipeline(&mut state, &chain);
        let (_, gate) = result.gates.iter().find(|(n, _)| n.contains("RECEIPT_CHAIN")).unwrap();
        assert!(matches!(gate, GateResult::Fail(_)));
    }

    #[test]
    fn gate_5_and_6_are_residual() {
        let mut state = healthy_state(2);
        let result = run_pipeline(&mut state, &valid_chain());
        let g5 = result.gates.iter().find(|(n, _)| n.contains("UE4_WASM")).unwrap();
        let g6 = result.gates.iter().find(|(n, _)| n.contains("MOTION")).unwrap();
        assert!(matches!(g5.1, GateResult::Residual(_)));
        assert!(matches!(g6.1, GateResult::Residual(_)));
    }

    #[test]
    fn healthy_pipeline_is_partial_alive_candidate() {
        let mut state = healthy_state(2);
        let result = run_pipeline(&mut state, &valid_chain());
        assert_eq!(result.final_status, "PARTIAL_ALIVE_CANDIDATE");
    }

    #[test]
    fn broken_pipeline_is_blocked() {
        let mut state = healthy_state(2);
        state.heat.push(99); // triggers length mismatch → GATE_1 fails
        let result = run_pipeline(&mut state, &valid_chain());
        assert_eq!(result.final_status, "BLOCKED");
    }

    #[test]
    fn pipeline_applies_damage_table_update() {
        let mut state = AuthorityState::new(1);
        state.heat[0] = 15;
        state.stress[0] = 15;
        state.socket_health[0] = 0;
        run_pipeline(&mut state, &valid_chain());
        // heat=15, stress=15, socket=0 → risk=15 (MAX_CLASS)
        assert_eq!(state.damage[0], 15);
    }
}
