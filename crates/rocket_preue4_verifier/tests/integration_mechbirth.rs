//! GC-MECHBIRTH-002: Integration test — MechBirth OCEL trace replay.
//!
//! Reads the real OCEL trace from `/Users/sac/powlv2lsp/out.json`,
//! parses it with `OcelLog::from_powlv2lsp_trace`, then exercises the full
//! Receipt Chain, AuthorityState, and Projection Manifest pipeline.
//!
//! Bounded status vocabulary:
//! VERIFIED_UNDER_SCOPE / ALIVE_UNDER_SCOPE / PARTIAL_ALIVE_CANDIDATE / BLOCKED / RESIDUAL

use rocket_preue4_verifier::authority::AuthorityState;
use rocket_preue4_verifier::error::RefusalReason;
use rocket_preue4_verifier::ocel::OcelLog;
use rocket_preue4_verifier::projection::{
    AdmissionStatus as ProjAdmission, ProjectionRow, ProjectionType, validate_manifest,
};
use rocket_preue4_verifier::receipt::{AdmissionStatus, ReceiptChain};
use rocket_preue4_verifier::transitions::batch_update_damage_scalar;

/// Expected MechBirth POWL step event types (must match out.json exactly).
const EXPECTED_EVENT_TYPES: &[&str] = &[
    "Select Frame",
    "Generate Socket Topology",
    "Generate Armor Panels",
    "Generate Rig",
    "Generate Motion Family",
    "Generate Skin Layers",
    "Package Artifacts",
    "Emit Receipt",
];

/// VERIFIED_UNDER_SCOPE: Parse real OCEL trace and verify 8 events
#[test]
fn test_ocel_parse_real_trace() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/out.json")
        .expect("BLOCKED: out.json not found at /Users/sac/powlv2lsp/out.json");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    assert_eq!(
        log.events.len(),
        8,
        "Expected exactly 8 MechBirth POWL events, found {}",
        log.events.len()
    );
}

/// VERIFIED_UNDER_SCOPE: Parsed events match expected MechBirth POWL step types
#[test]
fn test_ocel_event_types_match_mechbirth_powl() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/out.json")
        .expect("BLOCKED: out.json not found at /Users/sac/powlv2lsp/out.json");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    for (i, event) in log.events.iter().enumerate() {
        assert_eq!(
            event.event_type,
            EXPECTED_EVENT_TYPES[i],
            "Event {} type mismatch: expected '{}', got '{}'",
            i + 1,
            EXPECTED_EVENT_TYPES[i],
            event.event_type
        );
    }
}

/// VERIFIED_UNDER_SCOPE: ReceiptChain mirroring OCEL events verifies cleanly
#[test]
fn test_receipt_chain_mirrors_ocel_events() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/out.json")
        .expect("BLOCKED: out.json not found at /Users/sac/powlv2lsp/out.json");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    let mut chain = ReceiptChain::default();
    for event in &log.events {
        chain.append(
            event.event_type.clone(),
            event.objects.clone(),
            AdmissionStatus::Admitted,
            vec![],
        );
    }

    assert_eq!(chain.entries.len(), 8, "Chain should have 8 entries");
    assert!(
        chain.verify().is_ok(),
        "Mirrored chain should verify cleanly"
    );
    assert!(
        chain.verify_hashes().is_ok(),
        "Mirrored chain hashes should verify cleanly"
    );
}

