use chrono::Utc;
use nexus_mud::{MudEngine, MudError, MudEvent, Zone, AABB};

#[test]
fn test_aabb_intersection_boundaries() {
    // Test overlapping boxes
    let a = AABB {
        min_x: 0.0,
        max_x: 2.0,
        min_y: 0.0,
        max_y: 2.0,
        min_z: 0.0,
        max_z: 2.0,
    };
    let b = AABB {
        min_x: 1.0,
        max_x: 3.0,
        min_y: 1.0,
        max_y: 3.0,
        min_z: 1.0,
        max_z: 3.0,
    };
    assert!(a.intersects(&b), "Should overlap on all axes");
    assert!(b.intersects(&a), "Symmetry check");

    // Test edge-touching (non-overlapping) - min_x == max_x boundary
    let c = AABB {
        min_x: 2.0,
        max_x: 4.0,
        min_y: 0.0,
        max_y: 2.0,
        min_z: 0.0,
        max_z: 2.0,
    };
    assert!(
        !a.intersects(&c),
        "Touching on face should not be considered an intersection (strict less-than comparison)"
    );

    // Test completely separate boxes
    let d = AABB {
        min_x: 10.0,
        max_x: 12.0,
        min_y: 10.0,
        max_y: 12.0,
        min_z: 10.0,
        max_z: 12.0,
    };
    assert!(
        !a.intersects(&d),
        "Completely separated boxes must not intersect"
    );
}

#[test]
fn test_com_deviation_threshold() {
    // Create an engine, verify up to proving ground
    let mut engine = MudEngine::new();
    engine.gates.mission = true;
    engine.gates.materials = true;
    engine.gates.primitive = true;
    engine.gates.runner_wall = true;
    engine.gates.assembly = true;
    engine.gates.fit = true;
    engine.gates.collision = true;
    engine.current_zone = Zone::ProvingGround;

    // Set arm masses to be asymmetric but within limits (deviation <= 10)
    if let Some(left) = engine.parts.get_mut("left_arm") {
        left.mass = 20.0;
    }
    if let Some(right) = engine.parts.get_mut("right_arm") {
        right.mass = 29.9;
    }

    // Should verify motion successfully
    let res = engine.execute_command("verify motion");
    assert!(res.is_ok(), "COM deviation of 9.9 should pass");

    // Exceed the limit (deviation > 10)
    if let Some(right) = engine.parts.get_mut("right_arm") {
        right.mass = 31.0; // deviation = 11.0
    }
    engine.gates.motion = false; // Reset gate

    let res_fail = engine.execute_command("verify motion");
    assert!(
        res_fail.is_err(),
        "COM deviation of 11.0 should be rejected"
    );
    if let Err(MudError::GateBlocked(_)) = res_fail {
        // Structural check: diagnostics map must record a motion_fail entry referencing COM deviation
        let motion_fail = engine
            .diagnostics
            .get("motion_fail")
            .expect("Expected 'motion_fail' diagnostic to be set after COM deviation rejection");
        assert!(
            motion_fail.starts_with("Center of Mass deviation"),
            "motion_fail diagnostic must start with 'Center of Mass deviation', got: {}",
            motion_fail
        );
    } else {
        panic!("Expected GateBlocked error");
    }
}

#[test]
fn test_unbounded_joint_limits() {
    let mut engine = MudEngine::new();
    engine.gates.mission = true;
    engine.gates.materials = true;
    engine.gates.primitive = true;
    engine.gates.runner_wall = true;
    engine.gates.assembly = true;
    engine.gates.fit = true;
    engine.gates.collision = true;
    engine.current_zone = Zone::ProvingGround;

    // Remove limits from a joint
    if let Some(joint) = engine.joints.get_mut("ShoulderL") {
        joint.rotation_limits = None;
    }

    let res = engine.execute_command("verify motion");
    assert!(
        res.is_err(),
        "Unbounded joint rotation limits must fail verification"
    );
}

#[test]
fn test_event_log_monotonic_timestamps() {
    let mut engine = MudEngine::new();

    // Fire many events in rapid succession
    for i in 0..100 {
        engine.emit_event("test.event", "head", &format!("Event number {}", i));
    }

    // Verify strict monotonicity
    let log = &engine.event_log;
    for i in 1..log.len() {
        assert!(
            log[i].timestamp > log[i - 1].timestamp,
            "Event log timestamp is not strictly monotonic at index {}: {:?} vs {:?}",
            i,
            log[i].timestamp,
            log[i - 1].timestamp
        );
    }
}

#[test]
fn test_referential_integrity_failures() {
    let mut engine = MudEngine::new();

    // With nominal events, referential integrity check should pass
    assert!(engine.verify_referential_integrity().is_ok());

    // Inject an invalid object_id event
    engine.emit_event(
        "illegal.action",
        "unregistered_object_xyz",
        "This object is not in the whitelist",
    );

    let result = engine.verify_referential_integrity();
    assert!(
        result.is_err(),
        "Referential integrity check should fail with an invalid object_id"
    );
    // Structural check: find the offending event directly in the event log rather than string-matching the error
    let offending_event = engine
        .event_log
        .iter()
        .find(|e| e.object_id == "unregistered_object_xyz")
        .expect(
            "Event log must contain the injected event with object_id 'unregistered_object_xyz'",
        );
    assert_eq!(
        offending_event.event_type, "illegal.action",
        "Injected event must have event_type 'illegal.action', got: {}",
        offending_event.event_type
    );
}

#[test]
fn test_referential_integrity_gaps() {
    let mut engine = MudEngine::new();

    // Gap 1: Duplicate event IDs are not validated.
    engine.event_log.push(MudEvent {
        event_id: "evt_1".to_string(),
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        object_id: "head".to_string(),
        details: "First event".to_string(),
    });
    engine.event_log.push(MudEvent {
        event_id: "evt_1".to_string(), // duplicate ID
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        object_id: "head".to_string(),
        details: "Second event with duplicate ID".to_string(),
    });

    assert!(
        engine.verify_referential_integrity().is_ok(),
        "Referential integrity check passes even with duplicate event_id, demonstrating a gap in primary key validation."
    );

    // Gap 2: Malformed event ID (e.g. empty or non-standard format) is not checked.
    engine.event_log.push(MudEvent {
        event_id: "".to_string(), // empty ID
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        object_id: "head".to_string(),
        details: "Event with empty ID".to_string(),
    });

    assert!(
        engine.verify_referential_integrity().is_ok(),
        "Referential integrity check passes even with empty event_id, demonstrating a gap in ID format validation."
    );
}
