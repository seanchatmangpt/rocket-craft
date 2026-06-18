use genie_core::spec::{
    Actor, Bounds3D, HistoryEvent, Object, Place, Placement, Process, ProcessStep, Relationship,
    RelationshipType, Rule, RuleSeverity, Vector3, WorldSpec,
};

#[test]
fn test_empty_and_whitespace_intents() {
    // Empty intent
    assert!(genie_core::parse_intent("").is_err());
    assert!(genie_core::parse_intent("   ").is_err());
    assert!(genie_core::parse_intent("\n\t").is_err());
}

#[test]
fn test_malformed_json_deserialization() {
    // Invalid JSON structure
    let bad_json = r#"{ "places": [ { "id": "p1", "name": "Room" "#; // truncated
    let res: Result<WorldSpec, serde_json::Error> = serde_json::from_str(bad_json);
    assert!(res.is_err());

    // Place missing bounds
    let missing_bounds_json = r#"{
        "places": [
            {
                "id": "p1",
                "name": "Room"
            }
        ]
    }"#;
    let res: Result<WorldSpec, serde_json::Error> = serde_json::from_str(missing_bounds_json);
    assert!(res.is_err());

    // Place with wrong type for bounds
    let wrong_bounds_type_json = r#"{
        "places": [
            {
                "id": "p1",
                "name": "Room",
                "bounds": "invalid_bounds"
            }
        ]
    }"#;
    let res: Result<WorldSpec, serde_json::Error> = serde_json::from_str(wrong_bounds_type_json);
    assert!(res.is_err());
}

#[test]
fn test_duplicate_ids_permitted() {
    // Current model uses Vec without uniqueness checks, so duplicate IDs are allowed.
    // We document this behavior here as a known constraint/vulnerability of Milestone 1.
    let mut spec = WorldSpec::new();

    let bounds = Bounds3D::default();
    spec.places
        .push(Place::new("room_1", "Control Room A", bounds));
    spec.places
        .push(Place::new("room_1", "Control Room B", bounds)); // duplicate ID

    assert_eq!(spec.places.len(), 2);
    assert_eq!(spec.places[0].id, spec.places[1].id);

    spec.actors
        .push(Actor::new("actor_1", "Welder 1", "RoboticWelder", "room_1"));
    spec.actors
        .push(Actor::new("actor_1", "Welder 2", "RoboticWelder", "room_1")); // duplicate ID

    assert_eq!(spec.actors.len(), 2);
    assert_eq!(spec.actors[0].id, spec.actors[1].id);

    spec.objects
        .push(Object::new("obj_1", "CNC A", "CNC", "room_1"));
    spec.objects
        .push(Object::new("obj_1", "CNC B", "CNC", "room_1")); // duplicate ID

    assert_eq!(spec.objects.len(), 2);
    assert_eq!(spec.objects[0].id, spec.objects[1].id);

    spec.relationships.push(Relationship::new(
        "rel_1",
        RelationshipType::Contains,
        "room_1",
        "actor_1",
    ));
    spec.relationships.push(Relationship::new(
        "rel_1",
        RelationshipType::Contains,
        "room_1",
        "obj_1",
    )); // duplicate ID

    assert_eq!(spec.relationships.len(), 2);
    assert_eq!(spec.relationships[0].id, spec.relationships[1].id);

    spec.rules
        .push(Rule::new("rule_1", "Rule A", "true", RuleSeverity::Error));
    spec.rules.push(Rule::new(
        "rule_1",
        "Rule B",
        "false",
        RuleSeverity::Warning,
    )); // duplicate ID

    assert_eq!(spec.rules.len(), 2);
    assert_eq!(spec.rules[0].id, spec.rules[1].id);

    spec.processes.push(Process::new("proc_1", "Process A"));
    spec.processes.push(Process::new("proc_1", "Process B")); // duplicate ID

    assert_eq!(spec.processes.len(), 2);
    assert_eq!(spec.processes[0].id, spec.processes[1].id);
}

#[test]
fn test_floating_point_boundaries_serialization() {
    // 1. NaN and Infinities in coordinates
    let nan_vec = Vector3::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
    let placement = Placement::new(nan_vec, Vector3::default());
    let mut actor = Actor::new("actor_1", "Ghost Welder", "RoboticWelder", "room_1");
    actor.placement = placement;

    let mut spec = WorldSpec::new();
    spec.actors.push(actor);

    // Let's see if serialization succeeds. By default, standard serde_json
    // serialization of NaN and Infinity returns an error because JSON specification
    // does not allow them.
    let serialized_res = serde_json::to_string(&spec);

    // We document how serde_json behaves out of the box with NaN / Infinity.
    // If it fails, it's a serialization vulnerability/bug unless caught.
    // If it succeeds, it might produce `null` in JSON which then fails deserialization.
    match &serialized_res {
        Ok(json_str) => {
            println!("Serialized with NaN/Inf: {}", json_str);
            // Let's try to deserialize it back
            let deserialized_res: Result<WorldSpec, _> = serde_json::from_str(json_str);
            if let Err(e) = deserialized_res {
                println!("Deserialization failed on serialized JSON: {:?}", e);
            }
        }
        Err(e) => {
            println!("Serialization failed on NaN/Inf: {:?}", e);
        }
    }
}

