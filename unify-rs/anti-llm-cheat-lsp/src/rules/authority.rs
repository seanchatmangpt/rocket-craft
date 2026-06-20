use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // Check for fake CLAP abstraction
        if o.construct == "CLAP"
            || o.context.contains("CLAP authority")
            || o.context.contains("CLAP validation")
            || o.context.contains("CLAP command")
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-AUTH-002".to_string(),
                category: "authority".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message:
                    "Fake CLAP abstraction found. The actual component name is clap-noun-verb."
                        .to_string(),
                forbidden_implication: "Elegant abstraction => existing authority".to_string(),
                blocking: true,
                required_correction:
                    "Replace fake CLAP concepts with the concrete clap-noun-verb component."
                        .to_string(),
                required_next_proof: "Verify command admission via clap-noun-verb.".to_string(),
            });
        }

        // Check for string-shaped command treated as admitted command
        if o.construct == "string_command" || o.message.contains("String-shaped command") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-AUTH-004".to_string(),
                category: "authority".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "String-shaped command treated as admitted command.".to_string(),
                forbidden_implication: "StringShape(command) => command admission".to_string(),
                blocking: true,
                required_correction: "Avoid raw string checking for command execution; route only via noun/verb admission registry.".to_string(),
                required_next_proof: "Route command through clap-noun-verb registry.".to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(construct: &str, context: &str) -> Observation {
        Observation {
            file_path: "src/lib.rs".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "authority_smell".into(),
            construct: construct.into(), context: context.into(), message: String::new(),
        }
    }

    #[test]
    fn empty_returns_no_diags() { assert!(evaluate(&[]).is_empty()); }

    #[test]
    fn clap_construct_triggers_auth_002() {
        let d = evaluate(&[obs("CLAP", "")]);
        assert_eq!(d[0].code, "ANTI-LLM-AUTH-002");
        assert!(d[0].blocking);
    }

    #[test]
    fn clap_authority_in_context_triggers_auth_002() {
        let d = evaluate(&[obs("anything", "CLAP authority check")]);
        assert_eq!(d[0].code, "ANTI-LLM-AUTH-002");
    }

    #[test]
    fn string_command_construct_triggers_auth_004() {
        let d = evaluate(&[obs("string_command", "")]);
        assert_eq!(d[0].code, "ANTI-LLM-AUTH-004");
        assert!(d[0].blocking);
    }

    #[test]
    fn unknown_construct_produces_no_diag() {
        assert!(evaluate(&[obs("real_noun_verb_command", "")]).is_empty());
    }
}
