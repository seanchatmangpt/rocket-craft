use crate::observations::Observation;
use serde_json::Value;

pub fn parse_json_rpc_transcript(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();

    // Transcripts are usually JSONL format
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            // Check for initialize request and verify client capabilities
            if val.get("method").and_then(|m| m.as_str()) == Some("initialize") {
                if let Some(params) = val.get("params") {
                    let mut has_3_18_capabilities = false;

                    // Check if client capabilities advertise 3.18 inlineCompletion or textDocumentContent
                    if let Some(text_doc) = params
                        .get("capabilities")
                        .and_then(|c| c.get("textDocument"))
                    {
                        if text_doc.get("inlineCompletion").is_some()
                            || text_doc.get("foldingRange").is_some()
                        {
                            has_3_18_capabilities = true;
                        }
                    }

                    if !has_3_18_capabilities {
                        obs.push(Observation {
                            file_path: filepath.to_string(),
                            start_byte: 0,
                            end_byte: 0,
                            line: line_idx + 1,
                            column: 1,
                            kind: "json_rpc".to_string(),
                            construct: "initialize without 3.18 caps".to_string(),
                            context: trimmed.to_string(),
                            message: "LSP 3.18 initialize transcript lacks advertised client capabilities".to_string(),
                        });
                    }
                }
            }
        }
    }

    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_init(with_inline_completion: bool, with_folding_range: bool) -> String {
        let td = if with_inline_completion && with_folding_range {
            r#"{"inlineCompletion":{},"foldingRange":{}}"#
        } else if with_inline_completion {
            r#"{"inlineCompletion":{}}"#
        } else if with_folding_range {
            r#"{"foldingRange":{}}"#
        } else {
            r#"{"hover":{}}"#
        };
        format!(
            r#"{{"method":"initialize","params":{{"capabilities":{{"textDocument":{}}}}}}}"#,
            td
        )
    }

    #[test]
    fn empty_content_produces_no_obs() {
        assert!(parse_json_rpc_transcript("t.jsonl", "").is_empty());
    }

    #[test]
    fn blank_lines_are_skipped() {
        let content = "\n   \n";
        assert!(parse_json_rpc_transcript("t.jsonl", content).is_empty());
    }

    #[test]
    fn initialize_without_3_18_caps_produces_obs() {
        let line = make_init(false, false);
        let obs = parse_json_rpc_transcript("t.jsonl", &line);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].construct, "initialize without 3.18 caps");
        assert_eq!(obs[0].line, 1);
    }

    #[test]
    fn initialize_with_inline_completion_is_clean() {
        let line = make_init(true, false);
        assert!(parse_json_rpc_transcript("t.jsonl", &line).is_empty());
    }

    #[test]
    fn initialize_with_folding_range_is_clean() {
        let line = make_init(false, true);
        assert!(parse_json_rpc_transcript("t.jsonl", &line).is_empty());
    }

    #[test]
    fn non_initialize_method_is_ignored() {
        let line = r#"{"method":"textDocument/hover","params":{}}"#;
        assert!(parse_json_rpc_transcript("t.jsonl", line).is_empty());
    }

    #[test]
    fn multiple_init_lines_each_checked() {
        let bad = make_init(false, false);
        let good = make_init(true, false);
        let content = format!("{}\n{}", bad, good);
        let obs = parse_json_rpc_transcript("t.jsonl", &content);
        // only the bad line produces an obs
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].line, 1);
    }
}
