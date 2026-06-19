//! Receipt validation commands
//!
//! Understands two receipt formats:
//!   - OCEL cook receipt (from Html5PackageReport::write_receipt):
//!       { verdict, receipt_hash, is_real_package, wasm_mb, archive_dir, verified_at_unix_secs }
//!   - Playwright session receipt (from game-loop.spec.ts):
//!       { verdict, output_hash, run_id, timestamp, signature, [visualDelta] }

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

/// Detected receipt format — determines which validation rules apply.
#[derive(Debug, PartialEq)]
enum ReceiptKind {
    /// Html5PackageReport::write_receipt() — cook pipeline receipt
    OcelCook,
    /// Playwright game-loop-receipt.json — browser session receipt
    PlaywrightSession,
    /// Unknown format — require the minimal set of fields
    Unknown,
}

fn detect_kind(val: &Value) -> ReceiptKind {
    if val.get("receipt_hash").is_some() || val.get("is_real_package").is_some() {
        ReceiptKind::OcelCook
    } else if val.get("output_hash").is_some() || val.get("run_id").is_some() {
        ReceiptKind::PlaywrightSession
    } else {
        ReceiptKind::Unknown
    }
}

/// Pure validation logic extracted for unit testing.
/// Returns a list of validation error strings; empty = valid.
fn validate_receipt_value(val: &Value) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let kind = detect_kind(val);

    // Verdict is required in all formats
    match val.get("verdict").and_then(|v| v.as_str()) {
        None => errors.push("missing field 'verdict'".into()),
        Some(v) if v != "PASS" => errors.push(format!("verdict is '{v}', expected 'PASS'")),
        _ => {}
    }

    match kind {
        ReceiptKind::OcelCook => {
            // Hash field used by Html5PackageReport::write_receipt
            match val.get("receipt_hash").and_then(|v| v.as_str()) {
                None => {
                    // Older cook receipts may use "output_hash" — accept either
                    if val.get("output_hash").is_none() {
                        errors.push("missing field 'receipt_hash' (or 'output_hash')".into());
                    }
                }
                Some(h) if h.is_empty() => errors.push("receipt_hash is empty".into()),
                _ => {}
            }

            if val.get("is_real_package").is_none() && val.get("wasm_mb").is_none() {
                errors.push("missing field 'is_real_package' — not a valid cook receipt".into());
            }

            if let Some(false) = val.get("is_real_package").and_then(|v| v.as_bool()) {
                errors.push("is_real_package=false — WASM is a stub".into());
            }

            if let Some(mb) = val.get("wasm_mb").and_then(|v| v.as_f64()) {
                if mb < 10.0 {
                    errors.push(format!("wasm_mb={mb:.1} < 10 MB — likely a stub"));
                }
            }

            // engine_source must be "rocket_cli" for Rust-generated receipts
            if let Some(src) = val.get("engine_source").and_then(|v| v.as_str()) {
                if src == "synthetic" {
                    errors.push("engine_source='synthetic' — not a real cook receipt".into());
                }
            }
        }

        ReceiptKind::PlaywrightSession => {
            for field in &["output_hash", "run_id", "timestamp", "signature"] {
                if val.get(field).is_none() {
                    errors.push(format!("missing field '{field}'"));
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

            // Reject synthetic receipts from CI — they don't prove real UE4
            if let Some(src) = val.get("engine_source").and_then(|v| v.as_str()) {
                if src == "synthetic" {
                    errors.push(
                        "engine_source='synthetic' — this receipt was generated without real UE4 \
                         rendering. Run `rocket html5 e2e` against a real package to get a PASS."
                            .into(),
                    );
                }
            }
        }

        ReceiptKind::Unknown => {
            // Minimal: just verdict (already checked above)
            if val.get("receipt_hash").is_none() && val.get("output_hash").is_none() {
                errors.push("missing hash field ('receipt_hash' or 'output_hash')".into());
            }
        }
    }

    errors
}

fn do_validate_receipt(file: String) -> Result<Value> {
    use std::fs;
    let raw = fs::read_to_string(&file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("cannot read {file}: {e}"))
    })?;
    let val: Value = serde_json::from_str(&raw).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("invalid JSON in {file}: {e}"))
    })?;

    let kind = detect_kind(&val);
    let errors = validate_receipt_value(&val);

    if errors.is_empty() {
        let hash = val
            .get("receipt_hash")
            .or_else(|| val.get("output_hash"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let kind_label = match kind {
            ReceiptKind::OcelCook => "cook",
            ReceiptKind::PlaywrightSession => "session",
            ReceiptKind::Unknown => "unknown",
        };
        println!("PASS  kind={kind_label}  hash={hash}  file={file}");
        Ok(serde_json::json!({
            "status": "pass",
            "file": file,
            "kind": kind_label,
            "hash": hash,
        }))
    } else {
        Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "FAIL {file}: {}",
            errors.join("; ")
        )))
    }
}

