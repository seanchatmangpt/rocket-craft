use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // Debug diagnostic names found
        if o.construct == "CLAP-DEBUG" || o.construct == "CLAP-DEBUG-PATH" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-001".to_string(),
                category: "strange-code".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Debug diagnostic name found in admissible path.".to_string(),
                forbidden_implication: "Debug scaffold => law diagnostic".to_string(),
                blocking: true,
                required_correction:
                    "Remove temporary/debug diagnostics from production code paths.".to_string(),
                required_next_proof: "Verify all diagnostics are production-ready.".to_string(),
            });
        }

        // Diagnostic leaks raw content
        if o.construct == "Content was:" || o.message.contains("leaks raw content") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-002".to_string(),
                category: "strange-code".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message:
                    "Diagnostic leaks raw file content, which could leak secrets or private data."
                        .to_string(),
                forbidden_implication: "Raw content dump => useful diagnostic".to_string(),
                blocking: true,
                required_correction:
                    "Obfuscate or summarize content in diagnostics instead of printing raw content."
                        .to_string(),
                required_next_proof: "Check diagnostic message serialization.".to_string(),
            });
        }

        // Diagnostic leaks raw path
        if o.construct == "Path was:" || o.message.contains("leaks raw path") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-003".to_string(),
                category: "strange-code".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Diagnostic leaks raw path, violating environment isolation rules."
                    .to_string(),
                forbidden_implication: "Raw path dump => law diagnostic".to_string(),
                blocking: true,
                required_correction: "Output relative or sanitized paths in diagnostic details."
                    .to_string(),
                required_next_proof: "Check path scrubbing function in diagnostic emitter."
                    .to_string(),
            });
        }

        // Substring check used as law
        if o.construct.starts_with("content.contains")
            || o.construct.starts_with("path.ends_with")
            || o.construct.starts_with("path_str.contains")
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-007".to_string(),
                category: "strange-code".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Substring check used as law (e.g. searching 'customization-map.json' or 'TODO').".to_string(),
                forbidden_implication: "SubstringMatch => Authority".to_string(),
                blocking: true,
                required_correction: "Use structural AST or file metadata parsing instead of simple string searches for policy checks.".to_string(),
                required_next_proof: "Verify utilizing tree-sitter or JSON-TOML deserializers.".to_string(),
            });
        }

        // Stub macros and stub functions detected by rust_tree_sitter
        if o.kind == "rust_stub" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-RUST-001".to_string(),
                category: "rust-stub".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: o.message.clone(),
                forbidden_implication: "Stub macro/fn => Shipped implementation".to_string(),
                blocking: true,
                required_correction: "Replace stub with a real implementation.".to_string(),
                required_next_proof: "Confirm the function body performs real work.".to_string(),
            });
        }

        // Debug artifacts (println!, eprintln!, dbg!) in production code
        if o.kind == "rust_debug_artifact" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-RUST-002".to_string(),
                category: "rust-debug".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: o.message.clone(),
                forbidden_implication: "Debug macro => Production artifact".to_string(),
                blocking: false,
                required_correction: "Remove debug print macros from production code paths."
                    .to_string(),
                required_next_proof: "Confirm no debug output in production build.".to_string(),
            });
        }

        // #[allow(...)] suppression attributes
        if o.kind == "rust_suppression" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-RUST-003".to_string(),
                category: "rust-suppression".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: o.message.clone(),
                forbidden_implication: "allow(…) suppression => Compiler warning silenced"
                    .to_string(),
                blocking: false,
                required_correction: "Fix the underlying warning instead of suppressing it."
                    .to_string(),
                required_next_proof: "Remove the suppression attribute and confirm clean build."
                    .to_string(),
            });
        }

        // TODO/FIXME/HACK/STUB comments
        if o.kind == "rust_todo_comment" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-RUST-004".to_string(),
                category: "rust-todo".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: o.message.clone(),
                forbidden_implication: "TODO comment => Finished work".to_string(),
                blocking: false,
                required_correction: "Resolve the outstanding TODO before shipping.".to_string(),
                required_next_proof: "Confirm no TODO/FIXME markers remain in this path."
                    .to_string(),
            });
        }
    }

    diags
}
