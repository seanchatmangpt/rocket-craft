use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // assert_contains_string: .contains("literal") — Display/string cheat
        // assert_contains: unknown receiver type — flag conservatively
        // assert_contains_structural: .contains(&EnumVariant) — acceptable, no diagnostic
        if o.construct == "assert_contains_string" || o.construct == "assert_contains" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-TEST-001".to_string(),
                category: "test".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "String assertion in test file. Tests must verify structural \
                    properties instead of substring search on Display outputs."
                    .to_string(),
                forbidden_implication: "TestStdout => Receipt".to_string(),
                blocking: true,
                required_correction:
                    "Replace assert!(x.to_string().contains(\"Name\")) with \
                    assert_eq!(x, Enum::Variant). For Vec membership use \
                    assert!(vec.contains(&Enum::Variant)) — the scanner detects that as structural."
                        .to_string(),
                required_next_proof: "Run cargo test to verify structural assertions hold."
                    .to_string(),
            });
        }

        // assert_contains_structural: Vec::contains(&EnumVariant) — no diagnostic.
        // The scanner emits this construct so call sites can see it was classified,
        // but no CHEAT diagnostic fires.

        if o.construct == "negative_control_reference" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-TEST-003".to_string(),
                category: "test".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Standard test references or verifies negative controls directly. \
                    This is prohibited except in authorized dogfooding tests."
                    .to_string(),
                forbidden_implication: "Positive case passes => law holds".to_string(),
                blocking: true,
                required_correction:
                    "Remove negative control directory or fixture references from standard tests."
                        .to_string(),
                required_next_proof:
                    "Run test suite to verify tests use mocked or isolated test fixtures."
                        .to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(construct: &str) -> Observation {
        Observation {
            file_path: "tests/foo.rs".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "test_smell".into(),
            construct: construct.into(), context: String::new(), message: String::new(),
        }
    }

    #[test]
    fn empty_returns_no_diags() { assert!(evaluate(&[]).is_empty()); }

    #[test]
    fn assert_contains_string_triggers_test_001() {
        let d = evaluate(&[obs("assert_contains_string")]);
        assert_eq!(d[0].code, "ANTI-LLM-TEST-001");
        assert!(d[0].blocking);
    }

    #[test]
    fn assert_contains_triggers_test_001() {
        let d = evaluate(&[obs("assert_contains")]);
        assert_eq!(d[0].code, "ANTI-LLM-TEST-001");
    }

    #[test]
    fn assert_contains_structural_produces_no_diag() {
        assert!(evaluate(&[obs("assert_contains_structural")]).is_empty());
    }

    #[test]
    fn negative_control_reference_triggers_test_003() {
        let d = evaluate(&[obs("negative_control_reference")]);
        assert_eq!(d[0].code, "ANTI-LLM-TEST-003");
        assert!(d[0].blocking);
    }

    #[test]
    fn unknown_construct_produces_no_diag() {
        assert!(evaluate(&[obs("normal_assertion")]).is_empty());
    }
}
