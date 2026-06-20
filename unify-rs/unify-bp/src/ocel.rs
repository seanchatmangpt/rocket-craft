//! BlueprintOcelBridge — emit OCEL-like events from Blueprint operations.

use blueprint_core::Blueprint;

/// A single OCEL event recording a Blueprint operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BpOcelEvent {
    /// Unique event ID (monotonic counter prefixed with "bp-").
    pub id: String,
    /// Activity label, e.g. "blueprint:generate", "blueprint:validate".
    pub activity: String,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// The name of the Blueprint involved in this event.
    pub blueprint_name: String,
}

/// Collects OCEL events from Blueprint operations.
pub struct BlueprintOcelBridge {
    events: Vec<BpOcelEvent>,
}

impl Default for BlueprintOcelBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintOcelBridge {
    /// Create a new, empty OCEL bridge.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn next_id(&self) -> String {
        format!("bp-{}", self.events.len() + 1)
    }

    /// Record a Blueprint generation event and return a reference to it.
    pub fn record_generation(&mut self, bp: &Blueprint) -> &BpOcelEvent {
        let event = BpOcelEvent {
            id: self.next_id(),
            activity: "blueprint:generate".to_string(),
            timestamp_ms: Self::now_ms(),
            blueprint_name: bp.name.clone(),
        };
        self.events.push(event);
        self.events.last().unwrap()
    }

    /// Record a Blueprint validation event and return a reference to it.
    ///
    /// `errors` is the number of validation errors found (0 means passed).
    pub fn record_validation(&mut self, bp: &Blueprint, errors: usize) -> &BpOcelEvent {
        let activity = if errors == 0 {
            "blueprint:validate:pass".to_string()
        } else {
            format!("blueprint:validate:fail:{}", errors)
        };
        let event = BpOcelEvent {
            id: self.next_id(),
            activity,
            timestamp_ms: Self::now_ms(),
            blueprint_name: bp.name.clone(),
        };
        self.events.push(event);
        self.events.last().unwrap()
    }

    /// Record a T3D export event and return a reference to it.
    pub fn record_t3d_export(&mut self, bp: &Blueprint) -> &BpOcelEvent {
        let event = BpOcelEvent {
            id: self.next_id(),
            activity: "blueprint:export:t3d".to_string(),
            timestamp_ms: Self::now_ms(),
            blueprint_name: bp.name.clone(),
        };
        self.events.push(event);
        self.events.last().unwrap()
    }

    /// Return all recorded events.
    pub fn events(&self) -> &[BpOcelEvent] {
        &self.events
    }

    /// Serialize all events to a pretty-printed JSON array.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap_or_else(|_| "[]".to_string())
    }

    /// Return the total number of events recorded.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueprint_core::Blueprint;

    fn make_bp(name: &str) -> Blueprint {
        Blueprint::new(name, "Actor")
    }

    // ── BlueprintOcelBridge::new ──────────────────────────────────────────────

    #[test]
    fn new_starts_empty() {
        let bridge = BlueprintOcelBridge::new();
        assert_eq!(bridge.event_count(), 0);
        assert!(bridge.events().is_empty());
    }

    // ── record_generation ────────────────────────────────────────────────────

    #[test]
    fn record_generation_sets_activity_and_name() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = make_bp("MyActor");
        let ev = bridge.record_generation(&bp);
        assert_eq!(ev.activity, "blueprint:generate");
        assert_eq!(ev.blueprint_name, "MyActor");
    }

    #[test]
    fn record_generation_assigns_monotonic_id() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = make_bp("BP");
        let ev1 = bridge.record_generation(&bp).id.clone();
        let ev2 = bridge.record_generation(&bp).id.clone();
        assert_eq!(ev1, "bp-1");
        assert_eq!(ev2, "bp-2");
    }

    // ── record_validation ────────────────────────────────────────────────────

    #[test]
    fn record_validation_pass_when_zero_errors() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = make_bp("BP");
        let ev = bridge.record_validation(&bp, 0);
        assert_eq!(ev.activity, "blueprint:validate:pass");
    }

    #[test]
    fn record_validation_fail_encodes_error_count() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = make_bp("BP");
        let ev = bridge.record_validation(&bp, 3);
        assert_eq!(ev.activity, "blueprint:validate:fail:3");
    }

    // ── record_t3d_export ────────────────────────────────────────────────────

    #[test]
    fn record_t3d_export_sets_correct_activity() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = make_bp("Mesh");
        let ev = bridge.record_t3d_export(&bp);
        assert_eq!(ev.activity, "blueprint:export:t3d");
        assert_eq!(ev.blueprint_name, "Mesh");
    }

    // ── to_json ───────────────────────────────────────────────────────────────

    #[test]
    fn to_json_empty_produces_empty_array() {
        let bridge = BlueprintOcelBridge::new();
        assert_eq!(bridge.to_json().trim(), "[]");
    }

    #[test]
    fn to_json_includes_activity_field() {
        let mut bridge = BlueprintOcelBridge::new();
        bridge.record_generation(&make_bp("BP"));
        let json = bridge.to_json();
        assert!(json.contains("blueprint:generate"));
    }
}
