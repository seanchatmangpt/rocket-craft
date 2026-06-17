use genie_core::spec::{
    WorldSpec, Place, Actor, Object, RelationshipType,
    Rule, RuleSeverity, ProcessStep,
    Vector3, Bounds3D
};
use std::collections::HashMap;

#[test]
fn test_missing_placement_in_serialization() {
    // If "placement" is missing from Actor JSON, deserialization should succeed
    // because placement has #[serde(default)].
    let actor_json_missing_placement = r#"{
        "id": "actor_1",
        "name": "Welder 1",
        "role": "RoboticWelder",
        "place_id": "room_1",
        "properties": {}
    }"#;

    let res: Result<Actor, serde_json::Error> = serde_json::from_str(actor_json_missing_placement);
    assert!(
        res.is_ok(),
        "Expected deserialization of Actor to succeed when placement is missing: {:?}",
        res
    );
    let actor = res.unwrap();
    assert_eq!(actor.placement.position, Vector3::default());
    assert_eq!(actor.placement.rotation, Vector3::default());

    // Similarly for Object
    let object_json_missing_placement = r#"{
        "id": "obj_1",
        "name": "CNC A",
        "class": "CNC",
        "place_id": "room_1",
        "properties": {},
        "tags": []
    }"#;

    let res_obj: Result<Object, serde_json::Error> = serde_json::from_str(object_json_missing_placement);
    assert!(
        res_obj.is_ok(),
        "Expected deserialization of Object to succeed when placement is missing: {:?}",
        res_obj
    );
    let object = res_obj.unwrap();
    assert_eq!(object.placement.position, Vector3::default());
    assert_eq!(object.placement.rotation, Vector3::default());
}

#[test]
fn test_custom_relationship_serialization() {
    // Custom relationship type serialization format:
    let rel_type = RelationshipType::Custom("custom_op".to_string());
    let serialized = serde_json::to_string(&rel_type).unwrap();
    
    // Default serde representation for enums with payloads is externally tagged: {"custom":"custom_op"}
    assert_eq!(serialized, r#"{"custom":"custom_op"}"#);

    // Let's deserialize it back
    let deserialized: RelationshipType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, rel_type);

    // Test standard/unit relationship types: they serialize to plain strings
    let unit_rel = RelationshipType::Contains;
    let unit_serialized = serde_json::to_string(&unit_rel).unwrap();
    assert_eq!(unit_serialized, r#""contains""#);

    let unit_deserialized: RelationshipType = serde_json::from_str(&unit_serialized).unwrap();
    assert_eq!(unit_deserialized, unit_rel);
}

#[test]
fn test_floating_point_extreme_boundaries() {
    let mut spec = WorldSpec::new();

    // Extreme floating-point values
    let extreme_pos = Vector3::new(f32::MAX, f32::MIN, f32::EPSILON);
    let bounds = Bounds3D::new(extreme_pos, Vector3::new(f32::MAX, f32::MAX, f32::MAX));
    let place = Place::new("p1", "Extreme Place", bounds);
    spec.places.push(place);

    let json = serde_json::to_string(&spec).unwrap();
    let loaded: WorldSpec = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.places[0].bounds.center.x, f32::MAX);
    assert_eq!(loaded.places[0].bounds.center.y, f32::MIN);
    assert_eq!(loaded.places[0].bounds.center.z, f32::EPSILON);
}

#[test]
fn test_rule_empty_expression() {
    let rule = Rule::new("", "", "", RuleSeverity::Info);
    assert_eq!(rule.id, "");
    assert_eq!(rule.name, "");
    assert_eq!(rule.expression, "");

    let serialized = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, "");
    assert_eq!(deserialized.expression, "");
}

#[test]
fn test_step_number_zero_and_negative_duration() {
    // Process step numbers and durations are not validated at struct creation
    let step = ProcessStep::new(0, "Invalid Step", -0.01);
    assert_eq!(step.step_number, 0);
    assert_eq!(step.duration_seconds, -0.01);

    let serialized = serde_json::to_string(&step).unwrap();
    let deserialized: ProcessStep = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.step_number, 0);
    assert_eq!(deserialized.duration_seconds, -0.01);
}

