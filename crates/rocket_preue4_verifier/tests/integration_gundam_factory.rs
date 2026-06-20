//! GC-GUNDAM-FACTORY-001: Integration test — Gundam Factory OCEL trace replay.

use rocket_preue4_verifier::authority::AuthorityState;
use rocket_preue4_verifier::error::RefusalReason;
use rocket_preue4_verifier::ocel::OcelLog;
use rocket_preue4_verifier::projection::{
    AdmissionStatus as ProjAdmission, ProjectionRow, ProjectionType, validate_manifest,
};
use rocket_preue4_verifier::receipt::{AdmissionStatus, ReceiptChain};
use rocket_preue4_verifier::transitions::batch_update_damage_scalar;

/// Expected Gundam Factory POWL step event types (must match gundam_factory_trace.json exactly).
const EXPECTED_EVENT_TYPES: &[&str] = &[
    "Spawn",
    "Factory Entrance",
    "Frame Assembly",
    "Socket Topology",
    "Armor Skin Station",
    "Rig Motion Station",
    "Verification Gate",
    "Receipt Terminal",
    "Exit Or Loop",
];

/// VERIFIED_UNDER_SCOPE: Parse real Gundam Factory OCEL trace and verify 9 events
#[test]
fn test_ocel_parse_real_gundam_trace() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/gundam_factory_trace.json")
        .expect("BLOCKED: gundam_factory_trace.json not found");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    assert_eq!(
        log.events.len(),
        9,
        "Expected exactly 9 Gundam Factory POWL events, found {}",
        log.events.len()
    );
}

/// VERIFIED_UNDER_SCOPE: Parsed events match expected Gundam Factory POWL step types
#[test]
fn test_ocel_event_types_match_gundam_factory_powl() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/gundam_factory_trace.json")
        .expect("BLOCKED: gundam_factory_trace.json not found");

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
fn test_receipt_chain_mirrors_gundam_ocel_events() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/gundam_factory_trace.json")
        .expect("BLOCKED: gundam_factory_trace.json not found");

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

    assert_eq!(chain.entries.len(), 9, "Chain should have 9 entries");
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
fn test_receipt_chain_mutation_produces_broken_error_gundam() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/gundam_factory_trace.json")
        .expect("BLOCKED: gundam_factory_trace.json not found");

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

    // Tamper: corrupt prev_hash of entry 4 (sequence=4 = "Socket Topology")
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

/// VERIFIED_UNDER_SCOPE: Projection manifest with admitted row from Spawn receipt passes
#[test]
fn test_projection_manifest_with_spawn_receipt() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/gundam_factory_trace.json")
        .expect("BLOCKED: gundam_factory_trace.json not found");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    let spawn_event = &log.events[0];
    assert_eq!(spawn_event.event_type, "Spawn");

    let spawn_receipt = spawn_event.receipt.clone();
    assert!(
        !spawn_receipt.is_empty(),
        "Spawn event must have an audit receipt"
    );

    // Build a projection manifest with one admitted row pointing to Spawn receipt
    let row = ProjectionRow {
        projection_id: "proj-gundam-spawn-001".into(),
        source_powl_step: "Spawn".into(),
        source_receipt: spawn_receipt.clone(),
        object_id: spawn_event.objects[0].clone(),
        projection_type: ProjectionType::SetMeshVariant,
        authority_inputs: vec!["damage_class".into(), "threat_class".into()],
        semantic_lod_class: "Primary".into(),
        ue4_target_surface: "SM_GundamFrame".into(),
        admission_status: ProjAdmission::Admitted,
    };

    let failures = validate_manifest(&[row]);
    assert!(
        failures.is_empty(),
        "Projection manifest with valid Spawn receipt should have no failures: {:?}",
        failures
    );
}

/// VERIFIED_UNDER_SCOPE: All 9 OCEL events produce non-empty receipts and contain dynamic case id
#[test]
fn test_gundam_ocel_events_have_non_empty_receipts_and_correct_case_id() {
    let json = std::fs::read_to_string("/Users/sac/powlv2lsp/gundam_factory_trace.json")
        .expect("BLOCKED: gundam_factory_trace.json not found");

    let log = OcelLog::from_powlv2lsp_trace(&json).expect("BLOCKED: Failed to parse OCEL trace");

    for event in &log.events {
        assert!(
            !event.receipt.is_empty(),
            "OCEL event '{}' should have a non-empty audit_receipt",
            event.event_type
        );
        assert!(
            event.objects[0].starts_with("case-"),
            "OCEL event object id '{}' should start with 'case-'",
            event.objects[0]
        );
    }
}
