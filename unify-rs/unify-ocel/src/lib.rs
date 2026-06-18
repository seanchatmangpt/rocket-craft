//! OCEL 2.0 — Object-Centric Event Log implementation for process mining.
//!
//! This crate implements the [OCEL 2.0 standard](https://www.ocel-standard.org/)
//! which allows process events to relate to multiple objects simultaneously,
//! overcoming the "one case per event" limitation of traditional event logs.
//!
//! # Quick start
//!
//! ```rust
//! use unify_ocel::{OcelLogBuilder, OcelValue, OcelAttrType, OcelQuery};
//!
//! let mut builder = OcelLogBuilder::new();
//! builder.add_object_type("Order", vec![("amount", OcelAttrType::Float)]);
//! let order_id = builder.add_object("Order", vec![("amount", OcelValue::Float(99.0))]);
//! builder.add_event(
//!     "order:place",
//!     "2024-01-01T10:00:00Z",
//!     vec![],
//!     vec![(&order_id, "created")],
//! );
//! let log = builder.build();
//! let query = OcelQuery::new(&log);
//! assert_eq!(query.events_by_activity("order:place").len(), 1);
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Errors that can occur when working with OCEL logs.
#[derive(Debug, Error)]
pub enum OcelError {
    /// Error during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Reference to an object that does not exist in the log.
    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    /// Reference to an object type that does not exist.
    #[error("Object type not found: {0}")]
    TypeNotFound(String),

    /// Invalid attribute value for the expected type.
    #[error("Invalid attribute value for type {expected:?}: {found:?}")]
    InvalidAttributeType {
        expected: OcelAttrType,
        found: OcelValue,
    },
}

// ─── Attribute types ──────────────────────────────────────────────────────────

/// The data type of an OCEL attribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OcelAttrType {
    String,
    Integer,
    Float,
    Boolean,
    Timestamp,
}

/// A typed attribute value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OcelValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

// ─── Schema / type definitions ────────────────────────────────────────────────

/// Schema for a single attribute within an object type or event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: OcelAttrType,
}

/// Schema for an object type (e.g., "Blueprint", "RocketStage").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObjectType {
    pub name: String,
    pub attributes: Vec<OcelAttribute>,
}

// ─── Object instances ─────────────────────────────────────────────────────────

/// A single attribute change recorded in an object's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelAttributeChange {
    /// ISO 8601 timestamp of the change.
    pub time: String,
    pub name: String,
    pub value: OcelValue,
}

/// A concrete object instance in the log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcelObject {
    pub id: String,
    pub object_type: String,
    pub attributes: HashMap<String, OcelValue>,
    pub attribute_history: Vec<OcelAttributeChange>,
}

// ─── Events ───────────────────────────────────────────────────────────────────

/// A relationship between an event and an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcelRelationship {
    pub object_id: String,
    /// Semantic qualifier, e.g., "produced", "consumed", "receipted_by".
    pub qualifier: String,
}

/// A process event in the log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    pub id: String,
    /// Activity label, e.g., "blueprint:admit", "rocket:build".
    pub activity: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    pub attributes: HashMap<String, OcelValue>,
    pub relationships: Vec<OcelRelationship>,
}

// ─── The log ─────────────────────────────────────────────────────────────────

/// An OCEL 2.0 compliant event log.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OcelLog {
    pub object_types: Vec<OcelObjectType>,
    pub objects: Vec<OcelObject>,
    pub events: Vec<OcelEvent>,
}

/// Compact statistics about an [`OcelLog`].
#[derive(Debug, Serialize, Deserialize)]
pub struct OcelSummary {
    pub object_type_count: usize,
    pub object_count: usize,
    pub event_count: usize,
    pub activity_count: usize,
    pub relationship_count: usize,
}

