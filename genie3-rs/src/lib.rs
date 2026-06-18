//! `genie3-rs` — Core crate for Genie 3 World Model state representation and actions.
//!
//! This crate provides the data structures to model places, actors, objects,
//! latent actions (translation and rotation), and environment parameters.

pub mod types;
pub mod world;
pub mod simulation;

// Re-exports for convenience
pub use types::{Bounds3D, Rotation3D, Transform, Vector3};
pub use world::{Actor, Environment, LatentAction, Object, Place, Weather, WorldState};
pub use simulation::{SimulationConfig, SimulationCommand, SimulationEngine, get_actor_bounds, get_object_bounds};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_operations() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);

        let v3 = v1.add(&v2);
        assert_eq!(v3, Vector3::new(5.0, 7.0, 9.0));

        let v4 = v2.sub(&v1);
        assert_eq!(v4, Vector3::new(3.0, 3.0, 3.0));

        let v5 = v1.scale(2.0);
        assert_eq!(v5, Vector3::new(2.0, 4.0, 6.0));

        let dist = v1.distance(&Vector3::new(1.0, 2.0, 3.0));
        assert!(dist < f32::EPSILON);
    }

    #[test]
    fn test_angle_normalization() {
        assert!((Rotation3D::normalize_angle(190.0) - (-170.0)).abs() < f32::EPSILON);
        assert!((Rotation3D::normalize_angle(-190.0) - 170.0).abs() < f32::EPSILON);
        assert!((Rotation3D::normalize_angle(360.0) - 0.0).abs() < f32::EPSILON);
        assert!((Rotation3D::normalize_angle(725.0) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bounds_checks() {
        let center = Vector3::new(0.0, 0.0, 0.0);
        let half_extents = Vector3::new(5.0, 5.0, 5.0);
        let bounds = Bounds3D::new(center, half_extents);

        assert!(bounds.contains_point(&Vector3::new(2.0, -2.0, 4.0)));
        assert!(!bounds.contains_point(&Vector3::new(6.0, 0.0, 0.0)));

        let other_overlapping =
            Bounds3D::new(Vector3::new(4.0, 4.0, 4.0), Vector3::new(2.0, 2.0, 2.0));
        let other_separate =
            Bounds3D::new(Vector3::new(20.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 2.0));

        assert!(bounds.intersects(&other_overlapping));
        assert!(!bounds.intersects(&other_separate));
    }

    #[test]
    fn test_world_state_evolution() {
        let mut state = WorldState::new();

        // Setup environment
        state.environment = Environment::new(Weather::Sunny, 10.0);

        // Add places
        let room_bounds =
            Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
        let room = Place::new("room_1", "Living Room", room_bounds);
        state.places.push(room);

        // Add actor
        let mut actor = Actor::new(
            "actor_1",
            "Robo-Assistant",
            "Robot",
            Vector3::new(0.0, 0.0, 0.0),
        );
        actor.place_id = Some("room_1".to_string());
        state.actors.push(actor);

        // Verify initial state
        assert_eq!(state.step_index, 0);
        assert!((state.environment.time_of_day - 10.0).abs() < f32::EPSILON);

        // Step environment
        state.step(1.5);
        assert_eq!(state.step_index, 1);
        assert!((state.environment.time_of_day - 11.5).abs() < f32::EPSILON);

        // Apply latent action (move actor out of room_1 boundaries)
        let action = LatentAction::new(
            "actor_1",
            Vector3::new(15.0, 0.0, 0.0),
            Rotation3D::new(0.0, 90.0, 0.0),
        );
        let res = state.apply_latent_action(&action);
        assert!(res.is_ok());

        // Verify updated actor position
        let updated_actor = state.get_actor("actor_1").unwrap();
        assert_eq!(updated_actor.position, Vector3::new(15.0, 0.0, 0.0));
        assert!((updated_actor.rotation.yaw - 90.0).abs() < f32::EPSILON);

        // Place should have recalculated to None because actor moved outside bounds
        assert_eq!(updated_actor.place_id, None);
    }

    #[test]
    fn test_coherence_validation() {
        let mut state = WorldState::new();
        assert!(state.validate_coherence().is_ok());

        // Introduce an actor referencing a non-existent place
        let mut actor = Actor::new("actor_1", "Ghost", "Spectre", Vector3::default());
        actor.place_id = Some("non_existent_place".to_string());
        state.actors.push(actor);

        assert!(state.validate_coherence().is_err());
    }

    #[test]
    fn test_simulation_spatial_temporal_consistency() {
        use std::collections::HashMap;
        let mut state = WorldState::new();

        // 1. Create a Place with bounds
        let room_bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
        let mut room = Place::new("room_1", "Control Room", room_bounds);
        room.properties.insert("hard_containment".to_string(), serde_json::Value::Bool(true));
        state.places.push(room);

        // 2. Spawn two actors
        let mut actor1 = Actor::new("actor_1", "Bot 1", "Drone", Vector3::new(0.0, 0.0, 0.0));
        actor1.place_id = Some("room_1".to_string());
        // Set dimensions (extents) for collision
        let mut actor1_props = HashMap::new();
        actor1_props.insert("half_extents".to_string(), serde_json::json!({"x": 1.0, "y": 1.0, "z": 1.0}));
        actor1_props.insert("max_speed".to_string(), serde_json::json!(5.0));
        actor1.properties = actor1_props;
        state.actors.push(actor1);

        let mut actor2 = Actor::new("actor_2", "Bot 2", "Drone", Vector3::new(5.0, 0.0, 0.0));
        actor2.place_id = Some("room_1".to_string());
        let mut actor2_props = HashMap::new();
        actor2_props.insert("half_extents".to_string(), serde_json::json!({"x": 1.0, "y": 1.0, "z": 1.0}));
        actor2.properties = actor2_props;
        state.actors.push(actor2);

        let engine = SimulationEngine::default();

        // Check validation: Movement of actor1 to (4.0, 0.0, 0.0) should overlap with actor2 at 5.0 (bounds overlap because 4+1 >= 5-1)
        let res = engine.validate_movement(&state, "actor_1", Vector3::new(4.0, 0.0, 0.0), Rotation3D::default());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("overlap with another Actor"));

        // Check validation: Movement of actor1 to (2.0, 0.0, 0.0) is safe (distance 2 <= speed limit 5)
        let res = engine.validate_movement(&state, "actor_1", Vector3::new(2.0, 0.0, 0.0), Rotation3D::default());
        assert!(res.is_ok());

        // Check validation: Teleportation prevention (actor1 trying to move by 6.0 units, limit is 5.0)
        let res = engine.validate_movement(&state, "actor_1", Vector3::new(6.0, 0.0, 0.0), Rotation3D::default());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("exceeds speed limit"));

        // Check validation: Bounded containment violation (moving actor1 outside room_1 bounds [0-10, 0-10, 0-10])
        // Since room_1 has hard_containment: true, actor cannot move to (12.0, 0.0, 0.0)
        // Adjust actor1's max_speed first to allow the distance
        state.get_actor_mut("actor_1").unwrap().properties.insert("max_speed".to_string(), serde_json::json!(20.0));
        let res = engine.validate_movement(&state, "actor_1", Vector3::new(12.0, 0.0, 0.0), Rotation3D::default());
        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("violate hard containment") || err_msg.contains("extend beyond the hard containment"));

        // 3. Test spawning validation
        // Attempting to spawn overlapping actor
        let mut spawn_props = HashMap::new();
        spawn_props.insert("half_extents".to_string(), serde_json::json!({"x": 1.0, "y": 1.0, "z": 1.0}));
        let res = engine.validate_spawn_actor(&state, "actor_3", Vector3::new(0.5, 0.0, 0.0), &spawn_props);
        assert!(res.is_err());

        // 4. Test command execution
        let cmd = SimulationCommand::MoveActor {
            actor_id: "actor_1".to_string(),
            movement: Vector3::new(2.0, 0.0, 0.0),
            rotation: Rotation3D::default(),
        };
        let next_state = engine.execute_command(&state, &cmd, 0.1).unwrap();
        assert_eq!(next_state.get_actor("actor_1").unwrap().position, Vector3::new(2.0, 0.0, 0.0));
        assert_eq!(next_state.step_index, 1);
    }
}
