use crate::types::{Bounds3D, Rotation3D, Transform, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A physical location or zone with bounding limits in the Genie 3 world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Place {
    /// Unique identifier for the place.
    pub id: String,
    /// Name of the place.
    pub name: String,
    /// Spatial boundaries of this location.
    pub bounds: Bounds3D,
    /// Hierarchical reference to a parent place, if nested.
    pub parent_place_id: Option<String>,
    /// Metadata or additional attributes of the place.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Place {
    /// Create a new Place.
    pub fn new(id: impl Into<String>, name: impl Into<String>, bounds: Bounds3D) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            bounds,
            parent_place_id: None,
            properties: HashMap::new(),
        }
    }
}

/// An entity inside the world, such as a player, NPC, vehicle, or robot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    /// Unique identifier for the actor.
    pub id: String,
    /// Name of the actor.
    pub name: String,
    /// Classification or type of actor (e.g. "Player", "NPC", "Drone").
    pub actor_type: String,
    /// 3D position in the world.
    pub position: Vector3,
    /// Rotation orientation.
    pub rotation: Rotation3D,
    /// ID of the Place the actor is currently situated in.
    pub place_id: Option<String>,
    /// Metadata or attributes for the actor.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Actor {
    /// Create a new Actor.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        actor_type: impl Into<String>,
        position: Vector3,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            actor_type: actor_type.into(),
            position,
            rotation: Rotation3D::default(),
            place_id: None,
            properties: HashMap::new(),
        }
    }
}

/// A passive physical object, machine, prop, or item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    /// Unique identifier for the object.
    pub id: String,
    /// Name of the object.
    pub name: String,
    /// Class or archetype of the object.
    pub class: String,
    /// Complete transform (position, rotation, scale).
    pub transform: Transform,
    /// ID of the Place this object is currently located in.
    pub place_id: Option<String>,
    /// Metadata or custom attributes.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Object {
    /// Create a new Object.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        class: impl Into<String>,
        transform: Transform,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            class: class.into(),
            transform,
            place_id: None,
            properties: HashMap::new(),
        }
    }
}

/// Types of environmental weather in Genie 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weather {
    Sunny,
    Cloudy,
    Rainy,
    Stormy,
    Snowy,
    Foggy,
    Custom(String),
}

/// Atmospheric and temporal rules of the environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Environment {
    /// Weather state of the environment.
    pub weather: Weather,
    /// Time of day in hours [0.0, 24.0).
    pub time_of_day: f32,
    /// Metadata or environment configuration rules.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Environment {
    /// Create a new Environment.
    pub fn new(weather: Weather, time_of_day: f32) -> Self {
        Self {
            weather,
            time_of_day,
            properties: HashMap::new(),
        }
    }

    /// Advance time in the environment, wrapping around 24.0 hours.
    pub fn step_time(&mut self, time_delta_hours: f32) {
        self.time_of_day = (self.time_of_day + time_delta_hours) % 24.0;
        if self.time_of_day < 0.0 {
            self.time_of_day += 24.0;
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            weather: Weather::Sunny,
            time_of_day: 12.0, // Midday by default
            properties: HashMap::new(),
        }
    }
}

/// A latent control input or action distribution mapping state transitions in Genie 3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatentAction {
    /// ID of the actor executing the action.
    pub actor_id: String,
    /// Positional movement delta or velocity vector.
    pub movement: Vector3,
    /// Rotation delta or angular velocity.
    pub rotation: Rotation3D,
    /// Abstract latent code representation from the machine learning model.
    #[serde(default)]
    pub latent_vector: Option<Vec<f32>>,
}

impl LatentAction {
    /// Create a new LatentAction.
    pub fn new(actor_id: impl Into<String>, movement: Vector3, rotation: Rotation3D) -> Self {
        Self {
            actor_id: actor_id.into(),
            movement,
            rotation,
            latent_vector: None,
        }
    }
}

