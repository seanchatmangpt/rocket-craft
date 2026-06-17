use crate::diagnostic::{Diagnostic, DiagnosticSet, DiagnosticSeverity, Position, Range};
use anti_llm_cheat_lsp::config::AntiLlmConfig;
use anti_llm_cheat_lsp::diagnostics::AntiLlmDiagnostic;
use anti_llm_cheat_lsp::engine;

/// Converts anti-llm-cheat-lsp diagnostics into LSP `DiagnosticSet` entries.
pub struct AntiLlmGate {
    config: AntiLlmConfig,
}

impl AntiLlmGate {
    pub fn new() -> Self {
        Self {
            config: AntiLlmConfig::default(),
        }
    }

    pub fn with_config(config: AntiLlmConfig) -> Self {
        Self { config }
    }

    /// Scan a single file and return LSP diagnostics for it.
    pub fn scan_file_to_lsp(&self, file_path: &str) -> DiagnosticSet {
        let observations = engine::scan_file(file_path);
        let anti_diags =
            engine::evaluate_diagnostics_with_config(&observations, &self.config);
        self.build_set(anti_diags)
    }

    /// Scan a directory and return aggregated LSP diagnostics keyed by file URI.
    pub fn scan_directory_to_lsp(&self, dir_path: &str) -> DiagnosticSet {
        let observations = engine::scan_directory(dir_path);
        let anti_diags =
            engine::evaluate_diagnostics_with_config(&observations, &self.config);
        self.build_set(anti_diags)
    }

    fn build_set(&self, anti_diags: Vec<AntiLlmDiagnostic>) -> DiagnosticSet {
        let mut set = DiagnosticSet::new();
        for diag in anti_diags {
            let uri = path_to_uri(&diag.file_path);
            set.add(uri, to_lsp_diagnostic(&diag));
        }
        set
    }
}

impl Default for AntiLlmGate {
    fn default() -> Self {
        Self::new()
    }
}

fn to_lsp_diagnostic(diag: &AntiLlmDiagnostic) -> Diagnostic {
    let line = (diag.line.saturating_sub(1)) as u32;
    let character = (diag.column.saturating_sub(1)) as u32;
    Diagnostic {
        range: Range {
            start: Position { line, character },
            end: Position {
                line,
                character: character + 10,
            },
        },
        severity: if diag.blocking {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        },
        code: Some(diag.code.clone()),
        source: Some("anti-llm-cheat-lsp".to_string()),
        message: format!(
            "{}\nForbidden: {}\nCorrection: {}",
            diag.message, diag.forbidden_implication, diag.required_correction
        ),
    }
}

fn path_to_uri(file_path: &str) -> String {
    if file_path.starts_with('/') {
        format!("file://{}", file_path)
    } else {
        format!("file:///{}", file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_dir_produces_no_diagnostics() {
        let gate = AntiLlmGate::new();
        let set = gate.scan_directory_to_lsp("/nonexistent_dir_that_does_not_exist");
        assert_eq!(set.uri_count(), 0);
        assert_eq!(set.error_count(), 0);
    }

    #[test]
    fn blocking_diag_maps_to_error_severity() {
        use anti_llm_cheat_lsp::diagnostics::AntiLlmDiagnostic;
        let diag = AntiLlmDiagnostic {
            code: "ANTI-LLM-RUST-001".to_string(),
            category: "rust-stub".to_string(),
            file_path: "/src/lib.rs".to_string(),
            line: 10,
            column: 5,
            message: "stub detected".to_string(),
            forbidden_implication: "Stub => Real".to_string(),
            blocking: true,
            required_correction: "Replace stub".to_string(),
            required_next_proof: "Verify".to_string(),
        };
        let lsp = to_lsp_diagnostic(&diag);
        assert_eq!(lsp.severity, DiagnosticSeverity::Error);
        assert_eq!(lsp.range.start.line, 9);
        assert_eq!(lsp.range.start.character, 4);
    }

    #[test]
    fn non_blocking_diag_maps_to_warning_severity() {
        use anti_llm_cheat_lsp::diagnostics::AntiLlmDiagnostic;
        let diag = AntiLlmDiagnostic {
            code: "ANTI-LLM-RUST-002".to_string(),
            category: "rust-debug".to_string(),
            file_path: "/src/main.rs".to_string(),
            line: 5,
            column: 1,
            message: "debug macro".to_string(),
            forbidden_implication: "Debug => Prod".to_string(),
            blocking: false,
            required_correction: "Remove".to_string(),
            required_next_proof: "Check".to_string(),
        };
        let lsp = to_lsp_diagnostic(&diag);
        assert_eq!(lsp.severity, DiagnosticSeverity::Warning);
    }
}
