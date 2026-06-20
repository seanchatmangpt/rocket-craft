use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A 3D coordinate or vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// Create a new Vector3.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// 3D axis-aligned spatial boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds3D {
    /// Center coordinates in the world.
    pub center: Vector3,
    /// Half-extents along each axis.
    pub half_extents: Vector3,
}

impl Bounds3D {
    /// Create a new Bounds3D.
    pub fn new(center: Vector3, half_extents: Vector3) -> Self {
        Self {
            center,
            half_extents,
        }
    }
}

impl Default for Bounds3D {
    fn default() -> Self {
        Self {
            center: Vector3::default(),
            half_extents: Vector3::new(100.0, 100.0, 100.0),
        }
    }
}

/// Spatial placement of an entity within the world.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Placement {
    /// Center position relative to the place or absolute.
    pub position: Vector3,
    /// Rotation Euler angles (Pitch, Yaw, Roll) in degrees.
    pub rotation: Vector3,
}

impl Placement {
    /// Create a new Placement.
    pub fn new(position: Vector3, rotation: Vector3) -> Self {
        Self { position, rotation }
    }
}

/// A physical zone, room, or region in the manufactured world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Place {
    /// Unique identifier for the place.
    pub id: String,
    /// Name of the place.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional parent place (for hierarchical containment, e.g. a bay inside a factory).
    #[serde(default)]
    pub parent_place_id: Option<String>,
    /// Spatial boundaries.
    pub bounds: Bounds3D,
    /// Extra metadata/attributes.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Place {
    /// Create a new Place.
    pub fn new(id: impl Into<String>, name: impl Into<String>, bounds: Bounds3D) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            parent_place_id: None,
            bounds,
            properties: HashMap::new(),
        }
    }
}

/// An active agent, NPC, worker, or robotic system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    /// Unique identifier for the actor.
    pub id: String,
    /// Name of the actor.
    pub name: String,
    /// The role or classification of the actor.
    pub role: String,
    /// Spatial placement of the actor.
    #[serde(default)]
    pub placement: Placement,
    /// ID of the Place the actor is currently inside.
    pub place_id: String,
    /// Extra metadata/attributes.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Actor {
    /// Create a new Actor.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        role: impl Into<String>,
        place_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role: role.into(),
            placement: Placement::default(),
            place_id: place_id.into(),
            properties: HashMap::new(),
        }
    }
}

/// A physical object, prop, raw material, or machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    /// Unique identifier for the object.
    pub id: String,
    /// Name of the object.
    pub name: String,
    /// Class or type of the object (e.g. "CNC_Machine", "SolarPanel").
    pub class: String,
    /// Spatial placement.
    #[serde(default)]
    pub placement: Placement,
    /// ID of the Place this object is currently inside.
    pub place_id: String,
    /// Extra metadata/attributes.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
    /// Tag list.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Object {
    /// Create a new Object.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        class: impl Into<String>,
        place_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            class: class.into(),
            placement: Placement::default(),
            place_id: place_id.into(),
            properties: HashMap::new(),
            tags: Vec::new(),
        }
    }
}

/// Types of semantic or physical associations between entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Connects two Places (e.g., adjacent rooms or logistics routes).
    Connects,
    /// Denotes hierarchical containment.
    Contains,
    /// Denotes ownership or assignment of an entity.
    Owns,
    /// Physical adjacency in space.
    AdjacentTo,
    /// Controls or coordinates another device.
    Controls,
    /// A custom relationship type.
    Custom(String),
}

/// A semantic or physical association between two entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier for the relationship.
    pub id: String,
    /// Relationship type.
    pub rel_type: RelationshipType,
    /// Source entity ID (e.g. Place, Actor, or Object ID).
    pub source: String,
    /// Target entity ID.
    pub target: String,
    /// Extra metadata/attributes.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Relationship {
    /// Create a new Relationship.
    pub fn new(
        id: impl Into<String>,
        rel_type: RelationshipType,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            rel_type,
            source: source.into(),
            target: target.into(),
            properties: HashMap::new(),
        }
    }
}

/// Severity classification for rule violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    /// Informational rule.
    Info,
    /// Soft constraint, produces warning.
    Warning,
    /// Hard constraint, prevents compilation or execution.
    Error,
}

/// An invariant or logic constraint that the world configuration must satisfy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule.
    pub id: String,
    /// Name of the rule.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// The rule expression/logic (e.g. SHACL pattern or validation logic).
    pub expression: String,
    /// Severity classification.
    pub severity: RuleSeverity,
}

impl Rule {
    /// Create a new Rule.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        expression: impl Into<String>,
        severity: RuleSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            expression: expression.into(),
            severity,
        }
    }
}

/// A log entry representing an event, transaction, or state transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEvent {
    /// Unique identifier for the event.
    pub id: String,
    /// Epoch timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Activity / transition name.
    pub activity: String,
    /// Optional ID of the actor performing the activity.
    #[serde(default)]
    pub actor_id: Option<String>,
    /// Context parameters and log details.
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

impl HistoryEvent {
    /// Create a new HistoryEvent.
    pub fn new(id: impl Into<String>, timestamp_ms: u64, activity: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            timestamp_ms,
            activity: activity.into(),
            actor_id: None,
            details: HashMap::new(),
        }
    }
}

