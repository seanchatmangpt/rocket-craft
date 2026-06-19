
//! Verifier orchestration — GATE 0–7 pipeline.
//! ⚠️ GENERATED FILE — do NOT edit by hand.
//! Source of truth: ontology/mechbirth.ttl
//! Generator:       ggen/templates/verifier.rs.tera
//! SPARQL:          ggen/sparql/extract_verifier_gates.sparql

use crate::authority::AuthorityState;
use crate::error::JidokaEvent;
use crate::receipt::ReceiptChain;
use crate::transitions::TransitionTable;

/// Overall gate result for a single pipeline run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateResult {
    Pass,
    Fail(String),
    /// ResidualScope gate — excluded from ALIVE_UNDER_SCOPE scoring.
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
    /// True only when every gate is Pass (no Fails, no Residuals).
    pub fn is_all_pass(&self) -> bool {
        self.gates.iter().all(|(_, r)| *r == GateResult::Pass)
    }

    /// True when every in-scope gate (non-RESIDUAL) passes.
    pub fn is_in_scope_pass(&self) -> bool {
        self.gates
            .iter()
            .filter(|(_, r)| !matches!(r, GateResult::Residual(_)))
            .all(|(_, r)| *r == GateResult::Pass)
    }

    /// Returns bounded status vocabulary for Rust-scoped gates only.
    pub fn scoped_status(&self) -> &'static str {
        if self.is_in_scope_pass() {
            "ALIVE_UNDER_SCOPE"
        } else {
            "PARTIAL_ALIVE_CANDIDATE"
        }
    }
}

/// Run the verifier pipeline.
/// Gate sequence and scope classification generated from extract_verifier_gates.sparql.
pub fn run_pipeline(state: &mut AuthorityState, chain: &ReceiptChain) -> PipelineResult {
    let mut result = PipelineResult::default();

    // ── GATE 0: GATE_0_SOURCE_ADMISSION (RustScope — always passes) ─────────
    result.gates.push(("GATE_0_SOURCE_ADMISSION".into(), GateResult::Pass));

    // ── GATE 1: GATE_1_AUTHORITY_LENGTHS (RustScope) ────────────────────────
    let gate1 = match state.validate_lengths() {
        Ok(_) => GateResult::Pass,
        Err(e) => {
            result.jidoka_events.push(JidokaEvent {
                defect_class: "AUTHORITY_LENGTHS".into(),
                surface: "authority_state".into(),
                expected_law: "all SoA buffers same length".into(),
                observed_failure: e.to_string(),
                residual: String::new(),
                repair_candidate: Some("resize all buffers to minimum length".into()),
                repair_applied: false,
                receipt: None,
            });
            GateResult::Fail(e.to_string())
        }
    };
    result.gates.push(("GATE_1_AUTHORITY_LENGTHS".into(), gate1));

    // ── GATE 2: GATE_2_AUTHORITY_CLASSES (RustScope) ────────────────────────
    let violations2 = state.validate_classes();
    let gate2 = if violations2.is_empty() {
        GateResult::Pass
    } else {
        GateResult::Fail(format!("{} class violations", violations2.len()))
    };
    result.gates.push(("GATE_2_AUTHORITY_CLASSES".into(), gate2));

    // ── GATE 3: GATE_3_SCALAR_TABLE_EQUIV (RustScope) ───────────────────────
    let gate3 = match crate::simd::verify_simd_scalar_equivalence(
        &state.heat, &state.stress, &state.socket_health,
    ) {
        Ok(_)  => GateResult::Pass,
        Err(e) => GateResult::Fail(e.to_string()),
    };
    result.gates.push(("GATE_3_SCALAR_TABLE_EQUIV".into(), gate3));

    // ── GATE 4: GATE_4_RECEIPT_CHAIN (RustScope) ────────────────────────────
    let gate4 = match chain.verify() {
        Ok(_)  => GateResult::Pass,
        Err(e) => GateResult::Fail(e.to_string()),
    };
    result.gates.push(("GATE_4_RECEIPT_CHAIN".into(), gate4));

    // ── GATE 5: GATE_5_UE4_WASM_PACKAGE (ResidualScope — τ silent transition)
    result.gates.push((
        "GATE_5_UE4_WASM_PACKAGE".into(),
        GateResult::Residual("ue4_projection: no rendered surface — deferred to HTML5/UE4 pipeline".into()),
    ));

    // ── GATE 6: GATE_6_MOTION_DELTA (ResidualScope — τ silent transition) ───
    result.gates.push((
        "GATE_6_MOTION_DELTA".into(),
        GateResult::Residual("visual_delta: no Playwright browser session in pre-UE4 scope".into()),
    ));

    // ── GATE 7: GATE_7_RECEIPT (RustScope) — verify hash integrity ───────────
    let gate7 = match chain.verify_hashes() {
        Ok(_) => GateResult::Pass,
        Err(e) => {
            result.jidoka_events.push(JidokaEvent {
                defect_class: "RECEIPT_HASH_INTEGRITY".into(),
                surface: "receipt_chain".into(),
                expected_law: "every entry.receipt == blake3(content)".into(),
                observed_failure: e.to_string(),
                residual: String::new(),
                repair_candidate: Some("recompute receipts from source events".into()),
                repair_applied: false,
                receipt: None,
            });
            GateResult::Fail(e.to_string())
        }
    };
    result.gates.push(("GATE_7_RECEIPT".into(), gate7));

    // Apply table-driven damage update after validation gates.
    let table = TransitionTable::build();
    crate::transitions::batch_update_damage_table(state, &table);

    // final_status: ALIVE_UNDER_SCOPE when all RustScope gates pass.
    result.final_status = if result.is_in_scope_pass() {
        "ALIVE_UNDER_SCOPE".into()
    } else {
        "BLOCKED".into()
    };

    result
}

