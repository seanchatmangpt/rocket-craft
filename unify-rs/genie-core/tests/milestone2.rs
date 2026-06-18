use genie_core::spec::{
    WorldSpec, Place, Actor, Object, Relationship, RelationshipType,
    Rule, RuleSeverity, HistoryEvent, Vector3, Bounds3D
};
use genie_core::laws::{WorldCoherenceGate, spec_to_triples, get_genie_shacl_shapes};
use genie_core::receipt_chain::ReceiptChainManager;
use genie_core::parse_intent;
use unify_rdf::shacl::validate as shacl_validate;

#[test]
fn test_milestone2_intent_parser_success() {
    let intent = r#"
        # This is a comment
        create place room_1 name "Control Room" at (0.0, 10.0, 0.0) bounds (10.0, 10.0, 5.0)
        
        create actor bot_1 name "Welder Bot" role RoboticWelder in room_1
        
        create object cnc_1 name "CNC Alpha" class CNC_Machine in room_1
        
        create relationship rel_1 contains from room_1 to bot_1
        
        create rule rule_1 name TempCheck expression "room_1.temp < 30" severity error
    "#;

    let res = parse_intent(intent);
    assert!(res.is_ok(), "Failed to parse intent: {:?}", res.err());
    let spec = res.unwrap();

    assert_eq!(spec.places.len(), 1);
    assert_eq!(spec.places[0].id, "room_1");
    assert_eq!(spec.places[0].name, "Control Room");
    assert_eq!(spec.places[0].bounds.center, Vector3::new(0.0, 10.0, 0.0));
    assert_eq!(spec.places[0].bounds.half_extents, Vector3::new(10.0, 10.0, 5.0));

    assert_eq!(spec.actors.len(), 1);
    assert_eq!(spec.actors[0].id, "bot_1");
    assert_eq!(spec.actors[0].name, "Welder Bot");
    assert_eq!(spec.actors[0].role, "RoboticWelder");
    assert_eq!(spec.actors[0].place_id, "room_1");

    assert_eq!(spec.objects.len(), 1);
    assert_eq!(spec.objects[0].id, "cnc_1");
    assert_eq!(spec.objects[0].name, "CNC Alpha");
    assert_eq!(spec.objects[0].class, "CNC_Machine");
    assert_eq!(spec.objects[0].place_id, "room_1");

    assert_eq!(spec.relationships.len(), 1);
    assert_eq!(spec.relationships[0].id, "rel_1");
    assert_eq!(spec.relationships[0].rel_type, RelationshipType::Contains);
    assert_eq!(spec.relationships[0].source, "room_1");
    assert_eq!(spec.relationships[0].target, "bot_1");

    assert_eq!(spec.rules.len(), 1);
    assert_eq!(spec.rules[0].id, "rule_1");
    assert_eq!(spec.rules[0].name, "TempCheck");
    assert_eq!(spec.rules[0].expression, "room_1.temp < 30");
    assert_eq!(spec.rules[0].severity, RuleSeverity::Error);
}

#[test]
fn test_milestone2_intent_parser_unsupported_severity() {
    let intent = "create rule rule_1 name TempCheck expression room_1.temp severity debug";
    let res = parse_intent(intent);
    assert!(res.is_err(), "Expected parsing to fail on unknown severity");
}

#[test]
fn test_milestone2_validation_gate_success() {
    let mut spec = WorldSpec::new();
    
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec.places.push(Place::new("room_1", "Control Room", bounds));
    spec.actors.push(Actor::new("bot_1", "Welder Bot", "RoboticWelder", "room_1"));
    spec.objects.push(Object::new("cnc_1", "CNC Alpha", "CNC_Machine", "room_1"));
    spec.relationships.push(Relationship::new("rel_1", RelationshipType::Contains, "room_1", "bot_1"));
    spec.rules.push(Rule::new("rule_1", "TempCheck", "room_1.temp < 30", RuleSeverity::Error));

    let gate = WorldCoherenceGate::new();
    let res = gate.validate(&spec);
    assert!(res.is_ok(), "Validation failed: {:?}", res.err());
}

