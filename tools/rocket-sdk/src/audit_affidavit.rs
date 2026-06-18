//! Affidavit-compatible provenance receipts for audit runs.
//!
//! Implements the same BLAKE3 chain algorithm as `seanchatmangpt/affidavit`
//! (FORMAT_VERSION = "core/v1", GENESIS_SEED = b"affidavit-v26.6.14-genesis",
//! canonical JSON with sorted keys), so receipts written here pass `affi verify`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;

const FORMAT_VERSION: &str = "core/v1";
const GENESIS_SEED: &[u8] = b"affidavit-v26.6.14-genesis";

// ── Types (field names match affidavit's serde layout exactly) ───────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(blake3::hash(data).to_hex().to_string())
    }

    pub fn as_hex(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRef {
    pub id: String,
    pub obj_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEvent {
    pub id: String,
    pub seq: u64,
    pub event_type: String,
    pub objects: Vec<ObjectRef>,
    pub payload_commitment: Blake3Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub format_version: String,
    pub events: Vec<OperationEvent>,
    pub chain_hash: Blake3Hash,
}

// ── Audit input ───────────────────────────────────────────────────────────────

pub struct AuditEvent {
    pub project_name: String,
    pub passed: bool,
    pub violations: Vec<(String, String)>,
}

// ── Chain implementation (mirrors affidavit chain.rs exactly) ─────────────────

fn sort_value(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let sorted: serde_json::Map<_, _> = map
                .into_iter()
                .map(|(k, v)| (k, sort_value(v)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect();
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_value).collect())
        }
        other => other,
    }
}

fn canonical_bytes<T: Serialize>(v: &T) -> Vec<u8> {
    let value = serde_json::to_value(v).expect("type is always serializable");
    serde_json::to_vec(&sort_value(value)).expect("sorted value is always serializable")
}

fn genesis_hash() -> Blake3Hash {
    Blake3Hash(blake3::hash(GENESIS_SEED).to_hex().to_string())
}

fn fold_event(prev: &Blake3Hash, event: &OperationEvent) -> Blake3Hash {
    let mut data = Vec::new();
    data.extend_from_slice(prev.as_hex().as_bytes());
    data.extend_from_slice(&canonical_bytes(event));
    Blake3Hash(blake3::hash(&data).to_hex().to_string())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Build an affidavit-compatible receipt from project audit events.
/// Each project becomes one OperationEvent in the BLAKE3 chain.
pub fn record_audit(events: &[AuditEvent]) -> Result<Receipt> {
    let mut running = genesis_hash();
    let mut ops: Vec<OperationEvent> = Vec::with_capacity(events.len());

    for (seq, ev) in events.iter().enumerate() {
        let payload = serde_json::json!({
            "project": ev.project_name,
            "passed": ev.passed,
            "violations": ev.violations.iter()
                .map(|(law, msg)| serde_json::json!({"law": law, "message": msg}))
                .collect::<Vec<_>>()
        });
        let payload_bytes = serde_json::to_vec(&sort_value(payload))?;

        let op = OperationEvent {
            id: format!("rocket.audit.{}.{seq}", ev.project_name),
            seq: seq as u64,
            event_type: "rocket.audit.project".to_string(),
            objects: vec![ObjectRef {
                id: ev.project_name.clone(),
                obj_type: "UE4Project".to_string(),
                qualifier: Some(if ev.passed { "passed" } else { "failed" }.to_string()),
            }],
            payload_commitment: Blake3Hash::from_bytes(&payload_bytes),
        };

        running = fold_event(&running, &op);
        ops.push(op);
    }

    Ok(Receipt {
        format_version: FORMAT_VERSION.to_string(),
        events: ops,
        chain_hash: running,
    })
}

/// Write the receipt to `<root>/.ggen/receipts/affidavit-<ts>.json`
/// and mirror it to `latest.json` as the HEAD pointer.
/// Returns the timestamped path written.
pub fn persist_receipt(receipt: &Receipt, root: &Path) -> Result<PathBuf> {
    let dir = root.join(".ggen").join("receipts");
    fs::create_dir_all(&dir)?;

    let ts = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let path = dir.join(format!("affidavit-{ts}.json"));
    let json = serde_json::to_string_pretty(receipt)?;

    fs::write(&path, &json)?;
    fs::write(dir.join("latest.json"), &json)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_events(n: usize) -> Vec<AuditEvent> {
        (0..n)
            .map(|i| AuditEvent {
                project_name: format!("Project{i}"),
                passed: i % 2 == 0,
                violations: if i % 2 == 0 {
                    vec![]
                } else {
                    vec![("TestLaw".into(), "missing keystore".into())]
                },
            })
            .collect()
    }

    #[test]
    fn empty_audit_produces_receipt() {
        let receipt = record_audit(&[]).unwrap();
        assert_eq!(receipt.format_version, FORMAT_VERSION);
        assert!(receipt.events.is_empty());
        // chain_hash of empty chain == genesis_hash
        assert_eq!(receipt.chain_hash.as_hex(), genesis_hash().as_hex());
    }

    #[test]
    fn receipt_chain_hash_changes_with_events() {
        let r1 = record_audit(&make_events(1)).unwrap();
        let r2 = record_audit(&make_events(2)).unwrap();
        assert_ne!(r1.chain_hash.as_hex(), r2.chain_hash.as_hex());
    }

    #[test]
    fn tamper_detection_via_chain_recompute() {
        let mut events = make_events(2);
        let receipt = record_audit(&events).unwrap();
        let original_hash = receipt.chain_hash.0.clone();

        // Mutate one event payload and rebuild — hash must differ.
        events[0].project_name = "TamperedProject".into();
        let tampered = record_audit(&events).unwrap();
        assert_ne!(tampered.chain_hash.0, original_hash);
    }

    #[test]
    fn persist_writes_both_files() {
        let dir = tempdir().unwrap();
        let events = make_events(3);
        let receipt = record_audit(&events).unwrap();
        let path = persist_receipt(&receipt, dir.path()).unwrap();

        assert!(path.exists(), "timestamped file must exist");
        assert!(dir.path().join(".ggen/receipts/latest.json").exists(), "latest.json must exist");

        let loaded: Receipt = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.events.len(), 3);
        assert_eq!(loaded.chain_hash.as_hex(), receipt.chain_hash.as_hex());
    }

    #[test]
    fn deterministic_hash_for_same_input() {
        let events = make_events(4);
        let r1 = record_audit(&events).unwrap();
        let r2 = record_audit(&events).unwrap();
        assert_eq!(r1.chain_hash.as_hex(), r2.chain_hash.as_hex());
    }

    #[test]
    fn seq_numbers_are_monotone() {
        let events = make_events(5);
        let receipt = record_audit(&events).unwrap();
        for (i, op) in receipt.events.iter().enumerate() {
            assert_eq!(op.seq, i as u64);
        }
    }
}
