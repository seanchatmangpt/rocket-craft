use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // Direct file write in LSP authority path
        if o.construct == "std::fs::write"
            || o.construct == "tokio::fs::write"
            || o.construct == "File::create"
            || o.construct == "OpenOptions"
            || o.construct == "write_all"
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-MUT-001".to_string(),
                category: "mutation".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Direct file write or file creation found in LSP authority path. The server is read-only by default.".to_string(),
                forbidden_implication: "LSP observation => mutation authority".to_string(),
                blocking: true,
                required_correction: "Remove direct file write call. Route mutation requests via CodeAction to PackPlan intent instead.".to_string(),
                required_next_proof: "Verify with read-only permission checks.".to_string(),
            });
        }

        // WorkspaceEdit used as receipt binding
        if o.construct == "WorkspaceEdit"
            || o.message.contains("WorkspaceEdit used as receipt binding")
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-MUT-002".to_string(),
                category: "mutation".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "WorkspaceEdit used directly as receipt binding or mutation proof.".to_string(),
                forbidden_implication: "WorkspaceEdit => admitted receipt mutation".to_string(),
                blocking: true,
                required_correction: "WorkspaceEdit must represent a read-only template intent, not the final mutation receipt.".to_string(),
                required_next_proof: "Enforce MutationGate and sign receipts independently.".to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(construct: &str, msg: &str) -> Observation {
        Observation {
            file_path: "src/lib.rs".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "mutation_smell".into(),
            construct: construct.into(), context: String::new(), message: msg.into(),
        }
    }

    #[test]
    fn empty_returns_no_diags() { assert!(evaluate(&[]).is_empty()); }

    #[test]
    fn std_fs_write_triggers_mut_001() {
        let d = evaluate(&[obs("std::fs::write", "")]);
        assert_eq!(d[0].code, "ANTI-LLM-MUT-001");
        assert!(d[0].blocking);
    }

    #[test]
    fn file_create_triggers_mut_001() {
        let d = evaluate(&[obs("File::create", "")]);
        assert_eq!(d[0].code, "ANTI-LLM-MUT-001");
    }

    #[test]
    fn workspace_edit_construct_triggers_mut_002() {
        let d = evaluate(&[obs("WorkspaceEdit", "")]);
        assert_eq!(d[0].code, "ANTI-LLM-MUT-002");
        assert!(d[0].blocking);
    }

    #[test]
    fn workspace_edit_in_message_triggers_mut_002() {
        let d = evaluate(&[obs("something", "WorkspaceEdit used as receipt binding")]);
        assert_eq!(d[0].code, "ANTI-LLM-MUT-002");
    }

    #[test]
    fn unknown_construct_produces_no_diag() {
        assert!(evaluate(&[obs("safe_read_op", "")]).is_empty());
    }
}
