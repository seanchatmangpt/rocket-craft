//! GC-MECHBIRTH-002: Unit tests for Receipt Chain, Projection Manifest, and Skin Stack.
//!
//! Bounded status vocabulary:
//! VERIFIED_UNDER_SCOPE / ALIVE_UNDER_SCOPE / PARTIAL_ALIVE_CANDIDATE / BLOCKED / RESIDUAL

use rocket_preue4_verifier::error::RefusalReason;
use rocket_preue4_verifier::projection::{
    AdmissionStatus as ProjAdmission, ProjectionRow, ProjectionType, validate_manifest,
};
use rocket_preue4_verifier::receipt::{AdmissionStatus, ReceiptChain};
use rocket_preue4_verifier::skin::{SkinLayer, SkinSpec, validate_skin_stack};

// ─────────────────────────────────────────────────────────────────────────────
// Receipt Chain Tests
// ─────────────────────────────────────────────────────────────────────────────

/// VERIFIED_UNDER_SCOPE: Empty chain verify returns Ok
#[test]
fn test_empty_chain_verify_ok() {
    let chain = ReceiptChain::default();
    assert!(chain.verify().is_ok(), "Empty chain should verify cleanly");
    assert!(
        chain.verify_hashes().is_ok(),
        "Empty chain hashes should verify cleanly"
    );
}

/// VERIFIED_UNDER_SCOPE: Append 3 events and verify chain integrity
#[test]
fn test_chain_three_events_verify_ok() {
    let mut chain = ReceiptChain::default();

    chain.append(
        "Select Frame".into(),
        vec!["case-1".into()],
        AdmissionStatus::Admitted,
        vec![],
    );
    chain.append(
        "Generate Socket Topology".into(),
        vec!["case-1".into()],
        AdmissionStatus::Admitted,
        vec![],
    );
    chain.append(
        "Generate Armor Panels".into(),
        vec!["case-1".into()],
        AdmissionStatus::Admitted,
        vec![],
    );

    assert_eq!(chain.entries.len(), 3);
    assert!(
        chain.verify().is_ok(),
        "3-event chain should verify cleanly"
    );
    assert!(
        chain.verify_hashes().is_ok(),
        "3-event chain hashes should verify cleanly"
    );
}

/// VERIFIED_UNDER_SCOPE: Sequence numbers are contiguous starting at 1
#[test]
fn test_sequence_contiguous() {
    let mut chain = ReceiptChain::default();
    chain.append("e1".into(), vec![], AdmissionStatus::Admitted, vec![]);
    chain.append("e2".into(), vec![], AdmissionStatus::Admitted, vec![]);
    chain.append("e3".into(), vec![], AdmissionStatus::Admitted, vec![]);

    assert_eq!(chain.entries[0].sequence, 1);
    assert_eq!(chain.entries[1].sequence, 2);
    assert_eq!(chain.entries[2].sequence, 3);
}