impl OcelLog {
    /// Serialize to OCEL 2.0 JSON format (pretty-printed).
    pub fn to_ocel_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Parse from OCEL 2.0 JSON format.
    pub fn from_ocel_json(s: &str) -> Result<Self, OcelError> {
        Ok(serde_json::from_str(s)?)
    }

    /// Return compact statistics for this log.
    pub fn summary(&self) -> OcelSummary {
        let activity_count = self
            .events
            .iter()
            .map(|e| e.activity.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let relationship_count = self.events.iter().map(|e| e.relationships.len()).sum();

        OcelSummary {
            object_type_count: self.object_types.len(),
            object_count: self.objects.len(),
            event_count: self.events.len(),
            activity_count,
            relationship_count,
        }
    }
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Fluent builder for constructing an [`OcelLog`].
///
/// # Example
///
/// ```rust
/// use unify_ocel::{OcelLogBuilder, OcelValue, OcelAttrType};
///
/// let mut b = OcelLogBuilder::new();
/// b.add_object_type("Part", vec![("serial", OcelAttrType::String)]);
/// let id = b.add_object("Part", vec![("serial", OcelValue::String("SN-001".into()))]);
/// b.add_event("part:install", "2024-01-01T00:00:00Z", vec![], vec![(&id, "installed")]);
/// let log = b.build();
/// assert_eq!(log.events.len(), 1);
/// ```
pub struct OcelLogBuilder {
    log: OcelLog,
    event_counter: u64,
    object_counter: u64,
}

impl Default for OcelLogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OcelLogBuilder {
    /// Create a new, empty builder.
    pub fn new() -> Self {
        Self {
            log: OcelLog::default(),
            event_counter: 0,
            object_counter: 0,
        }
    }

    /// Register an object type with its attribute schema.
    ///
    /// ```rust
    /// use unify_ocel::{OcelLogBuilder, OcelAttrType};
    /// let mut b = OcelLogBuilder::new();
    /// b.add_object_type("Blueprint", vec![("version", OcelAttrType::Integer)]);
    /// assert_eq!(b.build().object_types.len(), 1);
    /// ```
    pub fn add_object_type(
        &mut self,
        name: &str,
        attributes: Vec<(&str, OcelAttrType)>,
    ) -> &mut Self {
        self.log.object_types.push(OcelObjectType {
            name: name.to_owned(),
            attributes: attributes
                .into_iter()
                .map(|(n, t)| OcelAttribute {
                    name: n.to_owned(),
                    attr_type: t,
                })
                .collect(),
        });
        self
    }

    /// Create an object instance and return its generated ID.
    pub fn add_object(&mut self, object_type: &str, attrs: Vec<(&str, OcelValue)>) -> String {
        self.object_counter += 1;
        let id = format!("obj-{}", self.object_counter);
        let attributes = attrs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
        self.log.objects.push(OcelObject {
            id: id.clone(),
            object_type: object_type.to_owned(),
            attributes,
            attribute_history: Vec::new(),
        });
        id
    }

    /// Record an event and return its generated ID.
    pub fn add_event(
        &mut self,
        activity: &str,
        timestamp: &str,
        attrs: Vec<(&str, OcelValue)>,
        relationships: Vec<(&str, &str)>, // (object_id, qualifier)
    ) -> String {
        self.event_counter += 1;
        let id = format!("evt-{}", self.event_counter);
        let attributes = attrs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
        let relationships = relationships
            .into_iter()
            .map(|(oid, q)| OcelRelationship {
                object_id: oid.to_owned(),
                qualifier: q.to_owned(),
            })
            .collect();
        self.log.events.push(OcelEvent {
            id: id.clone(),
            activity: activity.to_owned(),
            timestamp: timestamp.to_owned(),
            attributes,
            relationships,
        });
        id
    }

    /// Append an attribute change to an object's history.
    ///
    /// Also updates the object's current attribute map.
    pub fn update_attribute(
        &mut self,
        object_id: &str,
        name: &str,
        value: OcelValue,
        time: &str,
    ) -> &mut Self {
        if let Some(obj) = self.log.objects.iter_mut().find(|o| o.id == object_id) {
            obj.attribute_history.push(OcelAttributeChange {
                time: time.to_owned(),
                name: name.to_owned(),
                value: value.clone(),
            });
            obj.attributes.insert(name.to_owned(), value);
        }
        self
    }

    /// Consume the builder and return the finished [`OcelLog`].
    pub fn build(self) -> OcelLog {
        self.log
    }
}

// ─── Query engine ─────────────────────────────────────────────────────────────

/// Read-only query interface for an [`OcelLog`].
pub struct OcelQuery<'a> {
    log: &'a OcelLog,
}

impl<'a> OcelQuery<'a> {
    /// Create a new query view over `log`.
    pub fn new(log: &'a OcelLog) -> Self {
        Self { log }
    }