// ────────────────────────────────────────────────────────────────────────────
// Ontology-annotated gate summary (sourced from extract_verifier_gates.sparql)
// ────────────────────────────────────────────────────────────────────────────
// 
// GATE 0: GATE_0_SOURCE_ADMISSION
//   Scope:    RustScope
//   Law:      Contract for prompt is declared
//   Validates: N/A
//   Residual:  N/A
// 
// GATE 1: GATE_1_AUTHORITY_LENGTHS
//   Scope:    RustScope
//   Law:      All SoA buffers have identical length
//   Validates: AuthorityState::validate_lengths
//   Residual:  N/A
// 
// GATE 2: GATE_2_AUTHORITY_CLASSES
//   Scope:    RustScope
//   Law:      All class values in [0, MAX_CLASS]
//   Validates: AuthorityState::validate_classes
//   Residual:  N/A
// 
// GATE 3: GATE_3_SCALAR_TABLE_EQUIV
//   Scope:    RustScope
//   Law:      SIMD-equivalent path produces bit-identical output to scalar reference
//   Validates: verify_simd_scalar_equivalence
//   Residual:  N/A
// 
// GATE 4: GATE_4_RECEIPT_CHAIN
//   Scope:    RustScope
//   Law:      Receipt chain prev_hash links are intact
//   Validates: ReceiptChain::verify
//   Residual:  N/A
// 
// GATE 5: GATE_5_UE4_WASM_PACKAGE
//   Scope:    ResidualScope
//   Law:      SpeculativeCoder UE4.27 HTML5 ES3 build produces browser-deployable output
//   Validates: N/A
//   Residual:  ue4_projection: no rendered surface — deferred to HTML5/UE4 pipeline
// 
// GATE 6: GATE_6_MOTION_DELTA
//   Scope:    ResidualScope
//   Law:      Playwright motion delta exceeds threshold after input actuation
//   Validates: N/A
//   Residual:  visual_delta: no Playwright browser session in pre-UE4 scope
// 
// GATE 7: GATE_7_RECEIPT
//   Scope:    RustScope
//   Law:      Every receipt entry hash equals blake3(content)
//   Validates: ReceiptChain::verify_hashes
//   Residual:  N/A
// 