#[test]
fn test_complex_json_properties() {
    let mut actor = Actor::new("actor_1", "Welder", "Robot", "room_1");
    
    // Properties allow any JSON values (objects, nested arrays, booleans, nulls)
    let mut props = HashMap::new();
    props.insert("nested_obj".to_string(), serde_json::json!({
        "status": "online",
        "tasks": ["weld", "move"],
        "telemetry": {
            "voltage": 240,
            "temp_c": [12.5, 13.0, 14.1]
        }
    }));
    props.insert("is_active".to_string(), serde_json::json!(true));
    props.insert("null_val".to_string(), serde_json::json!(null));
    
    actor.properties = props;

    let serialized = serde_json::to_string(&actor).unwrap();
    let deserialized: Actor = serde_json::from_str(&serialized).unwrap();

    assert_eq!(
        deserialized.properties.get("is_active").unwrap(),
        &serde_json::json!(true)
    );
    assert_eq!(
        deserialized.properties.get("null_val").unwrap(),
        &serde_json::json!(null)
    );
    
    let nested = deserialized.properties.get("nested_obj").unwrap();
    assert_eq!(nested["status"], "online");
    assert_eq!(nested["telemetry"]["voltage"], 240);
    assert_eq!(nested["telemetry"]["temp_c"][1], 13.0);
}

#[test]
fn test_regex_parser_place_truncation_loophole() {
    // A place name that mimics the coords pattern will fail to parse under the new non-lazy regex
    let intent = "create place room_1 name \"Main Room at (1.0, 2.0, 3.0) bounds (10.0, 20.0, 30.0)";
    let spec_res = genie_core::parse_intent(intent);
    assert!(spec_res.is_err(), "Expected intent to fail to parse because of name hijacking protection");
}

#[test]
fn test_regex_parser_actor_truncation_loophole() {
    // An actor name containing the role/in keywords will fail to parse under the new non-lazy regex
    let intent = "create actor bot_1 name \"Welder Bot role RoboticWelder in room_1\"";
    let spec_res = genie_core::parse_intent(intent);
    assert!(spec_res.is_err(), "Expected intent to fail to parse because of name hijacking protection");
}


#[test]
fn test_parser_fails_on_inline_comment() {
    // Natural language parser does not support inline comments, failing with a parse error
    let intent = "create actor bot_1 name \"Welder\" role RoboticWelder in room_1 # inline comment";
    let res = genie_core::parse_intent(intent);
    assert!(res.is_err());
}

#[test]
fn test_validation_gate_rule_expression_referential_integrity_gap() {
    use genie_core::laws::WorldCoherenceGate;

    let gate = WorldCoherenceGate::new();

    // Case 1: First entity exists, but second entity (ghost_room) does not.
    let mut spec1 = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec1.places.push(Place::new("room_1", "Control Room", bounds));
    spec1.rules.push(Rule::new("rule_1", "TempCheck", "room_1.temp < 30 && ghost_room.temp < 40", RuleSeverity::Error));

    let res1 = gate.validate(&spec1);
    assert!(res1.is_err(), "Expected referential integrity failure for room_1 and ghost_room");
    let errs1 = res1.unwrap_err();
    assert!(errs1.iter().any(|e| e.contains("references non-existent entity 'ghost_room'")));

    // Case 2: Expression starts with non-alphanumeric token, referring to non-existent entity later.
    let mut spec2 = WorldSpec::new();
    spec2.rules.push(Rule::new("rule_2", "TempCheck", "30 > ghost_room.temp", RuleSeverity::Error));

    let res2 = gate.validate(&spec2);
    assert!(res2.is_err(), "Expected referential integrity failure for ghost_room");
    let errs2 = res2.unwrap_err();
    assert!(errs2.iter().any(|e| e.contains("references non-existent entity 'ghost_room'")));
}

#[test]
fn test_receipt_metadata_tampering_vulnerability() {
    use genie_core::receipt_chain::ReceiptChainManager;

    let mut spec = WorldSpec::new();
    spec.history.push(genie_core::spec::HistoryEvent::new("evt_1", 1000, "Boot"));

    let salt = b"genie_salt";
    assert!(ReceiptChainManager::generate_receipt_chain(&mut spec, salt).is_ok());
    assert_eq!(spec.receipts.len(), 1);

    // Tamper with the issued_at timestamp metadata
    spec.receipts[0].issued_at = 9999999999999;

    // The receipt verification passes despite the tampered issued_at metadata because verify() does not validate it
    let verified = ReceiptChainManager::verify_receipt_chain(&spec, salt);
    assert!(verified, "Receipt verification should pass despite tampered metadata");
}