    /// All events whose `activity` matches exactly.
    pub fn events_by_activity(&self, activity: &str) -> Vec<&'a OcelEvent> {
        self.log
            .events
            .iter()
            .filter(|e| e.activity == activity)
            .collect()
    }

    /// All objects whose `object_type` matches exactly.
    pub fn objects_by_type(&self, object_type: &str) -> Vec<&'a OcelObject> {
        self.log
            .objects
            .iter()
            .filter(|o| o.object_type == object_type)
            .collect()
    }

    /// All events that reference `object_id` in their relationships.
    pub fn events_for_object(&self, object_id: &str) -> Vec<&'a OcelEvent> {
        self.log
            .events
            .iter()
            .filter(|e| e.relationships.iter().any(|r| r.object_id == object_id))
            .collect()
    }

    /// Objects referenced by a specific event.
    pub fn objects_for_event(&self, event_id: &str) -> Vec<&'a OcelObject> {
        let event = match self.log.events.iter().find(|e| e.id == event_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        let ids: std::collections::HashSet<&str> = event
            .relationships
            .iter()
            .map(|r| r.object_id.as_str())
            .collect();
        self.log
            .objects
            .iter()
            .filter(|o| ids.contains(o.id.as_str()))
            .collect()
    }

    /// Map from activity name → number of occurrences.
    pub fn event_counts(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for e in &self.log.events {
            *counts.entry(e.activity.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Unique activities in the order they first appear in the log.
    pub fn activities(&self) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for e in &self.log.events {
            if seen.insert(e.activity.as_str()) {
                result.push(e.activity.as_str());
            }
        }
        result
    }

    /// Events where `object_id` appears with the given `qualifier`.
    pub fn events_by_qualifier(&self, object_id: &str, qualifier: &str) -> Vec<&'a OcelEvent> {
        self.log
            .events
            .iter()
            .filter(|e| {
                e.relationships
                    .iter()
                    .any(|r| r.object_id == object_id && r.qualifier == qualifier)
            })
            .collect()
    }
}

// ─── Example logs ────────────────────────────────────────────────────────────