/// VERIFIED_UNDER_SCOPE: Mutating prev_hash of an entry breaks chain verification
#[test]
fn test_mutated_prev_hash_breaks_chain() {
    let mut chain = ReceiptChain::default();
    chain.append("e1".into(), vec![], AdmissionStatus::Admitted, vec![]);
    chain.append("e2".into(), vec![], AdmissionStatus::Admitted, vec![]);
    chain.append("e3".into(), vec![], AdmissionStatus::Admitted, vec![]);

    // Tamper: mutate prev_hash of entry[2]
    chain.entries[2].prev_hash =
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into();

    let result = chain.verify();
    assert!(
        matches!(
            result,
            Err(RefusalReason::ReceiptChainBroken { sequence: 3, .. })
        ),
        "Expected ReceiptChainBroken at sequence 3, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: Mutating the stored receipt field breaks verify_hashes
#[test]
fn test_mutated_receipt_field_breaks_verify_hashes() {
    let mut chain = ReceiptChain::default();
    chain.append("e1".into(), vec![], AdmissionStatus::Admitted, vec![]);
    chain.append("e2".into(), vec![], AdmissionStatus::Admitted, vec![]);

    // Tamper: corrupt the receipt hash of entry[0]
    chain.entries[0].receipt =
        "cafecafecafecafecafecafecafecafecafecafecafecafecafecafecafecafe".into();

    let result = chain.verify_hashes();
    assert!(
        matches!(
            result,
            Err(RefusalReason::ReceiptChainBroken { sequence: 1, .. })
        ),
        "Expected ReceiptChainBroken at sequence 1, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: First entry prev_hash is the zero-hash sentinel
#[test]
fn test_first_entry_prev_hash_is_zero() {
    let mut chain = ReceiptChain::default();
    chain.append("e1".into(), vec![], AdmissionStatus::Admitted, vec![]);
    assert_eq!(
        chain.entries[0].prev_hash,
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Projection Manifest Tests
// ─────────────────────────────────────────────────────────────────────────────

fn make_admitted_row(id: &str, receipt: &str, lod: &str, authority: Vec<String>) -> ProjectionRow {
    ProjectionRow {
        projection_id: id.into(),
        source_powl_step: "Select Frame".into(),
        source_receipt: receipt.into(),
        object_id: "mech-001".into(),
        projection_type: ProjectionType::SetMeshVariant,
        authority_inputs: authority,
        semantic_lod_class: lod.into(),
        ue4_target_surface: "SM_Mech_Body".into(),
        admission_status: ProjAdmission::Admitted,
    }
}

/// VERIFIED_UNDER_SCOPE: Admitted row with non-empty receipt → Ok
#[test]
fn test_admitted_row_with_receipt_ok() {
    let row = make_admitted_row("proj-001", "abc123hash", "Secondary", vec!["heat".into()]);
    assert!(row.validate().is_ok(), "Valid admitted row should pass");
}

/// VERIFIED_UNDER_SCOPE: Admitted row with empty source_receipt → Err(OrphanProjectionRow)
#[test]
fn test_admitted_row_empty_receipt_orphan_error() {
    let row = make_admitted_row("proj-002", "", "Secondary", vec!["heat".into()]);
    let result = row.validate();
    assert!(
        matches!(result, Err(RefusalReason::OrphanProjectionRow { ref row_id }) if row_id == "proj-002"),
        "Expected OrphanProjectionRow for proj-002, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: Crown row with no authority_inputs → Err(OrphanProjectionRow)
#[test]
fn test_crown_row_missing_authority_inputs_error() {
    let row = make_admitted_row("proj-crown-001", "somehash", "Crown", vec![]);
    let result = row.validate();
    assert!(
        matches!(result, Err(RefusalReason::OrphanProjectionRow { .. })),
        "Expected OrphanProjectionRow for crown row missing authority_inputs, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: Refused row with empty source_receipt is acceptable
#[test]
fn test_refused_row_empty_receipt_ok() {
    let row = ProjectionRow {
        projection_id: "proj-refused-001".into(),
        source_powl_step: "Select Frame".into(),
        source_receipt: "".into(),
        object_id: "mech-001".into(),
        projection_type: ProjectionType::SetMeshVariant,
        authority_inputs: vec![],
        semantic_lod_class: "Background".into(),
        ue4_target_surface: "SM_Mech_Arm".into(),
        admission_status: ProjAdmission::Refused,
    };
    assert!(
        row.validate().is_ok(),
        "Refused row with empty receipt is OK"
    );
}

/// VERIFIED_UNDER_SCOPE: validate_manifest returns no failures for valid rows
#[test]
fn test_validate_manifest_clean() {
    let rows = vec![
        make_admitted_row("proj-a", "hash-a", "Primary", vec!["damage".into()]),
        make_admitted_row("proj-b", "hash-b", "Secondary", vec!["heat".into()]),
    ];
    let failures = validate_manifest(&rows);
    assert!(
        failures.is_empty(),
        "Valid manifest should have no failures: {:?}",
        failures
    );
}

/// VERIFIED_UNDER_SCOPE: validate_manifest returns all orphan failures
#[test]
fn test_validate_manifest_multiple_orphans() {
    let rows = vec![
        make_admitted_row("proj-orphan-1", "", "Secondary", vec!["damage".into()]),
        make_admitted_row("proj-orphan-2", "", "Secondary", vec!["heat".into()]),
    ];
    let failures = validate_manifest(&rows);
    assert_eq!(
        failures.len(),
        2,
        "Should detect 2 orphan rows, got: {:?}",
        failures
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Skin Stack Tests
// ─────────────────────────────────────────────────────────────────────────────

fn base_skin_spec() -> SkinSpec {
    SkinSpec {
        layers: vec![SkinLayer::BaseMaterial],
        damage_class_binding: 0,
        heat_class_binding: 0,
        has_thermal_vent_visible: true,
        sponsor_livery_present: false,
        repair_receipt: None,
    }
}

/// VERIFIED_UNDER_SCOPE: Valid skin stack passes
#[test]
fn test_valid_skin_stack_ok() {
    let spec = base_skin_spec();
    assert!(
        validate_skin_stack(&spec).is_ok(),
        "Base skin spec should be valid"
    );
}

/// VERIFIED_UNDER_SCOPE: SponsorLivery without ThermalZones → Err
#[test]
fn test_sponsor_livery_without_thermal_zones_error() {
    let spec = SkinSpec {
        layers: vec![SkinLayer::BaseMaterial, SkinLayer::SponsorLivery],
        sponsor_livery_present: true,
        has_thermal_vent_visible: true,
        ..base_skin_spec()
    };
    let result = validate_skin_stack(&spec);
    assert!(
        matches!(result, Err(RefusalReason::SkinOccludesRequiredFeature { ref feature }) if feature.contains("ThermalZones")),
        "Expected ThermalZones error, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: SponsorLivery hiding thermal vent → Err
#[test]
fn test_sponsor_livery_hides_thermal_vent_error() {
    let spec = SkinSpec {
        layers: vec![
            SkinLayer::BaseMaterial,
            SkinLayer::ThermalZones,
            SkinLayer::SponsorLivery,
        ],
        sponsor_livery_present: true,
        has_thermal_vent_visible: false,
        ..base_skin_spec()
    };
    let result = validate_skin_stack(&spec);
    assert!(
        matches!(result, Err(RefusalReason::SkinOccludesRequiredFeature { ref feature }) if feature.contains("thermal_vent")),
        "Expected thermal_vent error, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: DamageMasks without damage class binding → Err
#[test]
fn test_damage_masks_without_binding_error() {
    let spec = SkinSpec {
        layers: vec![SkinLayer::BaseMaterial, SkinLayer::DamageMasks],
        damage_class_binding: 0,
        ..base_skin_spec()
    };
    let result = validate_skin_stack(&spec);
    assert!(
        matches!(result, Err(RefusalReason::SkinOccludesRequiredFeature { ref feature }) if feature.contains("DamageMasks")),
        "Expected DamageMasks error, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: RepairResidue without repair receipt → Err
#[test]
fn test_repair_residue_without_receipt_error() {
    let spec = SkinSpec {
        layers: vec![SkinLayer::BaseMaterial, SkinLayer::RepairResidue],
        repair_receipt: None,
        ..base_skin_spec()
    };
    let result = validate_skin_stack(&spec);
    assert!(
        matches!(result, Err(RefusalReason::SkinOccludesRequiredFeature { ref feature }) if feature.contains("RepairResidue")),
        "Expected RepairResidue error, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: FactionPalette without BaseMaterial → Err
#[test]
fn test_faction_palette_without_base_material_error() {
    let spec = SkinSpec {
        layers: vec![SkinLayer::FactionPalette],
        ..base_skin_spec()
    };
    let result = validate_skin_stack(&spec);
    assert!(
        matches!(result, Err(RefusalReason::SkinOccludesRequiredFeature { ref feature }) if feature.contains("BaseMaterial")),
        "Expected BaseMaterial error, got: {:?}",
        result
    );
}

/// VERIFIED_UNDER_SCOPE: Full valid stack with all valid layers
#[test]
fn test_full_valid_skin_stack() {
    let spec = SkinSpec {
        layers: vec![
            SkinLayer::BaseMaterial,
            SkinLayer::ThermalZones,
            SkinLayer::SponsorLivery,
            SkinLayer::DamageMasks,
            SkinLayer::RepairResidue,
        ],
        damage_class_binding: 5,
        heat_class_binding: 3,
        has_thermal_vent_visible: true,
        sponsor_livery_present: true,
        repair_receipt: Some("receipt-abc123".into()),
    };
    assert!(
        validate_skin_stack(&spec).is_ok(),
        "Full valid skin stack should pass"
    );
}

/// VERIFIED_UNDER_SCOPE: Residual status entry in chain verify_hashes passes
#[test]
fn test_residual_status_entry_in_chain() {
    let mut chain = ReceiptChain::default();
    chain.append("e1".into(), vec![], AdmissionStatus::Admitted, vec![]);
    chain.append(
        "e2_residual".into(),
        vec![],
        AdmissionStatus::Residual,
        vec!["partial_output".into()],
    );
    chain.append("e3".into(), vec![], AdmissionStatus::Admitted, vec![]);

    assert!(
        chain.verify().is_ok(),
        "Chain with Residual entry should verify"
    );
    assert!(
        chain.verify_hashes().is_ok(),
        "Chain with Residual entry hashes should verify"
    );
}
