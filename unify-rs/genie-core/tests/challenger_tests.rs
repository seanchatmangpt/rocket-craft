use genie_core::laws::WorldCoherenceGate;
use genie_core::parse_intent;
use genie_core::receipt_chain::ReceiptChainManager;
use genie_core::spec::{Bounds3D, HistoryEvent, Place, Rule, RuleSeverity, Vector3, WorldSpec};
use unify_receipts::receipt::Receipt;

#[test]
fn test_dash_in_id_referential_integrity_bypass() {
    let gate = WorldCoherenceGate::new();

    // 1. Check with alphanumeric/underscore ID that DOES NOT exist (should fail validation)
    let mut spec1 = WorldSpec::new();
    spec1.rules.push(Rule::new(
        "rule_1",
        "Rule 1",
        "nonexistent_room.temp < 30",
        RuleSeverity::Error,
    ));
    let res1 = gate.validate(&spec1);
    assert!(
        res1.is_err(),
        "Expected validation to fail for nonexistent_room"
    );
    let errs1 = res1.unwrap_err();
    assert!(
        errs1
            .iter()
            .any(|e| e.contains("references non-existent entity")),
        "Expected error about non-existent entity, got: {:?}",
        errs1
    );

    // 2. Check with dash ID that DOES NOT exist (should fail validation but passes due to the bug!)
    let mut spec2 = WorldSpec::new();
    spec2.rules.push(Rule::new(
        "rule_2",
        "Rule 2",
        "nonexistent-room.temp < 30",
        RuleSeverity::Error,
    ));
    let res2 = gate.validate(&spec2);

    // If there is a bug, res2 will be Ok(()), meaning the invalid referential integrity is bypassed!
    if res2.is_ok() {
        println!(
            "BUG CONFIRMED: nonexistent-room (with dash) bypasses referential integrity check!"
        );
    } else {
        println!("No bug: res2 errors: {:?}", res2.err());
    }
}

#[test]
fn test_prefix_expression_referential_integrity_bypass() {
    let gate = WorldCoherenceGate::new();

    // 3. Check with alphanumeric ID but prefixed by comparison (e.g., "30 > nonexistent_room.temp")
    // This should fail validation, but will it pass because of the dot search logic?
    let mut spec3 = WorldSpec::new();
    spec3.rules.push(Rule::new(
        "rule_3",
        "Rule 3",
        "30 > nonexistent_room.temp",
        RuleSeverity::Error,
    ));
    let res3 = gate.validate(&spec3);

    if res3.is_ok() {
        println!(
            "BUG CONFIRMED: '30 > nonexistent_room.temp' bypasses referential integrity check!"
        );
    } else {
        println!("No bug: res3 errors: {:?}", res3.err());
    }
}

#[test]
fn test_self_containment_via_parent_place_id() {
    let gate = WorldCoherenceGate::new();

    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::default(), Vector3::new(10.0, 10.0, 10.0));
    let mut place = Place::new("room_1", "Self Contain Room", bounds);
    place.parent_place_id = Some("room_1".to_string());
    spec.places.push(place);

    let res = gate.validate(&spec);
    assert!(res.is_err(), "Expected self-containment to fail validation");
    let errs = res.unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("Cyclic Containment")),
        "Expected containment cycle error, got: {:?}",
        errs
    );
    println!(
        "SUCCESS: Self-containment via parent_place_id is correctly caught: {:?}",
        errs
    );
}

#[test]
fn test_parser_allows_infinity_via_scientific_notation() {
    let intent = r#"
        create place room_1 name "Control Room" at (1e40, 0.0, 0.0) bounds (10.0, 10.0, 5.0)
    "#;

    let res = parse_intent(intent);
    assert!(
        res.is_ok(),
        "Expected intent parser to succeed even with 1e40"
    );
    let spec = res.unwrap();
    let x_coord = spec.places[0].bounds.center.x;

    // 1e40 is parsed as f32::INFINITY
    assert_eq!(x_coord, f32::INFINITY);
    println!("BUG CONFIRMED: Intent parser allowed parsing 1e40 which evaluated to f32::INFINITY");

    // The validation gate should catch this
    let gate = WorldCoherenceGate::new();
    let val_res = gate.validate(&spec);
    assert!(
        val_res.is_err(),
        "Expected validation gate to catch the INFINITY value"
    );
    let errs = val_res.unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("Floating-point Safety")),
        "Expected float safety error, got: {:?}",
        errs
    );
    println!("SUCCESS: Validation gate correctly caught the parsed f32::INFINITY");
}

#[test]
fn test_multiple_entities_referential_integrity_bypass() {
    let gate = WorldCoherenceGate::new();

    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::default(), Vector3::new(10.0, 10.0, 10.0));
    spec.places.push(Place::new("room_1", "Room 1", bounds));

    // room_1 exists, but nonexistent_room does not.
    // Since nonexistent_room is after the first dot, does it bypass the check?
    spec.rules.push(Rule::new(
        "rule_1",
        "Rule 1",
        "room_1.temp < 30 && nonexistent_room.temp < 20",
        RuleSeverity::Error,
    ));
    let res = gate.validate(&spec);

    if res.is_ok() {
        println!(
            "BUG CONFIRMED: Multiple entities in expression bypass referential integrity check!"
        );
    } else {
        println!("No bug: res errors: {:?}", res.err());
    }
}

#[test]
fn test_receipt_chain_tail_tampering_without_salt() {
    let mut spec = WorldSpec::new();
    spec.history.push(HistoryEvent::new("evt_1", 1000, "Boot"));
    spec.history.push(HistoryEvent::new("evt_2", 2000, "Weld"));
    spec.history.push(HistoryEvent::new("evt_3", 3000, "Check"));

    let salt = b"very_secret_salt_12345";

    // Generate valid receipt chain
    let res = ReceiptChainManager::generate_receipt_chain(&mut spec, salt);
    assert!(res.is_ok());
    assert_eq!(spec.receipts.len(), 3);
    assert!(ReceiptChainManager::verify_receipt_chain(&spec, salt));

    // Now, we want to tamper with the last event (evt_3) without knowing the secret salt.
    // We change the activity to "Explode"
    spec.history[2].activity = "Explode".to_string();

    // Recompute only the last receipt's hash using the public hash of receipt 1 (idx=1)
    let prev_hash = spec.receipts[1].hash.as_bytes();
    let event_bytes = serde_json::to_vec(&spec.history[2]).unwrap();
    let mut data = Vec::with_capacity(prev_hash.len() + event_bytes.len());
    data.extend_from_slice(prev_hash);
    data.extend_from_slice(&event_bytes);

    // Create new receipt for tampered event
    let tampered_receipt = Receipt::new("history_receipt_evt_3", &data);
    spec.receipts[2] = tampered_receipt;

    // Verify the chain again using the secret salt
    let verified = ReceiptChainManager::verify_receipt_chain(&spec, salt);
    if verified {
        println!("BUG CONFIRMED: Receipt chain was successfully tampered with at the tail without knowing the secret salt!");
    } else {
        println!("No bug: verification failed as expected");
    }
    assert!(!verified);
}
