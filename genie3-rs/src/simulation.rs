use crate::types::{Bounds3D, Rotation3D, Transform, Vector3};
use crate::world::{Actor, LatentAction, Object, Weather, WorldState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration parameters for the simulation engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Default maximum speed an actor can travel in a single tick.
    pub default_max_speed: f32,
    /// Whether to check actor-to-actor collisions.
    pub check_actor_collisions: bool,
    /// Whether to check actor-to-object collisions.
    pub check_object_collisions: bool,
    /// Whether to enforce strict containment within place bounds.
    pub enforce_strict_containment: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            default_max_speed: 10.0,
            check_actor_collisions: true,
            check_object_collisions: true,
            enforce_strict_containment: false,
        }
    }
}

/// Simulation commands that can dynamically alter the world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimulationCommand {
    MoveActor {
        actor_id: String,
        movement: Vector3,
        rotation: Rotation3D,
    },
    SpawnActor {
        id: String,
        name: String,
        actor_type: String,
        position: Vector3,
        rotation: Option<Rotation3D>,
        #[serde(default)]
        properties: HashMap<String, serde_json::Value>,
    },
    SpawnObject {
        id: String,
        name: String,
        class: String,
        transform: Transform,
        #[serde(default)]
        properties: HashMap<String, serde_json::Value>,
    },
    ChangeWeather {
        weather: Weather,
    },
    ChangeTime {
        time_of_day: f32,
    },
}

/// Helper to parse Vector3 out of generic property maps.
fn get_vector3_property(
    properties: &HashMap<String, serde_json::Value>,
    key: &str,
    default: Vector3,
) -> Vector3 {
    if let Some(val) = properties.get(key) {
        if let Ok(vec) = serde_json::from_value::<Vector3>(val.clone()) {
            return vec;
        }
        if let Some(obj) = val.as_object() {
            let x = obj
                .get("x")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
                .unwrap_or(default.x);
            let y = obj
                .get("y")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
                .unwrap_or(default.y);
            let z = obj
                .get("z")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
                .unwrap_or(default.z);
            return Vector3::new(x, y, z);
        }
    }
    default
}

/// Computes the bounding box of an Actor.
pub fn get_actor_bounds(actor: &Actor) -> Bounds3D {
    let half_extents = get_vector3_property(
        &actor.properties,
        "half_extents",
        Vector3::new(0.5, 0.5, 1.0),
    );
    Bounds3D::new(actor.position, half_extents)
}

/// Computes the bounding box of an Object.
pub fn get_object_bounds(object: &Object) -> Bounds3D {
    let base_extents = get_vector3_property(
        &object.properties,
        "half_extents",
        Vector3::new(1.0, 1.0, 1.0),
    );
    let half_extents = Vector3::new(
        base_extents.x * object.transform.scale.x,
        base_extents.y * object.transform.scale.y,
        base_extents.z * object.transform.scale.z,
    );
    Bounds3D::new(object.transform.position, half_extents)
}

/// The core physics and transition simulator for the Genie 3 World Model.
pub struct SimulationEngine {
    pub config: SimulationConfig,
}

impl SimulationEngine {
    /// Create a new SimulationEngine with custom configuration.
    pub fn new(config: SimulationConfig) -> Self {
        Self { config }
    }

    /// Create a new SimulationEngine with default configuration.
    pub fn default() -> Self {
        Self::new(SimulationConfig::default())
    }

