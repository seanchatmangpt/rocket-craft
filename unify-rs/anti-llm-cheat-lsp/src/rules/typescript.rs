use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        if o.kind == "ts_smell" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-009".to_string(),
                category: "typescript-smell".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: format!("TypeScript code smell or stub detected: {}.", o.message),
                forbidden_implication: "TypeScriptStub => AdmissibleCode".to_string(),
                blocking: true,
                required_correction:
                    "Remove ts-ignore, eslint-disable, unsafe casting (as any), or TODO stubs."
                        .to_string(),
                required_next_proof: "Ensure typescript compiles strictly and uses explicit types."
                    .to_string(),
            });
        }

        if o.kind == "ts_claim" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-CLAIM-005".to_string(),
                category: "typescript-claim".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: format!("Forbidden victory/done claim: {}.", o.message),
                forbidden_implication: "StatusWord(CLAIMED) => Admitted".to_string(),
                blocking: true,
                required_correction: "Remove forbidden done/complete/victory words and replace with bounded status vocabulary.".to_string(),
                required_next_proof: "Admissibility audit checks TS surfaces.".to_string(),
            });
        }

        if o.kind == "ts_leak" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-CLAIM-006".to_string(),
                category: "typescript-leak".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: format!("Vocabulary or naming fence violation in TypeScript: {}.", o.message),
                forbidden_implication: "InternalTermLeak => PublicSurface".to_string(),
                blocking: true,
                required_correction: "Sanitize internal vocabulary terms (GALL, checkpoint, failset, etc.) or unauthorized naming (Nitro LSP) from TypeScript files.".to_string(),
                required_next_proof: "Verify all file paths and contents comply with the public-facing language boundary.".to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(kind: &str, msg: &str) -> Observation {
        Observation {
            file_path: "src/foo.ts".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: kind.into(),
            construct: "x".into(), context: String::new(), message: msg.into(),
        }
    }

    #[test]
    fn empty_returns_no_diags() { assert!(evaluate(&[]).is_empty()); }

    #[test]
    fn ts_smell_triggers_strange_009() {
        let d = evaluate(&[obs("ts_smell", "as any detected")]);
        assert_eq!(d[0].code, "ANTI-LLM-STRANGE-009");
        assert!(d[0].blocking);
    }

    #[test]
    fn ts_claim_triggers_claim_005() {
        let d = evaluate(&[obs("ts_claim", "DONE found")]);
        assert_eq!(d[0].code, "ANTI-LLM-CLAIM-005");
        assert!(d[0].blocking);
    }

    #[test]
    fn ts_leak_triggers_claim_006() {
        let d = evaluate(&[obs("ts_leak", "GALL found in variable name")]);
        assert_eq!(d[0].code, "ANTI-LLM-CLAIM-006");
        assert!(d[0].blocking);
    }

    #[test]
    fn unknown_kind_produces_no_diag() {
        assert!(evaluate(&[obs("unknown_kind", "")]).is_empty());
    }
}