#[test]
fn test_negative_values_and_physical_boundaries() {
    let mut spec = WorldSpec::new();

    // Bounds3D with negative half extents
    let invalid_bounds = Bounds3D::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(-10.0, -10.0, -10.0), // physically invalid negative half-extents
    );
    spec.places
        .push(Place::new("room_1", "Negative Room", invalid_bounds));

    // Process step with negative duration
    let mut process = Process::new("proc_1", "Process");
    let invalid_step = ProcessStep::new(1, "Step 1", -60.0); // negative duration
    process.steps.push(invalid_step);
    spec.processes.push(process);

    // Verify they serialize and deserialize without any complaints
    let json = serde_json::to_string(&spec).unwrap();
    let loaded: WorldSpec = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.places[0].bounds.half_extents.x, -10.0);
    assert_eq!(loaded.processes[0].steps[0].duration_seconds, -60.0);
}

#[test]
fn test_referential_integrity_violations() {
    let mut spec = WorldSpec::new();

    // Relationship refers to non-existent source and target
    spec.relationships.push(Relationship::new(
        "rel_1",
        RelationshipType::Contains,
        "non_existent_place",
        "non_existent_actor",
    ));

    // Actor refers to non-existent place
    spec.actors.push(Actor::new(
        "actor_1",
        "Welder",
        "RoboticWelder",
        "non_existent_place",
    ));

    // Object refers to non-existent place
    spec.objects.push(Object::new(
        "obj_1",
        "CNC",
        "CNC_Machine",
        "non_existent_place",
    ));

    // Rule refers to non-existent entities in expression
    spec.rules.push(Rule::new(
        "rule_1",
        "Invalid Rule",
        "non_existent_place.temperature > 50",
        RuleSeverity::Error,
    ));

    // Process step assigned actor is non-existent
    let mut process = Process::new("proc_1", "Process");
    let mut step = ProcessStep::new(1, "Step", 10.0);
    step.assigned_actor = Some("non_existent_actor".to_string());
    step.inputs.push("non_existent_input".to_string());
    step.outputs.push("non_existent_output".to_string());
    process.steps.push(step);
    spec.processes.push(process);

    // History event refers to non-existent actor
    let mut event = HistoryEvent::new("evt_1", 1000, "Boot");
    event.actor_id = Some("non_existent_actor".to_string());
    spec.history.push(event);

    // Verify it is completely constructible and serializable
    let json = serde_json::to_string(&spec).unwrap();
    let loaded: WorldSpec = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.relationships[0].source, "non_existent_place");
    assert_eq!(loaded.actors[0].place_id, "non_existent_place");
    assert_eq!(loaded.objects[0].place_id, "non_existent_place");
    assert_eq!(
        loaded.processes[0].steps[0].assigned_actor.as_deref(),
        Some("non_existent_actor")
    );
    assert_eq!(
        loaded.history[0].actor_id.as_deref(),
        Some("non_existent_actor")
    );
}

#[test]
fn test_relationship_cycles_and_self_reference() {
    let mut spec = WorldSpec::new();

    // Place A contains itself
    spec.relationships.push(Relationship::new(
        "rel_1",
        RelationshipType::Contains,
        "place_a",
        "place_a",
    ));

    // Place A adjacent to Place B, Place B adjacent to Place A (normal, symmetric)
    spec.relationships.push(Relationship::new(
        "rel_2",
        RelationshipType::AdjacentTo,
        "place_a",
        "place_b",
    ));
    spec.relationships.push(Relationship::new(
        "rel_3",
        RelationshipType::AdjacentTo,
        "place_b",
        "place_a",
    ));

    // Place A contains Place B, Place B contains Place A (impossible containment cycle)
    spec.relationships.push(Relationship::new(
        "rel_4",
        RelationshipType::Contains,
        "place_a",
        "place_b",
    ));
    spec.relationships.push(Relationship::new(
        "rel_5",
        RelationshipType::Contains,
        "place_b",
        "place_a",
    ));

    let json = serde_json::to_string(&spec).unwrap();
    let loaded: WorldSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.relationships.len(), 5);
}

#[test]
fn test_process_step_numbering_disorder() {
    let mut spec = WorldSpec::new();
    let mut process = Process::new("proc_1", "Disordered Process");

    // Steps have duplicate and out of order step numbers
    process.steps.push(ProcessStep::new(2, "Step 2", 10.0));
    process.steps.push(ProcessStep::new(1, "Step 1", 5.0));
    process
        .steps
        .push(ProcessStep::new(2, "Duplicate Step 2", 15.0));
    process
        .steps
        .push(ProcessStep::new(100, "Gap Step 100", 20.0));

    spec.processes.push(process);

    let json = serde_json::to_string(&spec).unwrap();
    let loaded: WorldSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.processes[0].steps.len(), 4);
    assert_eq!(loaded.processes[0].steps[0].step_number, 2);
}