/// VERIFIED_UNDER_SCOPE: Mutating a chain entry produces ReceiptChainBroken
#[test]
fn test_receipt_chain_mutation_produces_broken_error() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/out.json")
        .expect("BLOCKED: out.json not found at /Users/sac/powlv2lsp/out.json");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    let mut chain = ReceiptChain::default();
    for event in &log.events {
        chain.append(
            event.event_type.clone(),
            event.objects.clone(),
            AdmissionStatus::Admitted,
            vec![],
        );
    }

    // Tamper: corrupt prev_hash of entry 4 (sequence=4 = "Generate Rig")
    chain.entries[4].prev_hash =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();

    let result = chain.verify();
    assert!(
        matches!(
            result,
            Err(RefusalReason::ReceiptChainBroken { sequence: 5, .. })
        ),
        "Expected ReceiptChainBroken at sequence 5 after mutation, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: AuthorityState batch_update_damage_scalar integration bridge
#[test]
fn test_authority_state_damage_scalar_integration() {
    // Construct a small 4-part AuthorityState
    let mut state = AuthorityState::new(4);

    // Assign different heat/stress/socket_health values
    state.heat[0] = 2;
    state.stress[0] = 3;
    state.socket_health[0] = 14; // near full health

    state.heat[1] = 8;
    state.stress[1] = 6;
    state.socket_health[1] = 8;

    state.heat[2] = 14;
    state.stress[2] = 12;
    state.socket_health[2] = 2; // badly damaged

    state.heat[3] = 0;
    state.stress[3] = 0;
    state.socket_health[3] = 15; // MAX_CLASS

    // Validate lengths before update
    assert!(
        state.validate_lengths().is_ok(),
        "AuthorityState lengths should be valid"
    );

    // Run batch scalar damage update
    batch_update_damage_scalar(&mut state);

    // Verify damage values are within [0, MAX_CLASS]
    let class_violations = state.validate_classes();
    assert!(
        class_violations.is_empty(),
        "No class violations expected after damage update: {:?}",
        class_violations
    );

    // Spot-check: part[3] with all-zero heat/stress and full socket should have 0 damage
    // failure_risk = clamp((0 + 0 + (15 - 15)) / 3, 0, 15) = 0
    assert_eq!(
        state.damage[3], 0,
        "Full-health part should have 0 damage risk"
    );

    // Spot-check: part[2] with high heat+stress+degradation should have high damage
    // degradation = 15 - 2 = 13; failure_risk = (14 + 12 + 13) / 3 = 39 / 3 = 13
    assert_eq!(
        state.damage[2], 13,
        "Heavily degraded part should have damage class 13"
    );
}

/// VERIFIED_UNDER_SCOPE: Projection manifest with admitted row from SelectFrame receipt passes
#[test]
fn test_projection_manifest_with_select_frame_receipt() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/out.json")
        .expect("BLOCKED: out.json not found at /Users/sac/powlv2lsp/out.json");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    // Use the SelectFrame event's audit_receipt as the source receipt
    let select_frame_event = &log.events[0];
    assert_eq!(select_frame_event.event_type, "Select Frame");

    // The SelectFrame receipt from out.json
    let select_frame_receipt = select_frame_event.receipt.clone();
    assert!(
        !select_frame_receipt.is_empty(),
        "SelectFrame event must have an audit receipt"
    );

    // Build a projection manifest with one admitted row pointing to SelectFrame receipt
    let row = ProjectionRow {
        projection_id: "proj-mechbirth-frame-001".into(),
        source_powl_step: "Select Frame".into(),
        source_receipt: select_frame_receipt.clone(),
        object_id: "case-mechbirth-001".into(),
        projection_type: ProjectionType::SetMeshVariant,
        authority_inputs: vec!["damage_class".into(), "threat_class".into()],
        semantic_lod_class: "Primary".into(),
        ue4_target_surface: "SM_MechFrame_Heavy".into(),
        admission_status: ProjAdmission::Admitted,
    };

    let failures = validate_manifest(&[row]);
    assert!(
        failures.is_empty(),
        "Projection manifest with valid SelectFrame receipt should have no failures: {:?}",
        failures
    );
}

/// VERIFIED_UNDER_SCOPE: All 8 OCEL events produce non-empty receipts (not empty strings)
#[test]
fn test_ocel_events_have_non_empty_receipts() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/out.json")
        .expect("BLOCKED: out.json not found at /Users/sac/powlv2lsp/out.json");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    for event in &log.events {
        assert!(
            !event.receipt.is_empty(),
            "OCEL event '{}' should have a non-empty audit_receipt",
            event.event_type
        );
        assert_eq!(
            event.sequence,
            log.events.iter().position(|e| e.id == event.id).unwrap() as u64 + 1
        );
    }
}