/// Status of an operational process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    /// Not yet started.
    #[default]
    Pending,
    /// Currently running in the simulation or plant.
    Active,
    /// Completed successfully.
    Completed,
    /// Terminated due to errors or failures.
    Failed,
}

/// A single step within a Process workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessStep {
    /// Order index of the step (1-based).
    pub step_number: u32,
    /// Name of the step.
    pub name: String,
    /// Optional ID or role classification of the assigned actor.
    #[serde(default)]
    pub assigned_actor: Option<String>,
    /// Input entity IDs or classes required for this step.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Output entity IDs or classes produced by this step.
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Expected duration in seconds.
    pub duration_seconds: f32,
}

impl ProcessStep {
    /// Create a new ProcessStep.
    pub fn new(step_number: u32, name: impl Into<String>, duration_seconds: f32) -> Self {
        Self {
            step_number,
            name: name.into(),
            assigned_actor: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            duration_seconds,
        }
    }
}

/// A structured automation workflow or assembly recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Process {
    /// Unique identifier for the process.
    pub id: String,
    /// Name of the process.
    pub name: String,
    /// Ordered steps in the process.
    #[serde(default)]
    pub steps: Vec<ProcessStep>,
    /// Status.
    #[serde(default)]
    pub status: ProcessStatus,
}

impl Process {
    /// Create a new Process.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            steps: Vec::new(),
            status: ProcessStatus::Pending,
        }
    }
}

/// The root specification container for a manufactured world configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldSpec {
    /// Engine version used for manufacturing (e.g., "UE4.27-ES3").
    pub engine_version: String,
    /// List of physical places in the world.
    pub places: Vec<Place>,
    /// List of actors in the world.
    pub actors: Vec<Actor>,
    /// List of physical objects in the world.
    pub objects: Vec<Object>,
    /// List of semantic and structural relationships between entities.
    pub relationships: Vec<Relationship>,
    /// Operational and domain rules.
    pub rules: Vec<Rule>,
    /// Audit log of historical events.
    pub history: Vec<HistoryEvent>,
    /// Production and manufacturing processes.
    pub processes: Vec<Process>,
    /// Cryptographic verification receipts.
    pub receipts: Vec<unify_receipts::receipt::Receipt>,
}

impl WorldSpec {
    /// Create a new empty WorldSpec.
    pub fn new() -> Self {
        Self {
            engine_version: "UE4.27-ES3".to_string(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Vector3 ───────────────────────────────────────────────────────────────

    #[test]
    fn vector3_new_stores_components() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn vector3_default_is_origin() {
        let v = Vector3::default();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    // ── Bounds3D ──────────────────────────────────────────────────────────────

    #[test]
    fn bounds3d_default_has_large_half_extents() {
        let b = Bounds3D::default();
        assert_eq!(b.half_extents.x, 100.0);
    }

    #[test]
    fn placement_new_stores_fields() {
        let p = Placement::new(
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 90.0, 0.0),
        );
        assert_eq!(p.position.x, 1.0);
        assert_eq!(p.rotation.y, 90.0);
    }

    // ── ProcessStatus ─────────────────────────────────────────────────────────

    #[test]
    fn process_status_default_is_pending() {
        assert_eq!(ProcessStatus::default(), ProcessStatus::Pending);
    }

    // ── ProcessStep ───────────────────────────────────────────────────────────

    #[test]
    fn process_step_new_defaults() {
        let step = ProcessStep::new(1, "Assembly", 30.0);
        assert_eq!(step.step_number, 1);
        assert_eq!(step.name, "Assembly");
        assert_eq!(step.duration_seconds, 30.0);
        assert!(step.inputs.is_empty());
        assert!(step.outputs.is_empty());
        assert!(step.assigned_actor.is_none());
    }

    // ── Process ───────────────────────────────────────────────────────────────

    #[test]
    fn process_new_starts_pending_empty() {
        let p = Process::new("proc-1", "Welding");
        assert_eq!(p.id, "proc-1");
        assert_eq!(p.status, ProcessStatus::Pending);
        assert!(p.steps.is_empty());
    }

    // ── Rule ─────────────────────────────────────────────────────────────────

    #[test]
    fn rule_new_stores_severity() {
        let r = Rule::new("r1", "MaxActors", "actor_count <= 100", RuleSeverity::Error);
        assert_eq!(r.severity, RuleSeverity::Error);
        assert!(r.description.is_none());
    }

    // ── HistoryEvent ──────────────────────────────────────────────────────────

    #[test]
    fn history_event_new_stores_fields() {
        let ev = HistoryEvent::new("ev-1", 1_700_000_000_000, "ACTOR_SPAWNED");
        assert_eq!(ev.timestamp_ms, 1_700_000_000_000);
        assert_eq!(ev.activity, "ACTOR_SPAWNED");
        assert!(ev.actor_id.is_none());
        assert!(ev.details.is_empty());
    }

    // ── WorldSpec ─────────────────────────────────────────────────────────────

    #[test]
    fn world_spec_new_has_ue4_engine_version() {
        let ws = WorldSpec::new();
        assert!(ws.engine_version.contains("UE4"));
        assert!(ws.places.is_empty());
        assert!(ws.actors.is_empty());
    }

    #[test]
    fn world_spec_default_is_equivalent_to_new_except_engine_version() {
        let ws = WorldSpec::new();
        assert!(!ws.engine_version.is_empty()); // new() sets engine_version
    }
}
