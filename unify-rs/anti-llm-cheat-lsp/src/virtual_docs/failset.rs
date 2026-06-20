use crate::diagnostics::AntiLlmDiagnostic;

pub fn generate_failset_markdown(diags: &[AntiLlmDiagnostic]) -> String {
    let mut out = String::new();
    out.push_str("# Active Admissibility Failset\n\n");
    if diags.is_empty() {
        out.push_str(
            "Status: **REPORTED_CLEAN_WITH_RAW_SCAN**\n\nNo blocking failset items detected.\n",
        );
    } else {
        out.push_str("Status: **FAILSET_NONEMPTY**\n\n");
        out.push_str(
            "| Code | Category | Path | Line | Message | Forbidden Implication | Blocking |\n",
        );
        out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
        for d in diags {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                d.code,
                d.category,
                d.file_path,
                d.line,
                d.message,
                d.forbidden_implication,
                d.blocking
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diag(code: &str, line: usize, blocking: bool) -> AntiLlmDiagnostic {
        AntiLlmDiagnostic {
            code: code.to_string(),
            category: "refgraph".to_string(),
            file_path: "src/foo.rs".to_string(),
            line,
            column: 1,
            message: "test message".to_string(),
            forbidden_implication: "DependentSite => Witnessed".to_string(),
            blocking,
            required_correction: "".to_string(),
            required_next_proof: "".to_string(),
        }
    }

    #[test]
    fn empty_diags_produces_clean_status() {
        let out = generate_failset_markdown(&[]);
        assert!(out.contains("REPORTED_CLEAN_WITH_RAW_SCAN"));
        assert!(out.contains("No blocking failset items detected"));
    }

    #[test]
    fn nonempty_diags_produces_failset_status() {
        let diags = vec![make_diag("ANTI-LLM-001", 5, true)];
        let out = generate_failset_markdown(&diags);
        assert!(out.contains("FAILSET_NONEMPTY"));
    }

    #[test]
    fn diag_code_appears_in_table() {
        let diags = vec![make_diag("ANTI-LLM-REFGRAPH-001", 10, true)];
        let out = generate_failset_markdown(&diags);
        assert!(out.contains("ANTI-LLM-REFGRAPH-001"));
    }

    #[test]
    fn line_number_appears_in_table() {
        let diags = vec![make_diag("CODE", 42, false)];
        let out = generate_failset_markdown(&diags);
        assert!(out.contains("42"));
    }

    #[test]
    fn markdown_has_header_row() {
        let diags = vec![make_diag("CODE", 1, true)];
        let out = generate_failset_markdown(&diags);
        assert!(out.contains("| Code | Category |"));
    }
}
