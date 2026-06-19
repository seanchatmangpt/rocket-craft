use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiLlmDiagnostic {
    pub code: String,
    pub category: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub forbidden_implication: String,
    pub blocking: bool,
    pub required_correction: String,
    pub required_next_proof: String,
}

impl AntiLlmDiagnostic {
    pub fn to_lsp(&self) -> Diagnostic {
        let start_pos = Position::new(
            (self.line.saturating_sub(1)) as u32,
            (self.column.saturating_sub(1)) as u32,
        );
        let end_pos = Position::new(
            (self.line.saturating_sub(1)) as u32,
            (self.column.saturating_sub(1) + 10) as u32,
        );
        let severity = if self.blocking {
            DiagnosticSeverity::ERROR
        } else {
            DiagnosticSeverity::WARNING
        };
        Diagnostic {
            range: Range::new(start_pos, end_pos),
            severity: Some(severity),
            code: Some(tower_lsp::lsp_types::NumberOrString::String(
                self.code.clone(),
            )),
            source: Some("anti-llm-cheat-lsp".to_string()),
            message: format!(
                "{}\nForbidden Implication: {}\nRequired Correction: {}\nRequired Next Proof: {}",
                self.message,
                self.forbidden_implication,
                self.required_correction,
                self.required_next_proof
            ),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::DiagnosticSeverity;

    fn make_diag(blocking: bool, line: usize, column: usize) -> AntiLlmDiagnostic {
        AntiLlmDiagnostic {
            code: "TEST-001".into(),
            category: "test".into(),
            file_path: "src/lib.rs".into(),
            line,
            column,
            message: "msg".into(),
            forbidden_implication: "fi".into(),
            blocking,
            required_correction: "rc".into(),
            required_next_proof: "rnp".into(),
        }
    }

    #[test]
    fn blocking_diag_maps_to_error_severity() {
        let lsp = make_diag(true, 3, 5).to_lsp();
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn non_blocking_diag_maps_to_warning_severity() {
        let lsp = make_diag(false, 1, 1).to_lsp();
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn to_lsp_line_is_zero_based() {
        let lsp = make_diag(true, 3, 1).to_lsp();
        // line 3 → row 2 (0-based)
        assert_eq!(lsp.range.start.line, 2);
    }

    #[test]
    fn to_lsp_column_is_zero_based() {
        let lsp = make_diag(true, 1, 5).to_lsp();
        assert_eq!(lsp.range.start.character, 4);
    }

    #[test]
    fn to_lsp_end_column_is_start_plus_10() {
        let lsp = make_diag(false, 1, 1).to_lsp();
        assert_eq!(lsp.range.end.character, lsp.range.start.character + 10);
    }

    #[test]
    fn to_lsp_message_contains_all_fields() {
        let d = make_diag(false, 1, 1);
        let lsp = d.to_lsp();
        assert!(lsp.message.contains("msg"));
        assert!(lsp.message.contains("fi"));
        assert!(lsp.message.contains("rc"));
        assert!(lsp.message.contains("rnp"));
    }

    #[test]
    fn to_lsp_source_is_anti_llm() {
        let lsp = make_diag(true, 1, 1).to_lsp();
        assert_eq!(lsp.source.as_deref(), Some("anti-llm-cheat-lsp"));
    }

    #[test]
    fn to_lsp_code_is_preserved() {
        let lsp = make_diag(false, 1, 1).to_lsp();
        match lsp.code {
            Some(tower_lsp::lsp_types::NumberOrString::String(s)) => assert_eq!(s, "TEST-001"),
            _ => panic!("expected string code"),
        }
    }

    #[test]
    fn to_lsp_saturation_prevents_underflow_on_line_1_col_1() {
        let lsp = make_diag(true, 1, 1).to_lsp();
        assert_eq!(lsp.range.start.line, 0);
        assert_eq!(lsp.range.start.character, 0);
    }
}
