use lsp_types_max::{Diagnostic, DiagnosticSeverity, Position, Range, NumberOrString};
use std::collections::HashSet;

/// GGEN-MTLX-001: Material node specifies an input connection that is unbound.
pub const GGEN_MTLX_001: &str = "GGEN-MTLX-001";

#[derive(Clone)]
pub struct MtlxAnalyzer {
    source: String,
}

impl MtlxAnalyzer {
    pub fn new(content: &str) -> Self {
        Self { source: content.to_string() }
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();
        let mut defined_nodes: HashSet<String> = HashSet::new();
        let mut required_inputs: Vec<(String, u32)> = Vec::new();

        let mut create_diag = |line: u32, severity: DiagnosticSeverity, code: &str, message: String| {
            let len = self.source.lines().nth(line as usize).map(|l| l.len()).unwrap_or(0) as u32;
            Diagnostic {
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: len },
                },
                severity: Some(severity),
                code: Some(NumberOrString::String(code.to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message,
                ..Default::default()
            }
        };

        for (idx, line) in self.source.lines().enumerate() {
            let line_no = u32::try_from(idx).unwrap_or(0);
            let trimmed = line.trim();

            if trimmed.starts_with("<surfacematerial ") || trimmed.starts_with("<standard_surface ") || trimmed.starts_with("<image ") {
                if let Some(name_idx) = trimmed.find("name=\"") {
                    let start = name_idx + "name=\"".len();
                    if let Some(end) = trimmed[start..].find('"') {
                        let name = &trimmed[start..start + end];
                        defined_nodes.insert(name.to_string());
                    }
                }
            }

            if trimmed.starts_with("<input ") && trimmed.contains("nodename=\"") {
                if let Some(node_idx) = trimmed.find("nodename=\"") {
                    let start = node_idx + "nodename=\"".len();
                    if let Some(end) = trimmed[start..].find('"') {
                        let dep = &trimmed[start..start + end];
                        required_inputs.push((dep.to_string(), line_no));
                    }
                }
            }
        }

        for (dep, line_no) in required_inputs {
            if !defined_nodes.contains(&dep) {
                diags.push(create_diag(
                    line_no,
                    DiagnosticSeverity::ERROR,
                    GGEN_MTLX_001,
                    format!("GGEN-MTLX-001: unbound material input — nodename \"{}\" is not defined.", dep),
                ));
            }
        }

        diags
    }
}