/// Build an example OCEL log for a simulated rocket build pipeline.
///
/// Activities in order:
/// `project:load` → `env:doctor` → `project:audit` → `project:build` → `artifact:export`
pub fn example_rocket_build_log() -> OcelLog {
    let mut b = OcelLogBuilder::new();

    // Object types
    b.add_object_type(
        "UeProject",
        vec![
            ("name", OcelAttrType::String),
            ("engine_version", OcelAttrType::String),
        ],
    );
    b.add_object_type(
        "Artifact",
        vec![
            ("path", OcelAttrType::String),
            ("size_bytes", OcelAttrType::Integer),
        ],
    );
    b.add_object_type(
        "Environment",
        vec![
            ("os", OcelAttrType::String),
            ("ready", OcelAttrType::Boolean),
        ],
    );

    // Objects
    let proj = b.add_object(
        "UeProject",
        vec![
            ("name", OcelValue::String("RocketCraft".into())),
            ("engine_version", OcelValue::String("5.3.0".into())),
        ],
    );
    let artifact = b.add_object(
        "Artifact",
        vec![
            ("path", OcelValue::String("/dist/RocketCraft.pak".into())),
            ("size_bytes", OcelValue::Integer(0)),
        ],
    );
    let env = b.add_object(
        "Environment",
        vec![
            ("os", OcelValue::String("linux".into())),
            ("ready", OcelValue::Boolean(false)),
        ],
    );

    // Events
    b.add_event(
        "project:load",
        "2024-03-01T08:00:00Z",
        vec![("status", OcelValue::String("ok".into()))],
        vec![(&proj, "loaded")],
    );
    b.add_event(
        "env:doctor",
        "2024-03-01T08:01:00Z",
        vec![("checks_passed", OcelValue::Integer(12))],
        vec![(&env, "checked"), (&proj, "context")],
    );
    b.update_attribute(
        &env,
        "ready",
        OcelValue::Boolean(true),
        "2024-03-01T08:01:30Z",
    );

    b.add_event(
        "project:audit",
        "2024-03-01T08:02:00Z",
        vec![("warnings", OcelValue::Integer(0))],
        vec![(&proj, "audited")],
    );
    b.add_event(
        "project:build",
        "2024-03-01T08:05:00Z",
        vec![
            ("duration_s", OcelValue::Float(183.4)),
            ("success", OcelValue::Boolean(true)),
        ],
        vec![(&proj, "built"), (&env, "used"), (&artifact, "produced")],
    );
    b.update_attribute(
        &artifact,
        "size_bytes",
        OcelValue::Integer(2_048_000_000),
        "2024-03-01T08:08:04Z",
    );

    b.add_event(
        "artifact:export",
        "2024-03-01T08:08:10Z",
        vec![("target", OcelValue::String("s3://rocket-builds/".into()))],
        vec![(&artifact, "exported")],
    );

    b.build()
}

