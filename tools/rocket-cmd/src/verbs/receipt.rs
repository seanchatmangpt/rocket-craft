//! Receipt validation commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_validate_receipt(file: String) -> Result<Value> {
    use std::fs;
    let raw = fs::read_to_string(&file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("cannot read {}: {}", file, e))
    })?;
    let val: Value = serde_json::from_str(&raw).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("invalid JSON in {}: {}", file, e))
    })?;

    let errors = validate_receipt_value(&val);

    if errors.is_empty() {
        let run_id = val.get("run_id").and_then(|v| v.as_str()).unwrap_or("?");
        let hash = val
            .get("output_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        println!("PASS  run={run_id}  output={hash}  file={file}");
        Ok(serde_json::json!({"status": "pass", "file": file, "run_id": run_id}))
    } else {
        Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "FAIL {file}: {}",
            errors.join("; ")
        )))
    }
}

/// Validate a receipt JSON has required fields and a passing verdict
///
/// # Arguments
/// * `file` - Path to the receipt JSON file
#[verb("validate", "receipt")]
fn validate_receipt(file: String) -> Result<Value> {
    do_validate_receipt(file)
}

/// Pure validation logic extracted for unit testing.
/// Returns a list of validation error strings; empty = valid.
fn validate_receipt_value(val: &Value) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    for field in &["verdict", "output_hash", "run_id", "timestamp", "signature"] {
        if val.get(field).is_none() {
            errors.push(format!("missing field '{field}'"));
        }
    }

    if let Some(v) = val.get("verdict").and_then(|v| v.as_str()) {
        if v != "PASS" {
            errors.push(format!("verdict is '{v}', expected 'PASS'"));
        }
    }

    if let Some(h) = val.get("output_hash").and_then(|v| v.as_str()) {
        if h.is_empty() {
            errors.push("output_hash is empty".into());
        } else if !h.starts_with("sha256:") && !h.starts_with("blake3:") {
            errors.push(format!("output_hash has unknown algorithm prefix: '{h}'"));
        }
    }

    if let Some(id) = val.get("run_id").and_then(|v| v.as_str()) {
        if id.is_empty() {
            errors.push("run_id is empty".into());
        }
    }

    if let Some(delta) = val.get("visualDelta").and_then(|v| v.as_u64()) {
        if delta < 20 {
            errors.push(format!("visualDelta={delta} < 20 — canvas appears static"));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn proven_receipt() -> Value {
        json!({
            "verdict": "PASS",
            "output_hash": "sha256:a737628e84f38eec9ef91495d6fb1522710929f4a5e31496f12c0f1b5ac76f10",
            "run_id": "tps-dflss-1781855287977",
            "timestamp": "2026-06-17T00:00:00Z",
            "signature": "deb3010a",
            "visualDelta": 54
        })
    }

    #[test]
    fn proven_receipt_passes() {
        assert!(validate_receipt_value(&proven_receipt()).is_empty());
    }

    #[test]
    fn missing_required_fields_caught() {
        let receipt = json!({ "verdict": "PASS" });
        let errs = validate_receipt_value(&receipt);
        assert!(errs
            .iter()
            .any(|e| e.contains("missing field 'output_hash'")));
        assert!(errs.iter().any(|e| e.contains("missing field 'run_id'")));
        assert!(errs.iter().any(|e| e.contains("missing field 'timestamp'")));
        assert!(errs.iter().any(|e| e.contains("missing field 'signature'")));
    }

    #[test]
    fn fail_verdict_rejected() {
        let mut r = proven_receipt();
        r["verdict"] = json!("FAIL");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("verdict is 'FAIL'")));
    }

    #[test]
    fn unknown_hash_prefix_rejected() {
        let mut r = proven_receipt();
        r["output_hash"] = json!("md5:abc123");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("unknown algorithm prefix")));
    }

    #[test]
    fn blake3_hash_accepted() {
        let mut r = proven_receipt();
        r["output_hash"] = json!("blake3:deadbeefdeadbeef");
        assert!(validate_receipt_value(&r).is_empty());
    }

    #[test]
    fn empty_run_id_rejected() {
        let mut r = proven_receipt();
        r["run_id"] = json!("");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("run_id is empty")));
    }

    #[test]
    fn visual_delta_below_threshold_rejected() {
        let mut r = proven_receipt();
        r["visualDelta"] = json!(5u64);
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("visualDelta=5 < 20")));
    }

    #[test]
    fn visual_delta_at_threshold_passes() {
        let mut r = proven_receipt();
        r["visualDelta"] = json!(20u64);
        assert!(validate_receipt_value(&r).is_empty());
    }

    #[test]
    fn receipt_without_visual_delta_passes() {
        // visualDelta is optional — not all receipts include it
        let r = json!({
            "verdict": "PASS",
            "output_hash": "sha256:abc123",
            "run_id": "run-1",
            "timestamp": "2026-01-01T00:00:00Z",
            "signature": "sig"
        });
        assert!(validate_receipt_value(&r).is_empty());
    }
}