#[test]
fn test_milestone2_validation_gate_finite_float_failure() {
    let gate = WorldCoherenceGate::new();

    // NaN center coordinates
    let mut spec = WorldSpec::new();
    let invalid_bounds = Bounds3D::new(Vector3::new(f32::NAN, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec.places.push(Place::new("room_1", "NaN Center Place", invalid_bounds));

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Floating-point Safety")), "Expected float safety error, got: {:?}", errs);

    // Infinity half extents
    let mut spec = WorldSpec::new();
    let invalid_bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(f32::INFINITY, 10.0, 10.0));
    spec.places.push(Place::new("room_2", "Inf Extent Place", invalid_bounds));

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Floating-point Safety")), "Expected float safety error, got: {:?}", errs);
}

#[test]
fn test_milestone2_validation_gate_containment_cycle_failure() {
    let gate = WorldCoherenceGate::new();

    // Cycle via parent_place_id: room_1 parent is room_2, room_2 parent is room_1
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    
    let mut p1 = Place::new("room_1", "Room 1", bounds);
    p1.parent_place_id = Some("room_2".to_string());
    let mut p2 = Place::new("room_2", "Room 2", bounds);
    p2.parent_place_id = Some("room_1".to_string());

    spec.places.push(p1);
    spec.places.push(p2);

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Containment")), "Expected containment cycle error, got: {:?}", errs);

    // Cycle via Contains relationships: room_1 Contains room_2, room_2 Contains room_1
    let mut spec = WorldSpec::new();
    spec.places.push(Place::new("room_1", "Room 1", bounds));
    spec.places.push(Place::new("room_2", "Room 2", bounds));
    spec.relationships.push(Relationship::new("rel_1", RelationshipType::Contains, "room_1", "room_2"));
    spec.relationships.push(Relationship::new("rel_2", RelationshipType::Contains, "room_2", "room_1"));

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Containment")), "Expected containment cycle error, got: {:?}", errs);
}

#[test]
fn test_milestone2_validation_gate_referential_integrity_failure() {
    let gate = WorldCoherenceGate::new();

    // Actor points to non-existent place
    let mut spec = WorldSpec::new();
    spec.actors.push(Actor::new("bot_1", "Welder", "Role", "ghost_room"));

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Referential Integrity")), "Expected integrity error, got: {:?}", errs);

    // Relationship refers to non-existent source
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec.places.push(Place::new("room_1", "Room", bounds));
    spec.relationships.push(Relationship::new("rel_1", RelationshipType::Contains, "ghost_source", "room_1"));

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Referential Integrity")), "Expected integrity error, got: {:?}", errs);
}

#[test]
fn test_milestone2_validation_gate_duplicate_ids_failure() {
    let gate = WorldCoherenceGate::new();

    // Duplicate place IDs
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec.places.push(Place::new("room_1", "Room 1", bounds));
    spec.places.push(Place::new("room_1", "Room 2", bounds));

    let res = gate.validate(&spec);
    assert!(res.is_err());
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| e.contains("Duplicate Entity ID")), "Expected duplicate ID error, got: {:?}", errs);
}

#[test]
fn test_milestone2_shacl_validation() {
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec.places.push(Place::new("room_1", "Control Room", bounds));
    spec.actors.push(Actor::new("bot_1", "Welder Bot", "RoboticWelder", "room_1"));

    let store = spec_to_triples(&spec);
    let shapes = get_genie_shacl_shapes();
    let result = shacl_validate(&store, &shapes);
    assert!(result.conforms);
    assert!(result.violations.is_empty());
}

#[test]
fn test_milestone2_receipt_chaining() {
    let mut spec = WorldSpec::new();
    spec.history.push(HistoryEvent::new("evt_1", 1000, "Boot"));
    spec.history.push(HistoryEvent::new("evt_2", 2000, "Weld"));
    spec.history.push(HistoryEvent::new("evt_3", 1500, "Check")); // Out of order timestamp

    let salt = b"genie_salt";
    
    // Generate receipt chain
    let res = ReceiptChainManager::generate_receipt_chain(&mut spec, salt);
    assert!(res.is_ok());
    assert_eq!(spec.receipts.len(), 3);

    // Verify receipt chain matches
    let verified = ReceiptChainManager::verify_receipt_chain(&spec, salt);
    assert!(verified);

    // Tamper with history and verify it fails
    spec.history[0].activity = "Tampered Activity".to_string();
    let verified_tampered = ReceiptChainManager::verify_receipt_chain(&spec, salt);
    assert!(!verified_tampered);
}
