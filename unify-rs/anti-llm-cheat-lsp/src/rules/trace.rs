use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

fn is_test_path(path: &str) -> bool {
    path.contains("tests/")
        || path.ends_with("_test.rs")
        || path.contains("/test/")
        || path.contains("fixtures/")
}

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        match o.construct.as_str() {
            // TRACE-001: inference_trace.push with constant string literal
            "trace_constant_push" if !is_test_path(&o.file_path) => {
                diags.push(AntiLlmDiagnostic {
                    code: "ANTI-LLM-TRACE-001".to_string(),
                    category: "trace".to_string(),
                    file_path: o.file_path.clone(),
                    line: o.line,
                    column: o.column,
                    message: format!(
                        "inference_trace.push() with constant string literal in '{}' — fabricated trace evidence.",
                        o.context
                    ),
                    forbidden_implication: "TraceEntry => ComputedEvidence".to_string(),
                    blocking: true,
                    required_correction: "Replace constant string trace entries with dynamically constructed messages that include computed values (e.g., rule IDs, confidence scores, matched predicates). Static strings prove nothing.".to_string(),
                    required_next_proof: "Trace entries include computed values; step count equals algorithm-derived structural count (e.g., rule firings, decomposition depth).".to_string(),
                });
            }

            // TRACE-002: hardcoded trace length assertion
            "trace_len_magic_assert" if !is_test_path(&o.file_path) => {
                diags.push(AntiLlmDiagnostic {
                    code: "ANTI-LLM-TRACE-002".to_string(),
                    category: "trace".to_string(),
                    file_path: o.file_path.clone(),
                    line: o.line,
                    column: o.column,
                    message: format!(
                        "assert!(trace.len() == N) with literal integer in '{}' — hardcoded trace count.",
                        o.context
                    ),
                    forbidden_implication: "LenAssertion => DynamicTrace".to_string(),
                    blocking: true,
                    required_correction: "Derive expected step count from algorithm structure (e.g., rule count, composition table size) rather than embedding a magic literal. The count must vary with algorithm parameters.".to_string(),
                    required_next_proof: "Expected step count is computed from the input; hidden oracle with different parameters produces different step count.".to_string(),
                });
            }

            // TRACE-003: format!() with no {} placeholders in trace push
            "trace_static_format" if !is_test_path(&o.file_path) => {
                diags.push(AntiLlmDiagnostic {
                    code: "ANTI-LLM-TRACE-003".to_string(),
                    category: "trace".to_string(),
                    file_path: o.file_path.clone(),
                    line: o.line,
                    column: o.column,
                    message: format!(
                        "format!() with no {{}} interpolation in trace push in '{}' — static trace masquerading as computed.",
                        o.context
                    ),
                    forbidden_implication: "FormatCall => DynamicContent".to_string(),
                    blocking: true,
                    required_correction: "format!() calls in trace entries must inject computed values via {} placeholders. A format! with no interpolation is a string literal with extra ceremony.".to_string(),
                    required_next_proof: "All trace format strings contain at least one {} with a computed value (rule id, score, predicate, etc.).".to_string(),
                });
            }

            // TRACE-004: uniform trace push across match arms
            "trace_uniform_arms" if !is_test_path(&o.file_path) => {
                diags.push(AntiLlmDiagnostic {
                    code: "ANTI-LLM-TRACE-004".to_string(),
                    category: "trace".to_string(),
                    file_path: o.file_path.clone(),
                    line: o.line,
                    column: o.column,
                    message: format!(
                        "Identical trace entries across multiple match arms in '{}' — trace does not discriminate between cases.",
                        o.context
                    ),
                    forbidden_implication: "UniformTrace => CaseDiscrimination".to_string(),
                    blocking: true,
                    required_correction: "Each match arm must produce a trace entry that reflects the specific case matched (rule applied, conclusion drawn, etc.). Identical entries across arms mean the trace carries no algorithmic information.".to_string(),
                    required_next_proof: "Trace entries vary across match arms; inspector can reconstruct the execution path from the trace alone.".to_string(),
                });
            }

            _ => {}
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(file: &str, construct: &str) -> Observation {
        Observation {
            file_path: file.into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "ast_node".into(),
            construct: construct.into(), context: "ctx".into(), message: String::new(),
        }
    }

    #[test]
    fn empty_obs_returns_no_diags() {
        assert!(evaluate(&[]).is_empty());
    }

    #[test]
    fn trace_constant_push_in_prod_triggers_trace_001() {
        let diags = evaluate(&[obs("src/lib.rs", "trace_constant_push")]);
        assert_eq!(diags[0].code, "ANTI-LLM-TRACE-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn trace_constant_push_in_test_is_allowed() {
        let diags = evaluate(&[obs("tests/foo.rs", "trace_constant_push")]);
        assert!(diags.iter().all(|d| d.code != "ANTI-LLM-TRACE-001"));
    }

    #[test]
    fn trace_len_magic_assert_triggers_trace_002() {
        let diags = evaluate(&[obs("src/lib.rs", "trace_len_magic_assert")]);
        assert_eq!(diags[0].code, "ANTI-LLM-TRACE-002");
        assert!(diags[0].blocking);
    }

    #[test]
    fn trace_static_format_triggers_trace_003() {
        let diags = evaluate(&[obs("src/lib.rs", "trace_static_format")]);
        assert_eq!(diags[0].code, "ANTI-LLM-TRACE-003");
    }

    #[test]
    fn trace_rules_exempt_test_paths() {
        for construct in &["trace_len_magic_assert", "trace_static_format"] {
            let diags = evaluate(&[obs("tests/bar.rs", construct)]);
            assert!(diags.is_empty(), "{construct} in tests/ must not fire");
        }
    }

    #[test]
    fn unknown_construct_produces_no_diag() {
        let diags = evaluate(&[obs("src/lib.rs", "some_other_construct")]);
        assert!(diags.is_empty());
    }
}