/// Validate a receipt JSON has required fields and a passing verdict.
///
/// Accepts both cook receipts (from `rocket html5 verify`) and Playwright
/// session receipts (from `rocket html5 e2e`). Rejects synthetic receipts
/// that were generated without real UE4 rendering.
///
/// # Arguments
/// * `file` - Path to the receipt JSON file
#[verb("validate", "receipt")]
fn validate_receipt(file: String) -> Result<Value> {
    do_validate_receipt(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cook_receipt_pass() -> Value {
        json!({
            "verdict": "PASS",
            "is_real_package": true,
            "receipt_hash": "1a2b3c4d5e6f7a8b",
            "wasm_mb": 175.4,
            "archive_dir": "/tmp/brm-html5-archive/HTML5",
            "verified_at_unix_secs": 1718754000u64,
            "engine_source": "rocket_cli",
        })
    }

    fn session_receipt_pass() -> Value {
        json!({
            "verdict": "PASS",
            "output_hash": "sha256:a737628e84f38eec9ef91495d6fb1522710929f4a5e31496f12c0f1b5ac76f10",
            "run_id": "tps-dflss-1781855287977",
            "timestamp": "2026-06-17T00:00:00Z",
            "signature": "deb3010a",
            "visualDelta": 54,
            "engine_source": "real_ue4",
        })
    }

    // ── format detection ──────────────────────────────────────────────────────

    #[test]
    fn detects_cook_receipt_by_receipt_hash() {
        assert_eq!(detect_kind(&cook_receipt_pass()), ReceiptKind::OcelCook);
    }

    #[test]
    fn detects_session_receipt_by_output_hash() {
        assert_eq!(detect_kind(&session_receipt_pass()), ReceiptKind::PlaywrightSession);
    }

    #[test]
    fn detects_unknown_when_no_hash_field() {
        assert_eq!(detect_kind(&json!({ "verdict": "PASS" })), ReceiptKind::Unknown);
    }

    // ── cook receipt ──────────────────────────────────────────────────────────

    #[test]
    fn cook_receipt_pass_validates() {
        assert!(validate_receipt_value(&cook_receipt_pass()).is_empty());
    }

    #[test]
    fn cook_receipt_stub_wasm_rejected() {
        let mut r = cook_receipt_pass();
        r["is_real_package"] = json!(false);
        r["wasm_mb"] = json!(0.004);
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("is_real_package=false")));
        assert!(errs.iter().any(|e| e.contains("wasm_mb")));
    }

    #[test]
    fn cook_receipt_small_wasm_rejected() {
        let mut r = cook_receipt_pass();
        r["wasm_mb"] = json!(0.1);
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("wasm_mb=0.1 < 10 MB")));
    }

    #[test]
    fn cook_receipt_fail_verdict_rejected() {
        let mut r = cook_receipt_pass();
        r["verdict"] = json!("FAIL");
        assert!(!validate_receipt_value(&r).is_empty());
    }

    #[test]
    fn cook_receipt_synthetic_engine_source_rejected() {
        let mut r = cook_receipt_pass();
        r["engine_source"] = json!("synthetic");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("synthetic")));
    }

    #[test]
    fn cook_receipt_missing_is_real_package_rejected() {
        let mut r = cook_receipt_pass();
        r.as_object_mut().unwrap().remove("is_real_package");
        r.as_object_mut().unwrap().remove("wasm_mb");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("is_real_package")));
    }

    // ── session (Playwright) receipt ──────────────────────────────────────────

    #[test]
    fn session_receipt_pass_validates() {
        assert!(validate_receipt_value(&session_receipt_pass()).is_empty());
    }

    #[test]
    fn session_receipt_synthetic_engine_source_rejected() {
        let mut r = session_receipt_pass();
        r["engine_source"] = json!("synthetic");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("synthetic")));
    }

    #[test]
    fn session_receipt_missing_fields_caught() {
        let r = json!({ "verdict": "PASS", "output_hash": "sha256:abc" });
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("missing field 'run_id'")));
        assert!(errs.iter().any(|e| e.contains("missing field 'timestamp'")));
    }

    #[test]
    fn session_receipt_unknown_hash_prefix_rejected() {
        let mut r = session_receipt_pass();
        r["output_hash"] = json!("md5:abc123");
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("unknown algorithm prefix")));
    }

    #[test]
    fn session_receipt_visual_delta_below_threshold_rejected() {
        let mut r = session_receipt_pass();
        r["visualDelta"] = json!(5u64);
        let errs = validate_receipt_value(&r);
        assert!(errs.iter().any(|e| e.contains("visualDelta=5 < 20")));
    }

    #[test]
    fn session_receipt_without_visual_delta_passes() {
        let r = json!({
            "verdict": "PASS",
            "output_hash": "blake3:abc123",
            "run_id": "run-1",
            "timestamp": "2026-01-01T00:00:00Z",
            "signature": "sig",
            "engine_source": "real_ue4",
        });
        assert!(validate_receipt_value(&r).is_empty());
    }

    #[test]
    fn blake3_hash_accepted() {
        let mut r = session_receipt_pass();
        r["output_hash"] = json!("blake3:deadbeefdeadbeef");
        assert!(validate_receipt_value(&r).is_empty());
    }

    // ── unknown format ────────────────────────────────────────────────────────

    #[test]
    fn unknown_format_requires_hash_field() {
        let errs = validate_receipt_value(&json!({ "verdict": "PASS" }));
        assert!(errs.iter().any(|e| e.contains("missing hash field")));
    }
}
