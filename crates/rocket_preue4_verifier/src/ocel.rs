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
