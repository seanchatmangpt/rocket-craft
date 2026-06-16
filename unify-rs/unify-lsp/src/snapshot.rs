use crate::conformance::ConformanceScore;

/// A record of system state captured at a point in time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotRecord {
    /// Unix timestamp (ms) when the snapshot was taken.
    pub timestamp_ms: u64,
    /// Conformance score at snapshot time.
    pub conformance: ConformanceScore,
    /// Human-readable label or description.
    pub label: String,
}

impl SnapshotRecord {
    /// Create a new snapshot with the current time.
    pub fn new(label: impl Into<String>, conformance: ConformanceScore) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            timestamp_ms,
            conformance,
            label: label.into(),
        }
    }
}