/// Build an example OCEL log for a blueprint authoring pipeline.
///
/// Activities in order:
/// `blueprint:draft` → `blueprint:admit` → `blueprint:serialize` → `blueprint:export`
pub fn example_blueprint_authoring_log() -> OcelLog {
    let mut b = OcelLogBuilder::new();

    // Object types
    b.add_object_type(
        "Blueprint",
        vec![
            ("name", OcelAttrType::String),
            ("version", OcelAttrType::Integer),
            ("admitted", OcelAttrType::Boolean),
        ],
    );
    b.add_object_type(
        "ReceiptChain",
        vec![
            ("chain_id", OcelAttrType::String),
            ("length", OcelAttrType::Integer),
        ],
    );
    b.add_object_type(
        "ExportTarget",
        vec![
            ("format", OcelAttrType::String),
            ("path", OcelAttrType::String),
        ],
    );

    // Objects
    let bp = b.add_object(
        "Blueprint",
        vec![
            ("name", OcelValue::String("RocketStage1".into())),
            ("version", OcelValue::Integer(1)),
            ("admitted", OcelValue::Boolean(false)),
        ],
    );
    let chain = b.add_object(
        "ReceiptChain",
        vec![
            ("chain_id", OcelValue::String("rc-0001".into())),
            ("length", OcelValue::Integer(0)),
        ],
    );
    let target = b.add_object(
        "ExportTarget",
        vec![
            ("format", OcelValue::String("json".into())),
            ("path", OcelValue::String("/exports/stage1.json".into())),
        ],
    );

    // Events
    b.add_event(
        "blueprint:draft",
        "2024-04-10T09:00:00Z",
        vec![("author", OcelValue::String("engineer-42".into()))],
        vec![(&bp, "drafted")],
    );
    b.add_event(
        "blueprint:admit",
        "2024-04-10T09:05:00Z",
        vec![("gate_score", OcelValue::Float(0.97))],
        vec![(&bp, "admitted"), (&chain, "receipted_by")],
    );
    b.update_attribute(
        &bp,
        "admitted",
        OcelValue::Boolean(true),
        "2024-04-10T09:05:01Z",
    );
    b.update_attribute(
        &chain,
        "length",
        OcelValue::Integer(1),
        "2024-04-10T09:05:01Z",
    );

    b.add_event(
        "blueprint:serialize",
        "2024-04-10T09:06:00Z",
        vec![("bytes", OcelValue::Integer(4096))],
        vec![(&bp, "serialized"), (&target, "destination")],
    );
    b.add_event(
        "blueprint:export",
        "2024-04-10T09:07:00Z",
        vec![("success", OcelValue::Boolean(true))],
        vec![(&target, "exported"), (&chain, "finalized")],
    );

    b.build()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rocket_log() -> OcelLog {
        example_rocket_build_log()
    }

    fn blueprint_log() -> OcelLog {
        example_blueprint_authoring_log()
    }

    // ── Builder tests ──────────────────────────────────────────────────────

    #[test]
    fn builder_add_object_type_registers_type() {
        let mut b = OcelLogBuilder::new();
        b.add_object_type("Order", vec![("amount", OcelAttrType::Float)]);
        let log = b.build();
        assert_eq!(log.object_types.len(), 1);
        assert_eq!(log.object_types[0].name, "Order");
        assert_eq!(log.object_types[0].attributes[0].name, "amount");
    }

    #[test]
    fn builder_add_object_creates_object_with_attributes() {
        let mut b = OcelLogBuilder::new();
        b.add_object_type("Widget", vec![("color", OcelAttrType::String)]);
        let id = b.add_object("Widget", vec![("color", OcelValue::String("red".into()))]);
        let log = b.build();
        assert_eq!(log.objects.len(), 1);
        let obj = &log.objects[0];
        assert_eq!(obj.id, id);
        assert_eq!(obj.object_type, "Widget");
        assert_eq!(obj.attributes["color"], OcelValue::String("red".into()));
    }

    #[test]
    fn builder_add_event_creates_event_with_relationships() {
        let mut b = OcelLogBuilder::new();
        b.add_object_type("Item", vec![]);
        let item_id = b.add_object("Item", vec![]);
        let evt_id = b.add_event(
            "item:ship",
            "2024-01-01T00:00:00Z",
            vec![("priority", OcelValue::Integer(1))],
            vec![(&item_id, "shipped")],
        );
        let log = b.build();
        assert_eq!(log.events.len(), 1);
        let evt = &log.events[0];
        assert_eq!(evt.id, evt_id);
        assert_eq!(evt.activity, "item:ship");
        assert_eq!(evt.relationships.len(), 1);
        assert_eq!(evt.relationships[0].object_id, item_id);
        assert_eq!(evt.relationships[0].qualifier, "shipped");
    }

    #[test]
    fn builder_update_attribute_adds_to_history() {
        let mut b = OcelLogBuilder::new();
        b.add_object_type("Sensor", vec![("value", OcelAttrType::Float)]);
        let sid = b.add_object("Sensor", vec![("value", OcelValue::Float(0.0))]);
        b.update_attribute(
            &sid,
            "value",
            OcelValue::Float(42.5),
            "2024-06-01T12:00:00Z",
        );
        let log = b.build();
        let obj = &log.objects[0];
        assert_eq!(obj.attribute_history.len(), 1);
        assert_eq!(obj.attribute_history[0].name, "value");
        assert_eq!(obj.attribute_history[0].value, OcelValue::Float(42.5));
        assert_eq!(obj.attributes["value"], OcelValue::Float(42.5));
    }

    #[test]
    fn builder_update_attribute_current_value_updated() {
        let mut b = OcelLogBuilder::new();
        b.add_object_type("Counter", vec![("n", OcelAttrType::Integer)]);
        let cid = b.add_object("Counter", vec![("n", OcelValue::Integer(0))]);
        b.update_attribute(&cid, "n", OcelValue::Integer(1), "T1");
        b.update_attribute(&cid, "n", OcelValue::Integer(2), "T2");
        let log = b.build();
        assert_eq!(log.objects[0].attributes["n"], OcelValue::Integer(2));
        assert_eq!(log.objects[0].attribute_history.len(), 2);
    }

    #[test]
    fn builder_ids_are_unique_across_types() {
        let mut b = OcelLogBuilder::new();
        b.add_object_type("A", vec![]);
        b.add_object_type("B", vec![]);
        let id1 = b.add_object("A", vec![]);
        let id2 = b.add_object("B", vec![]);
        assert_ne!(id1, id2);
    }

    // ── Serialization tests ────────────────────────────────────────────────

    #[test]
    fn ocel_log_round_trips_json() {
        let log = rocket_log();
        let json = log.to_ocel_json();
        let restored = OcelLog::from_ocel_json(&json).expect("should parse");
        assert_eq!(restored.events.len(), log.events.len());
        assert_eq!(restored.objects.len(), log.objects.len());
        assert_eq!(restored.object_types.len(), log.object_types.len());
    }

    #[test]
    fn ocel_json_contains_object_types_key() {
        let log = rocket_log();
        let json = log.to_ocel_json();
        assert!(
            json.contains("objectTypes"),
            "OCEL 2.0 JSON must contain 'objectTypes' key"
        );
    }

    #[test]
    fn ocel_json_contains_events_key() {
        let log = blueprint_log();
        let json = log.to_ocel_json();
        assert!(
            json.contains("\"events\""),
            "JSON must contain events array"
        );
    }

    #[test]
    fn from_ocel_json_rejects_invalid_json() {
        let result = OcelLog::from_ocel_json("not json at all");
        assert!(result.is_err());
    }

    // ── Query tests ────────────────────────────────────────────────────────

    #[test]
    fn query_events_by_activity_returns_matching_events() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        let evts = q.events_by_activity("project:build");
        assert_eq!(evts.len(), 1);
        assert_eq!(evts[0].activity, "project:build");
    }

    #[test]
    fn query_events_by_activity_returns_empty_for_unknown() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        assert!(q.events_by_activity("does:not:exist").is_empty());
    }

    #[test]
    fn query_objects_by_type_returns_matching_objects() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        let objs = q.objects_by_type("UeProject");
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].object_type, "UeProject");
    }

    #[test]
    fn query_events_for_object_returns_related_events() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        // The artifact object is referenced in "project:build" and "artifact:export"
        let artifact_obj = q.objects_by_type("Artifact")[0];
        let evts = q.events_for_object(&artifact_obj.id);
        let activities: Vec<&str> = evts.iter().map(|e| e.activity.as_str()).collect();
        assert!(activities.contains(&"project:build"));
        assert!(activities.contains(&"artifact:export"));
    }

    #[test]
    fn query_objects_for_event_returns_correct_objects() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        let build_evt = q.events_by_activity("project:build")[0];
        let objs = q.objects_for_event(&build_evt.id);
        // project:build references UeProject, Environment, and Artifact
        assert_eq!(objs.len(), 3);
    }

    #[test]
    fn query_event_counts_sums_correctly() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        let counts = q.event_counts();
        assert_eq!(*counts.get("project:load").unwrap_or(&0), 1);
        assert_eq!(*counts.get("env:doctor").unwrap_or(&0), 1);
        assert_eq!(*counts.get("artifact:export").unwrap_or(&0), 1);
        // Total unique activities
        assert_eq!(counts.len(), 5);
    }

    #[test]
    fn query_activities_returns_unique_in_order() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        let acts = q.activities();
        // No duplicates
        let unique: std::collections::HashSet<&&str> = acts.iter().collect();
        assert_eq!(acts.len(), unique.len());
        // First activity is project:load
        assert_eq!(acts[0], "project:load");
    }

    #[test]
    fn query_events_by_qualifier_filters_correctly() {
        let log = blueprint_log();
        let q = OcelQuery::new(&log);
        let bp = q.objects_by_type("Blueprint")[0];
        let evts = q.events_by_qualifier(&bp.id, "admitted");
        assert_eq!(evts.len(), 1);
        assert_eq!(evts[0].activity, "blueprint:admit");
    }

    #[test]
    fn query_events_by_qualifier_returns_empty_for_wrong_qualifier() {
        let log = blueprint_log();
        let q = OcelQuery::new(&log);
        let bp = q.objects_by_type("Blueprint")[0];
        let evts = q.events_by_qualifier(&bp.id, "nonexistent_qualifier");
        assert!(evts.is_empty());
    }

    // ── Summary tests ──────────────────────────────────────────────────────

    #[test]
    fn summary_event_count_matches_actual() {
        let log = rocket_log();
        let summary = log.summary();
        assert_eq!(summary.event_count, log.events.len());
    }

    #[test]
    fn summary_relationship_count_is_correct() {
        let log = rocket_log();
        let total_rels: usize = log.events.iter().map(|e| e.relationships.len()).sum();
        assert_eq!(log.summary().relationship_count, total_rels);
    }

    #[test]
    fn summary_activity_count_is_unique() {
        let log = rocket_log();
        let summary = log.summary();
        let q = OcelQuery::new(&log);
        assert_eq!(summary.activity_count, q.activities().len());
    }

    // ── Example log tests ─────────────────────────────────────────────────

    #[test]
    fn rocket_build_log_has_at_least_five_events() {
        let log = rocket_log();
        assert!(
            log.events.len() >= 5,
            "expected ≥5 events, got {}",
            log.events.len()
        );
    }

    #[test]
    fn rocket_build_log_activities_correct_order() {
        let log = rocket_log();
        let q = OcelQuery::new(&log);
        let acts = q.activities();
        assert_eq!(acts[0], "project:load");
        assert_eq!(acts[1], "env:doctor");
        assert_eq!(acts[2], "project:audit");
        assert_eq!(acts[3], "project:build");
        assert_eq!(acts[4], "artifact:export");
    }

    #[test]
    fn blueprint_authoring_log_has_at_least_four_events() {
        let log = blueprint_log();
        assert!(
            log.events.len() >= 4,
            "expected ≥4 events, got {}",
            log.events.len()
        );
    }

    #[test]
    fn blueprint_authoring_log_activities_correct_order() {
        let log = blueprint_log();
        let q = OcelQuery::new(&log);
        let acts = q.activities();
        assert_eq!(acts[0], "blueprint:draft");
        assert_eq!(acts[1], "blueprint:admit");
        assert_eq!(acts[2], "blueprint:serialize");
        assert_eq!(acts[3], "blueprint:export");
    }

    #[test]
    fn blueprint_log_attribute_history_recorded() {
        let log = blueprint_log();
        // The Blueprint object gets "admitted" updated
        let bp = log
            .objects
            .iter()
            .find(|o| o.object_type == "Blueprint")
            .unwrap();
        assert!(
            !bp.attribute_history.is_empty(),
            "Blueprint should have attribute history"
        );
        let admitted_change = bp
            .attribute_history
            .iter()
            .find(|c| c.name == "admitted")
            .unwrap();
        assert_eq!(admitted_change.value, OcelValue::Boolean(true));
    }

    #[test]
    fn ocel_value_null_serializes_as_null() {
        let val = OcelValue::Null;
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn ocel_value_boolean_roundtrips() {
        let val = OcelValue::Boolean(true);
        let json = serde_json::to_string(&val).unwrap();
        let restored: OcelValue = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, OcelValue::Boolean(true));
    }
}
