use crate::observations::Observation;
use serde_json::Value;

pub fn parse_receipt_json(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();

    if let Ok(val) = serde_json::from_str::<Value>(content) {
        // Enforce required fields
        let required_fields = [
            "digest",
            "digest_algorithm",
            "boundary",
            "checkpoint",
            "raw_command",
            "output_digest",
        ];
        for field in &required_fields {
            if val.get(field).is_none() {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: 1,
                    column: 1,
                    kind: "receipt_json".to_string(),
                    construct: format!("missing {}", field),
                    context: content.to_string(),
                    message: format!("Receipt file lacks required field '{}'", field),
                });
            }
        }

        // Enforce BLAKE3 for Gall receipts
        if let Some(alg) = val.get("digest_algorithm").and_then(|a| a.as_str()) {
            if alg != "BLAKE3" && alg != "SHA-256" {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: 1,
                    column: 1,
                    kind: "receipt_json".to_string(),
                    construct: "invalid digest_algorithm".to_string(),
                    context: content.to_string(),
                    message: format!(
                        "Receipt uses invalid digest algorithm '{}'; expected BLAKE3 or SHA-256",
                        alg
                    ),
                });
            }
        }
    } else {
        obs.push(Observation {
            file_path: filepath.to_string(),
            start_byte: 0,
            end_byte: 0,
            line: 1,
            column: 1,
            kind: "receipt_json".to_string(),
            construct: "invalid json".to_string(),
            context: content.to_string(),
            message: "Receipt file is not valid JSON".to_string(),
        });
    }

    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_receipt() -> &'static str {
        r#"{
            "digest": "abc123",
            "digest_algorithm": "BLAKE3",
            "boundary": "stage_4",
            "checkpoint": "cp_01",
            "raw_command": "cargo test",
            "output_digest": "def456"
        }"#
    }

    #[test]
    fn valid_receipt_produces_no_obs() {
        let obs = parse_receipt_json("receipt.json", valid_receipt());
        assert!(obs.is_empty(), "got: {:?}", obs.iter().map(|o| &o.construct).collect::<Vec<_>>());
    }

    #[test]
    fn invalid_json_produces_obs() {
        let obs = parse_receipt_json("receipt.json", "not-json");
        assert_eq!(obs[0].construct, "invalid json");
    }

    #[test]
    fn missing_digest_produces_obs() {
        let json = r#"{"digest_algorithm":"BLAKE3","boundary":"x","checkpoint":"y","raw_command":"z","output_digest":"w"}"#;
        let obs = parse_receipt_json("receipt.json", json);
        assert!(obs.iter().any(|o| o.construct == "missing digest"));
    }

    #[test]
    fn missing_multiple_fields_each_produce_obs() {
        let json = r#"{"digest": "x"}"#;
        let obs = parse_receipt_json("receipt.json", json);
        // 5 missing fields
        assert!(obs.len() >= 5, "expected at least 5 violations, got {}", obs.len());
    }

    #[test]
    fn sha256_algorithm_is_accepted() {
        let json = r#"{
            "digest":"x","digest_algorithm":"SHA-256",
            "boundary":"b","checkpoint":"c","raw_command":"r","output_digest":"d"
        }"#;
        let obs = parse_receipt_json("receipt.json", json);
        assert!(!obs.iter().any(|o| o.construct == "invalid digest_algorithm"));
    }

    #[test]
    fn unknown_algorithm_produces_obs() {
        let json = r#"{
            "digest":"x","digest_algorithm":"MD5",
            "boundary":"b","checkpoint":"c","raw_command":"r","output_digest":"d"
        }"#;
        let obs = parse_receipt_json("receipt.json", json);
        assert!(obs.iter().any(|o| o.construct == "invalid digest_algorithm"));
    }
}
