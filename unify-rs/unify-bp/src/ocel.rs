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