    /// Validates the transition of an actor to a new position.
    pub fn validate_movement(
        &self,
        state: &WorldState,
        actor_id: &str,
        proposed_position: Vector3,
        _proposed_rotation: Rotation3D,
    ) -> Result<(), String> {
        let actor = state
            .get_actor(actor_id)
            .ok_or_else(|| format!("Actor '{}' not found", actor_id))?;

        // 1. Prevent Teleportation: Check max speed limits
        let distance = actor.position.distance(&proposed_position);
        let max_speed = actor
            .properties
            .get("max_speed")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(self.config.default_max_speed);

        if distance > max_speed {
            return Err(format!(
                "Movement exceeds speed limit (distance: {:.2}, max_speed: {:.2}). Teleportation prevented.",
                distance, max_speed
            ));
        }

        // 2. Spatial Consistency: Avoid Overlap
        let actor_half_extents = get_vector3_property(
            &actor.properties,
            "half_extents",
            Vector3::new(0.5, 0.5, 1.0),
        );
        let proposed_actor_bounds = Bounds3D::new(proposed_position, actor_half_extents);

        // Check collision with other actors
        if self.config.check_actor_collisions {
            for other_actor in &state.actors {
                if other_actor.id != actor_id {
                    let other_bounds = get_actor_bounds(other_actor);
                    if proposed_actor_bounds.intersects(&other_bounds) {
                        return Err(format!(
                            "Movement would cause overlap with another Actor '{}' at {:?}",
                            other_actor.id, other_actor.position
                        ));
                    }
                }
            }
        }

        // Check collision with objects
        if self.config.check_object_collisions {
            for object in &state.objects {
                let object_bounds = get_object_bounds(object);
                if proposed_actor_bounds.intersects(&object_bounds) {
                    return Err(format!(
                        "Movement would cause overlap with Object '{}' of class '{}' at {:?}",
                        object.id, object.class, object.transform.position
                    ));
                }
            }
        }

        // 3. Bounded Containment
        let current_place_id = actor.place_id.as_ref();
        if let Some(pid) = current_place_id {
            if let Some(place) = state.get_place(pid) {
                let is_hard_containment = place
                    .properties
                    .get("hard_containment")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(self.config.enforce_strict_containment);

                if is_hard_containment {
                    if !place.bounds.contains_point(&proposed_position) {
                        return Err(format!(
                            "Movement would violate hard containment of Place '{}' (bounds center: {:?}, extents: {:?})",
                            place.id, place.bounds.center, place.bounds.half_extents
                        ));
                    }

                    // Strict bounding box containment
                    let place_min = place.bounds.center.sub(&place.bounds.half_extents);
                    let place_max = place.bounds.center.add(&place.bounds.half_extents);
                    let actor_min = proposed_position.sub(&actor_half_extents);
                    let actor_max = proposed_position.add(&actor_half_extents);

                    if actor_min.x < place_min.x
                        || actor_max.x > place_max.x
                        || actor_min.y < place_min.y
                        || actor_max.y > place_max.y
                        || actor_min.z < place_min.z
                        || actor_max.z > place_max.z
                    {
                        return Err(format!(
                            "Actor bounds would extend beyond the hard containment boundaries of Place '{}'",
                            place.id
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates whether a new actor can be spawned.
    pub fn validate_spawn_actor(
        &self,
        state: &WorldState,
        id: &str,
        position: Vector3,
        properties: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if state.get_actor(id).is_some() {
            return Err(format!("Actor ID '{}' is already in use", id));
        }

        let half_extents =
            get_vector3_property(properties, "half_extents", Vector3::new(0.5, 0.5, 1.0));
        let proposed_bounds = Bounds3D::new(position, half_extents);

        for other_actor in &state.actors {
            let other_bounds = get_actor_bounds(other_actor);
            if proposed_bounds.intersects(&other_bounds) {
                return Err(format!(
                    "Cannot spawn Actor '{}' because it overlaps with existing Actor '{}'",
                    id, other_actor.id
                ));
            }
        }

        for object in &state.objects {
            let object_bounds = get_object_bounds(object);
            if proposed_bounds.intersects(&object_bounds) {
                return Err(format!(
                    "Cannot spawn Actor '{}' because it overlaps with Object '{}'",
                    id, object.id
                ));
            }
        }

        Ok(())
    }

    /// Validates whether a new object can be spawned.
    pub fn validate_spawn_object(
        &self,
        state: &WorldState,
        id: &str,
        transform: Transform,
        properties: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if state.objects.iter().any(|o| o.id == id) {
            return Err(format!("Object ID '{}' is already in use", id));
        }

        let base_extents =
            get_vector3_property(properties, "half_extents", Vector3::new(1.0, 1.0, 1.0));
        let half_extents = Vector3::new(
            base_extents.x * transform.scale.x,
            base_extents.y * transform.scale.y,
            base_extents.z * transform.scale.z,
        );
        let proposed_bounds = Bounds3D::new(transform.position, half_extents);

        for actor in &state.actors {
            let actor_bounds = get_actor_bounds(actor);
            if proposed_bounds.intersects(&actor_bounds) {
                return Err(format!(
                    "Cannot spawn Object '{}' because it overlaps with Actor '{}'",
                    id, actor.id
                ));
            }
        }

        for other_object in &state.objects {
            let other_bounds = get_object_bounds(other_object);
            if proposed_bounds.intersects(&other_bounds) {
                return Err(format!(
                    "Cannot spawn Object '{}' because it overlaps with existing Object '{}'",
                    id, other_object.id
                ));
            }
        }

        Ok(())
    }

    /// Evolve the state using a latent action.
    pub fn step(
        &self,
        state: &WorldState,
        action: &LatentAction,
        time_delta_hours: f32,
    ) -> Result<WorldState, String> {
        let mut next_state = state.clone();

        let actor = state.get_actor(&action.actor_id).ok_or_else(|| {
            format!(
                "Actor with ID '{}' not found in WorldState",
                action.actor_id
            )
        })?;

        let proposed_position = actor.position.add(&action.movement);
        let proposed_rotation = actor.rotation.add(&action.rotation);

        self.validate_movement(
            state,
            &action.actor_id,
            proposed_position,
            proposed_rotation,
        )?;

        next_state.apply_latent_action(action)?;
        next_state.step(time_delta_hours);

        Ok(next_state)
    }

    /// Execute a simulation command on the state, returning the new state.
    pub fn execute_command(
        &self,
        state: &WorldState,
        command: &SimulationCommand,
        time_delta_hours: f32,
    ) -> Result<WorldState, String> {
        let mut next_state = state.clone();

        match command {
            SimulationCommand::MoveActor {
                actor_id,
                movement,
                rotation,
            } => {
                let action = LatentAction::new(actor_id, *movement, *rotation);
                return self.step(state, &action, time_delta_hours);
            }
            SimulationCommand::SpawnActor {
                id,
                name,
                actor_type,
                position,
                rotation,
                properties,
            } => {
                self.validate_spawn_actor(state, id, *position, properties)?;
                let mut actor = Actor::new(id, name, actor_type, *position);
                actor.rotation = rotation.unwrap_or_default();
                actor.properties = properties.clone();
                next_state.actors.push(actor);
                next_state.recalculate_actor_place(id);
                next_state.step(time_delta_hours);
            }
            SimulationCommand::SpawnObject {
                id,
                name,
                class,
                transform,
                properties,
            } => {
                self.validate_spawn_object(state, id, *transform, properties)?;
                let mut object = Object::new(id, name, class, *transform);
                object.properties = properties.clone();

                let mut best_place_id: Option<String> = None;
                let mut best_place_is_nested = false;
                for place in &state.places {
                    if place.bounds.contains_point(&transform.position) {
                        let is_nested = place.parent_place_id.is_some();
                        if best_place_id.is_none() || (is_nested && !best_place_is_nested) {
                            best_place_id = Some(place.id.clone());
                            best_place_is_nested = is_nested;
                        }
                    }
                }
                object.place_id = best_place_id;

                next_state.objects.push(object);
                next_state.step(time_delta_hours);
            }
            SimulationCommand::ChangeWeather { weather } => {
                next_state.environment.weather = weather.clone();
                next_state.step(time_delta_hours);
            }
            SimulationCommand::ChangeTime { time_of_day } => {
                if *time_of_day < 0.0 || *time_of_day >= 24.0 {
                    return Err(format!(
                        "Invalid time of day: {:.2}. Must be in [0.0, 24.0)",
                        time_of_day
                    ));
                }
                next_state.environment.time_of_day = *time_of_day;
                next_state.step_index += 1;
            }
        }

        next_state.validate_coherence()?;

        Ok(next_state)
    }
}
