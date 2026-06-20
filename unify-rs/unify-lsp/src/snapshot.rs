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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::ConformanceScore;

    #[test]
    fn new_snapshot_has_nonzero_timestamp() {
        let s = SnapshotRecord::new("test", ConformanceScore::perfect());
        assert!(s.timestamp_ms > 0, "timestamp should be set from system time");
    }

    #[test]
    fn label_is_preserved() {
        let s = SnapshotRecord::new("pipeline-check", ConformanceScore::zero());
        assert_eq!(s.label, "pipeline-check");
    }

    #[test]
    fn conformance_score_is_preserved() {
        let score = ConformanceScore::perfect();
        let s = SnapshotRecord::new("x", score.clone());
        assert_eq!(s.conformance.fitness, score.fitness);
    }

    #[test]
    fn snapshot_serializes_to_json() {
        let s = SnapshotRecord::new("check", ConformanceScore::perfect());
        let json = serde_json::to_string(&s).expect("should serialize");
        assert!(json.contains("timestamp_ms"));
        assert!(json.contains("label"));
    }
}
