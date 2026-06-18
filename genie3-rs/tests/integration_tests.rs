use genie3_rs::{
    simulation::{SimulationCommand, SimulationEngine},
    types::{Bounds3D, Rotation3D, Transform, Vector3},
    world::{Actor, Environment, Object, Place, Weather, WorldState},
};
use std::collections::HashMap;

/// Helper to create a standard property map with half_extents and max_speed.
fn create_actor_properties(
    hx: f32,
    hy: f32,
    hz: f32,
    max_speed: f32,
) -> HashMap<String, serde_json::Value> {
    let mut props = HashMap::new();
    props.insert(
        "half_extents".to_string(),
        serde_json::json!({"x": hx, "y": hy, "z": hz}),
    );
    props.insert("max_speed".to_string(), serde_json::json!(max_speed));
    props
}

/// Helper to create a standard property map for objects.
fn create_object_properties(hx: f32, hy: f32, hz: f32) -> HashMap<String, serde_json::Value> {
    let mut props = HashMap::new();
    props.insert(
        "half_extents".to_string(),
        serde_json::json!({"x": hx, "y": hy, "z": hz}),
    );
    props
}

#[test]
fn test_scenario_movement_and_promptable_event() {
    // 1. Create an initial state
    let mut state = WorldState::new();
    state.environment = Environment::new(Weather::Sunny, 12.0); // Midday, Sunny

    // Add a place: "room_1" (Control Room) centered at (0, 0, 0) with extents (50, 50, 50)
    let room_bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(50.0, 50.0, 50.0));
    let mut room1 = Place::new("room_1", "Control Room", room_bounds);
    room1.properties.insert(
        "hard_containment".to_string(),
        serde_json::Value::Bool(true),
    );
    state.places.push(room1);

    // Spawn an actor: "bot_1" (Welder Bot) at (0, 0, 0)
    let mut bot = Actor::new("bot_1", "Welder Bot", "Robot", Vector3::new(0.0, 0.0, 0.0));
    bot.place_id = Some("room_1".to_string());
    bot.properties = create_actor_properties(1.0, 1.0, 2.0, 15.0); // extents 1x1x2, max speed 15
    state.actors.push(bot);

    // Spawn an object: "cnc_1" (CNC Machine) at (10, 10, 0)
    let cnc_transform = Transform::new(
        Vector3::new(10.0, 10.0, 0.0),
        Rotation3D::default(),
        Vector3::new(1.0, 1.0, 1.0),
    );
    let mut cnc = Object::new("cnc_1", "CNC Alpha", "Machine", cnc_transform);
    cnc.place_id = Some("room_1".to_string());
    cnc.properties = create_object_properties(2.0, 2.0, 2.0); // extents 2x2x2
    state.objects.push(cnc);

    // Verify initial setup and consistency
    assert_eq!(state.step_index, 0);
    assert!(state.validate_coherence().is_ok());

    let engine = SimulationEngine::default();

    // 2. Apply movement actions
    // Move bot_1 by (+5, +5, 0). Safe distance (sqrt(50) ~= 7.07 < 15.0 max speed), no collision.
    let move_cmd = SimulationCommand::MoveActor {
        actor_id: "bot_1".to_string(),
        movement: Vector3::new(5.0, 5.0, 0.0),
        rotation: Rotation3D::new(0.0, 45.0, 0.0),
    };

    let state_after_move = engine
        .execute_command(&state, &move_cmd, 0.1)
        .expect("Failed to move actor");

    // Assert movement state transition
    let updated_bot = state_after_move.get_actor("bot_1").unwrap();
    assert_eq!(updated_bot.position, Vector3::new(5.0, 5.0, 0.0));
    assert_eq!(updated_bot.rotation.yaw, 45.0);
    assert_eq!(state_after_move.step_index, 1);
    assert!((state_after_move.environment.time_of_day - 12.1).abs() < f32::EPSILON);
    assert!(state_after_move.validate_coherence().is_ok());

    // 3. Apply a promptable event (weather change + spawning a new obstacle object)
    // Apply Promptable Event A: Change weather to stormy and time of day to evening (20.0)
    let weather_cmd = SimulationCommand::ChangeWeather {
        weather: Weather::Stormy,
    };
    let state_weather = engine
        .execute_command(&state_after_move, &weather_cmd, 0.1)
        .expect("Failed to change weather");
    let time_cmd = SimulationCommand::ChangeTime { time_of_day: 20.0 };
    // Pass 0.0 for time delta to ensure time_of_day is exactly 20.0
    let state_time = engine
        .execute_command(&state_weather, &time_cmd, 0.0)
        .expect("Failed to change time");

    assert_eq!(state_time.environment.weather, Weather::Stormy);
    assert!((state_time.environment.time_of_day - 20.0).abs() < f32::EPSILON);

    // Apply Promptable Event B: Spawn a barrier object "barrier_1" at (5.0, 10.0, 0.0)
    let barrier_transform = Transform::new(
        Vector3::new(5.0, 10.0, 0.0),
        Rotation3D::default(),
        Vector3::new(1.0, 1.0, 1.0),
    );
    let spawn_cmd = SimulationCommand::SpawnObject {
        id: "barrier_1".to_string(),
        name: "Security Barrier".to_string(),
        class: "Barrier".to_string(),
        transform: barrier_transform,
        properties: create_object_properties(1.5, 0.5, 1.0),
    };
    let state_spawned = engine
        .execute_command(&state_time, &spawn_cmd, 0.1)
        .expect("Failed to spawn object");

    // Assert spawned object properties and spatial containment
    assert_eq!(state_spawned.objects.len(), 2);
    let spawned_obj = state_spawned
        .objects
        .iter()
        .find(|o| o.id == "barrier_1")
        .unwrap();
    assert_eq!(spawned_obj.transform.position, Vector3::new(5.0, 10.0, 0.0));
    assert_eq!(spawned_obj.place_id, Some("room_1".to_string()));
    assert!(state_spawned.validate_coherence().is_ok());

    // 4. Assert that further movement into the spawned obstacle is rejected
    // Moving bot_1 to (5.0, 9.0) which ranges [8.0, 10.0] would cause an overlap with the barrier at Y=10.0 (extents 0.5, ranges [9.5, 10.5])
    let col_move_cmd_overlap = SimulationCommand::MoveActor {
        actor_id: "bot_1".to_string(),
        movement: Vector3::new(0.0, 4.0, 0.0), // proposed position (5.0, 9.0)
        rotation: Rotation3D::default(),
    };

    let col_res = engine.execute_command(&state_spawned, &col_move_cmd_overlap, 0.1);
    assert!(col_res.is_err());
    assert!(col_res
        .unwrap_err()
        .contains("overlap with Object 'barrier_1'"));
}

