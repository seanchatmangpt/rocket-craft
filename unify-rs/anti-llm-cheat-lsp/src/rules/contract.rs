use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        if o.kind != "contract_schism" {
            continue;
        }

        match o.construct.as_str() {
            // CONTRACT-001: zero vocabulary overlap between impl and oracle test
            "contract_vocab_divergence" => {
                diags.push(AntiLlmDiagnostic {
                    code: "ANTI-LLM-CONTRACT-001".to_string(),
                    category: "contract".to_string(),
                    file_path: o.file_path.clone(),
                    line: o.line,
                    column: o.column,
                    message: o.message.clone(),
                    forbidden_implication: "TestVocab => ImplVocab".to_string(),
                    blocking: true,
                    required_correction: "Implementation and oracle tests must share function vocabulary. Divergent vocabularies indicate the tests were rewritten to match the implementation (A9 contract schism) rather than validating an independent spec.".to_string(),
                    required_next_proof: "Oracle tests predate or are independently authored from the implementation; CI diff gate confirms no commit modifies both impl and oracle simultaneously.".to_string(),
                });
            }

            // CONTRACT-002: function shadow override
            "contract_fn_shadow" => {
                diags.push(AntiLlmDiagnostic {
                    code: "ANTI-LLM-CONTRACT-002".to_string(),
                    category: "contract".to_string(),
                    file_path: o.file_path.clone(),
                    line: o.line,
                    column: o.column,
                    message: o.message.clone(),
                    forbidden_implication: "TestFnDef => ImplFnDef".to_string(),
                    blocking: true,
                    required_correction: "Non-trivial function names must not appear in both implementation and test source for the same module. Shadow definitions indicate a test rewriting the production API to match its own vocabulary.".to_string(),
                    required_next_proof: "Remove duplicate definition; oracle test calls through the standard public API without redefining it.".to_string(),
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

    fn obs(construct: &str) -> Observation {
        Observation {
            file_path: "src/breeds/alpha/lib.rs".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "contract_schism".into(),
            construct: construct.into(), context: String::new(), message: "test".into(),
        }
    }

    #[test]
    fn empty_returns_no_diags() { assert!(evaluate(&[]).is_empty()); }

    #[test]
    fn contract_vocab_divergence_triggers_contract_001() {
        let d = evaluate(&[obs("contract_vocab_divergence")]);
        assert_eq!(d[0].code, "ANTI-LLM-CONTRACT-001");
        assert!(d[0].blocking);
    }

    #[test]
    fn contract_fn_shadow_triggers_contract_002() {
        let d = evaluate(&[obs("contract_fn_shadow")]);
        assert_eq!(d[0].code, "ANTI-LLM-CONTRACT-002");
        assert!(d[0].blocking);
    }

    #[test]
    fn unknown_contract_construct_produces_no_diag() {
        assert!(evaluate(&[obs("something_else")]).is_empty());
    }

    #[test]
    fn non_contract_schism_kind_is_ignored() {
        let mut o = obs("contract_vocab_divergence");
        o.kind = "not_schism".into();
        assert!(evaluate(&[o]).is_empty());
    }
}
