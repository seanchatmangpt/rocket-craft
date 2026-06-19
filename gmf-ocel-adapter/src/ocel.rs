use serde::{Deserialize, Serialize};

/// A reference from an OCEL event to an object, with a qualifying role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObjectRef {
    pub object_id: String,
    pub qualifier: String,
}

/// An attribute change on an OCEL object at a specific timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelAttributeChange {
    pub attribute: String,
    pub value: serde_json::Value,
    pub timestamp_ms: u64,
}

/// OCEL 2.0 event: a single activity occurrence referencing multiple objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    pub id: String,
    pub activity: String,
    pub timestamp_ms: u64,
    pub object_refs: Vec<OcelObjectRef>,
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

/// OCEL 2.0 object: an entity with a type and an attribute change history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObject {
    pub id: String,
    pub object_type: String,
    pub attribute_changes: Vec<OcelAttributeChange>,
}

impl OcelObject {
    pub fn new(id: impl Into<String>, object_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object_type: object_type.into(),
            attribute_changes: Vec::new(),
        }
    }

    pub fn with_attr_change(mut self, attr: impl Into<String>, val: impl Into<serde_json::Value>, ts: u64) -> Self {
        self.attribute_changes.push(OcelAttributeChange {
            attribute: attr.into(),
            value: val.into(),
            timestamp_ms: ts,
        });
        self
    }
}

/// The top-level OCEL log: a collection of typed objects and their events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcelLog {
    pub objects: Vec<OcelObject>,
    pub events: Vec<OcelEvent>,
}

impl OcelLog {
    pub fn add_object(&mut self, obj: OcelObject) { self.objects.push(obj); }
    pub fn add_event(&mut self, ev: OcelEvent) { self.events.push(ev); }

    /// Validate referential integrity: every object_ref in every event must
    /// reference a known object id. Returns a list of violation messages.
    pub fn validate(&self) -> Vec<String> {
        let known: std::collections::HashSet<&str> =
            self.objects.iter().map(|o| o.id.as_str()).collect();
        let mut violations = Vec::new();

        for event in &self.events {
            for oref in &event.object_refs {
                if !known.contains(oref.object_id.as_str()) {
                    violations.push(format!(
                        "event '{}' references unknown object '{}'",
                        event.id, oref.object_id
                    ));
                }
            }
        }

        // Temporal monotonicity: object attribute changes must be non-decreasing in time
        for obj in &self.objects {
            let mut prev_ts = 0u64;
            for change in &obj.attribute_changes {
                if change.timestamp_ms < prev_ts {
                    violations.push(format!(
                        "object '{}' attribute '{}' has non-monotonic timestamp {} < {}",
                        obj.id, change.attribute, change.timestamp_ms, prev_ts
                    ));
                }
                prev_ts = change.timestamp_ms;
            }
        }

        violations
    }

    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_event(id: &str, activity: &str, object_refs: Vec<OcelObjectRef>) -> OcelEvent {
        OcelEvent {
            id: id.into(), activity: activity.into(),
            timestamp_ms: 1000,
            object_refs,
            attributes: serde_json::Map::new(),
        }
    }

    fn obj_ref(object_id: &str) -> OcelObjectRef {
        OcelObjectRef { object_id: object_id.into(), qualifier: "target".into() }
    }

    // ── OcelObject ────────────────────────────────────────────────────────────

    #[test]
    fn new_object_has_empty_changes() {
        let obj = OcelObject::new("p-1", "MechPart");
        assert_eq!(obj.id, "p-1");
        assert_eq!(obj.object_type, "MechPart");
        assert!(obj.attribute_changes.is_empty());
    }

    #[test]
    fn with_attr_change_appends_change() {
        let obj = OcelObject::new("p-1", "MechPart")
            .with_attr_change("status", json!("active"), 500);
        assert_eq!(obj.attribute_changes.len(), 1);
        assert_eq!(obj.attribute_changes[0].attribute, "status");
        assert_eq!(obj.attribute_changes[0].timestamp_ms, 500);
    }

    // ── OcelLog ───────────────────────────────────────────────────────────────

    #[test]
    fn empty_log_is_valid() {
        let log = OcelLog::default();
        assert!(log.validate().is_empty());
    }

    #[test]
    fn valid_event_ref_produces_no_violation() {
        let mut log = OcelLog::default();
        log.add_object(OcelObject::new("obj-1", "MechPart"));
        log.add_event(make_event("e-1", "Assemble", vec![obj_ref("obj-1")]));
        assert!(log.validate().is_empty());
    }

    #[test]
    fn unknown_object_ref_produces_violation() {
        let mut log = OcelLog::default();
        log.add_event(make_event("e-1", "Assemble", vec![obj_ref("missing")]));
        let v = log.validate();
        assert!(!v.is_empty());
        assert!(v[0].contains("missing"));
    }

    #[test]
    fn non_monotonic_attribute_change_produces_violation() {
        let mut log = OcelLog::default();
        let obj = OcelObject::new("o-1", "Part")
            .with_attr_change("heat", json!(100), 1000)
            .with_attr_change("heat", json!(200), 500); // going backwards in time
        log.add_object(obj);
        let v = log.validate();
        assert!(!v.is_empty());
        assert!(v[0].contains("non-monotonic"));
    }

    #[test]
    fn to_json_pretty_roundtrips() {
        let mut log = OcelLog::default();
        log.add_object(OcelObject::new("x", "Type"));
        let json = log.to_json_pretty().unwrap();
        let back: OcelLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back.objects.len(), 1);
    }
}