#[test]
fn test_speed_limit_and_hard_containment() {
    let mut state = WorldState::new();
    let room_bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(20.0, 20.0, 20.0));
    let mut room = Place::new("room_1", "Testing Room", room_bounds);
    room.properties.insert(
        "hard_containment".to_string(),
        serde_json::Value::Bool(true),
    );
    state.places.push(room);

    let mut bot = Actor::new("bot_1", "Speedy Bot", "Robot", Vector3::new(0.0, 0.0, 0.0));
    bot.place_id = Some("room_1".to_string());
    bot.properties = create_actor_properties(0.5, 0.5, 0.5, 5.0); // max speed 5
    state.actors.push(bot);

    let engine = SimulationEngine::default();

    // Test movement within speed limit
    let safe_cmd = SimulationCommand::MoveActor {
        actor_id: "bot_1".to_string(),
        movement: Vector3::new(3.0, 0.0, 0.0),
        rotation: Rotation3D::default(),
    };
    assert!(engine.execute_command(&state, &safe_cmd, 0.1).is_ok());

    // Test movement exceeding speed limit
    let unsafe_cmd = SimulationCommand::MoveActor {
        actor_id: "bot_1".to_string(),
        movement: Vector3::new(6.0, 0.0, 0.0),
        rotation: Rotation3D::default(),
    };
    let speed_res = engine.execute_command(&state, &unsafe_cmd, 0.1);
    assert!(speed_res.is_err());
    assert!(speed_res.unwrap_err().contains("exceeds speed limit"));

    // Test movement violating hard containment of place bounds (extents 20, center 0, bot position 0, move by 21)
    // First increase bot max speed so speed limit doesn't fail first
    state
        .get_actor_mut("bot_1")
        .unwrap()
        .properties
        .insert("max_speed".to_string(), serde_json::json!(50.0));
    let containment_cmd = SimulationCommand::MoveActor {
        actor_id: "bot_1".to_string(),
        movement: Vector3::new(21.0, 0.0, 0.0),
        rotation: Rotation3D::default(),
    };
    let containment_res = engine.execute_command(&state, &containment_cmd, 0.1);
    assert!(containment_res.is_err());
    assert!(
        containment_res
            .as_ref()
            .unwrap_err()
            .contains("violate hard containment")
            || containment_res
                .as_ref()
                .unwrap_err()
                .contains("extend beyond the hard containment")
    );
}

#[test]
fn test_coherence_referential_integrity() {
    let mut state = WorldState::new();

    // 1. Valid Place
    let room_bounds = Bounds3D::new(Vector3::default(), Vector3::new(10.0, 10.0, 10.0));
    state
        .places
        .push(Place::new("room_1", "Test Room", room_bounds));

    // 2. Actor referencing non-existent place
    let mut bot = Actor::new("bot_1", "Orphan Bot", "Robot", Vector3::default());
    bot.place_id = Some("non_existent_room".to_string());
    state.actors.push(bot);

    assert!(state.validate_coherence().is_err());

    // Fix actor reference
    state.get_actor_mut("bot_1").unwrap().place_id = Some("room_1".to_string());
    assert!(state.validate_coherence().is_ok());

    // 3. Object referencing non-existent place
    let transform = Transform::default();
    let mut obj = Object::new("obj_1", "Props", "Prop", transform);
    obj.place_id = Some("invalid_room".to_string());
    state.objects.push(obj);

    assert!(state.validate_coherence().is_err());
}
