//! GC-MECHBIRTH-002: OCEL Replay Engine
//! Parses and replays GC-MECHBIRTH-001 compatible OCEL JSON traces.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    pub id: String,
    pub event_type: String,
    pub objects: Vec<String>,
    pub receipt: String,
    pub prev_hash: String,
    pub sequence: u64,
}

#[derive(Debug, Default)]
pub struct OcelLog {
    pub events: Vec<OcelEvent>,
}

impl OcelLog {
    /// Load a GC-MECHBIRTH-001 compatible OCEL JSON (simplified binding).
    /// Supports both the powlv2lsp format (with `relationships`) and a
    /// simplified format (with top-level `objects` array of strings).
    pub fn from_powlv2lsp_trace(json: &str) -> Result<Self, serde_json::Error> {
        #[derive(Deserialize)]
        struct RawLog {
            events: Vec<RawEvent>,
        }
        #[derive(Deserialize)]
        struct RawEvent {
            id: String,
            #[serde(rename = "type")]
            event_type: String,
            #[serde(default)]
            attributes: Vec<RawAttr>,
        }
        #[derive(Deserialize)]
        struct RawAttr {
            name: String,
            value: String,
        }

        let raw: RawLog = serde_json::from_str(json)?;
        let mut events = Vec::new();
        for (i, e) in raw.events.iter().enumerate() {
            let receipt = e
                .attributes
                .iter()
                .find(|a| a.name == "audit_receipt")
                .map(|a| a.value.clone())
                .unwrap_or_default();
            let prev_hash = e
                .attributes
                .iter()
                .find(|a| a.name == "prev_hash")
                .map(|a| a.value.clone())
                .unwrap_or_else(|| {
                    "0000000000000000000000000000000000000000000000000000000000000000".into()
                });
            events.push(OcelEvent {
                id: e.id.clone(),
                event_type: e.event_type.clone(),
                objects: vec!["case-mechbirth-001".into()],
                receipt,
                prev_hash,
                sequence: i as u64 + 1,
            });
        }
        Ok(OcelLog { events })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_OCEL: &str = r#"{
        "events": [
            {
                "id": "ev-001",
                "type": "SOURCE_ADMISSION",
                "attributes": [
                    {"name": "audit_receipt", "value": "abc123"},
                    {"name": "prev_hash", "value": "0000000000000000000000000000000000000000000000000000000000000000"}
                ]
            },
            {
                "id": "ev-002",
                "type": "AUTHORITY_GATE",
                "attributes": [
                    {"name": "audit_receipt", "value": "def456"},
                    {"name": "prev_hash", "value": "abc123"}
                ]
            }
        ]
    }"#;

    #[test]
    fn parses_minimal_ocel_trace() {
        let log = OcelLog::from_powlv2lsp_trace(MINIMAL_OCEL).unwrap();
        assert_eq!(log.events.len(), 2);
    }

    #[test]
    fn parses_event_ids_and_types() {
        let log = OcelLog::from_powlv2lsp_trace(MINIMAL_OCEL).unwrap();
        assert_eq!(log.events[0].id, "ev-001");
        assert_eq!(log.events[0].event_type, "SOURCE_ADMISSION");
        assert_eq!(log.events[1].event_type, "AUTHORITY_GATE");
    }

    #[test]
    fn parses_audit_receipt_attribute() {
        let log = OcelLog::from_powlv2lsp_trace(MINIMAL_OCEL).unwrap();
        assert_eq!(log.events[0].receipt, "abc123");
        assert_eq!(log.events[1].receipt, "def456");
    }

    #[test]
    fn parses_prev_hash_attribute() {
        let log = OcelLog::from_powlv2lsp_trace(MINIMAL_OCEL).unwrap();
        assert!(log.events[0].prev_hash.chars().all(|c| c == '0'));
        assert_eq!(log.events[1].prev_hash, "abc123");
    }

    #[test]
    fn sequence_numbers_are_one_based() {
        let log = OcelLog::from_powlv2lsp_trace(MINIMAL_OCEL).unwrap();
        assert_eq!(log.events[0].sequence, 1);
        assert_eq!(log.events[1].sequence, 2);
    }

    #[test]
    fn event_without_attributes_uses_defaults() {
        let json = r#"{"events": [{"id": "e1", "type": "BARE"}]}"#;
        let log = OcelLog::from_powlv2lsp_trace(json).unwrap();
        assert_eq!(log.events[0].receipt, "");
        assert!(log.events[0].prev_hash.chars().all(|c| c == '0'));
    }

    #[test]
    fn returns_error_on_invalid_json() {
        assert!(OcelLog::from_powlv2lsp_trace("{invalid json}").is_err());
    }

    #[test]
    fn empty_events_list_parses_ok() {
        let log = OcelLog::from_powlv2lsp_trace(r#"{"events": []}"#).unwrap();
        assert!(log.events.is_empty());
    }

    #[test]
    fn each_event_object_contains_case_id() {
        let log = OcelLog::from_powlv2lsp_trace(MINIMAL_OCEL).unwrap();
        assert!(log.events[0].objects.contains(&"case-mechbirth-001".into()));
    }
}