/// The complete state container of Genie 3's world model.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    /// List of locations/places.
    pub places: Vec<Place>,
    /// List of active actors.
    pub actors: Vec<Actor>,
    /// List of physical objects.
    pub objects: Vec<Object>,
    /// Atmospheric/weather/time environment.
    pub environment: Environment,
    /// Current step index or frame tick of the simulation.
    pub step_index: u64,
    /// Custom state properties.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl WorldState {
    /// Create a new empty WorldState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve a reference to an actor by ID.
    pub fn get_actor(&self, actor_id: &str) -> Option<&Actor> {
        self.actors.iter().find(|a| a.id == actor_id)
    }

    /// Retrieve a mutable reference to an actor by ID.
    pub fn get_actor_mut(&mut self, actor_id: &str) -> Option<&mut Actor> {
        self.actors.iter_mut().find(|a| a.id == actor_id)
    }

    /// Retrieve a reference to a place by ID.
    pub fn get_place(&self, place_id: &str) -> Option<&Place> {
        self.places.iter().find(|p| p.id == place_id)
    }

    /// Apply a latent action to an actor, updating their position and orientation.
    pub fn apply_latent_action(&mut self, action: &LatentAction) -> Result<(), String> {
        let actor = self.get_actor_mut(&action.actor_id).ok_or_else(|| {
            format!(
                "Actor with ID '{}' not found in WorldState",
                action.actor_id
            )
        })?;

        // Update actor state
        actor.position = actor.position.add(&action.movement);
        actor.rotation = actor.rotation.add(&action.rotation);

        // Optional: Update actor's current place based on boundary check
        self.recalculate_actor_place(&action.actor_id);

        Ok(())
    }

    /// Recalculate which Place an actor is inside based on their 3D coordinates.
    pub fn recalculate_actor_place(&mut self, actor_id: &str) {
        let actor_pos = if let Some(actor) = self.get_actor(actor_id) {
            actor.position
        } else {
            return;
        };

        // Find the place containing the actor's position.
        // Nested / child places take priority over parent places.
        let mut best_place_id: Option<String> = None;
        let mut best_place_is_nested = false;

        for place in &self.places {
            if place.bounds.contains_point(&actor_pos) {
                let is_nested = place.parent_place_id.is_some();
                if best_place_id.is_none() || (is_nested && !best_place_is_nested) {
                    best_place_id = Some(place.id.clone());
                    best_place_is_nested = is_nested;
                }
            }
        }

        if let Some(actor) = self.get_actor_mut(actor_id) {
            actor.place_id = best_place_id;
        }
    }

    /// Evolve the world state by advancing time and incrementing the step counter.
    pub fn step(&mut self, time_delta_hours: f32) {
        self.environment.step_time(time_delta_hours);
        self.step_index += 1;
    }

    /// Validate the referential and spatial coherence of the world state.
    pub fn validate_coherence(&self) -> Result<(), String> {
        // 1. Validate Actors refer to valid Places
        for actor in &self.actors {
            if let Some(ref pid) = actor.place_id {
                if self.get_place(pid).is_none() {
                    return Err(format!(
                        "Actor '{}' references non-existent Place ID '{}'",
                        actor.id, pid
                    ));
                }
            }
        }

        // 2. Validate Objects refer to valid Places
        for object in &self.objects {
            if let Some(ref pid) = object.place_id {
                if self.get_place(pid).is_none() {
                    return Err(format!(
                        "Object '{}' references non-existent Place ID '{}'",
                        object.id, pid
                    ));
                }
            }
        }

        // 3. Validate parent-child references in Places
        for place in &self.places {
            if let Some(ref parent_id) = place.parent_place_id {
                if self.get_place(parent_id).is_none() {
                    return Err(format!(
                        "Place '{}' references non-existent parent Place ID '{}'",
                        place.id, parent_id
                    ));
                }
            }
        }

        Ok(())
    }
}
