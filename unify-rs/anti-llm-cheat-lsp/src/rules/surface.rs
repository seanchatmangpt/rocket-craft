use crate::config::AntiLlmConfig;
use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation], config: &AntiLlmConfig) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        if o.construct.contains("tower-lsp") || o.construct.contains("tower_lsp") {
            // Downgrade to non-blocking for doc paths (historical references, archived tickets).
            let blocking = !config.surface_is_non_blocking(&o.file_path);
            let message = if blocking {
                "Plain tower-lsp found in codebase. All tower LSP hosts must migrate to lsp-max."
                    .to_string()
            } else {
                format!(
                    "Historical tower-lsp reference in documentation ({}). \
                    Non-blocking — update prose when convenient.",
                    o.file_path
                )
            };
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-SURFACE-001".to_string(),
                category: "surface".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message,
                forbidden_implication: "Pass(plain LSP) => Pass(LSP 3.18)".to_string(),
                blocking,
                required_correction: "Replace plain 'tower-lsp' dependency and use 'lsp-max'. \
                    For doc-only references, add the path prefix to anti-llm.toml \
                    [surface] non_blocking_path_prefixes."
                    .to_string(),
                required_next_proof: "Run cargo check / cargo test to verify lsp-max integration."
                    .to_string(),
            });
        }

        if o.construct == "PackObserver" || o.message.contains("observer dependency") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-SURFACE-003".to_string(),
                category: "surface".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Pack observer dependency treated as runtime authority.".to_string(),
                forbidden_implication: "Pack observes surface => runtime uses surface".to_string(),
                blocking: true,
                required_correction:
                    "Do not use PackObserver/static analyzer results as runtime authority."
                        .to_string(),
                required_next_proof: "Verify with active capability checks.".to_string(),
            });
        }

        if o.construct == "initialize without 3.18 caps" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-SURFACE-005".to_string(),
                category: "surface".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message:
                    "LSP 3.18 claimed but initialize capability transcript lacks 3.18 support."
                        .to_string(),
                forbidden_implication: "Basic LSP transcript => LSP 3.18".to_string(),
                blocking: true,
                required_correction: "Negotiate and log 3.18 inlineCompletion/foldingRange \
                    capability in initialize transcript."
                    .to_string(),
                required_next_proof: "Provide a client-to-server initialize handshake transcript."
                    .to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AntiLlmConfig, SurfaceConfig};
    use crate::observations::Observation;

    fn obs(file: &str, construct: &str) -> Observation {
        Observation {
            file_path: file.into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "surface_smell".into(),
            construct: construct.into(), context: String::new(), message: String::new(),
        }
    }

    fn default_config() -> AntiLlmConfig {
        AntiLlmConfig::default()
    }

    fn config_with_non_blocking_prefix(prefix: &str) -> AntiLlmConfig {
        AntiLlmConfig {
            surface: SurfaceConfig {
                non_blocking_path_prefixes: vec![prefix.into()],
            },
            ..Default::default()
        }
    }

    #[test]
    fn empty_obs_produces_no_diags() {
        assert!(evaluate(&[], &default_config()).is_empty());
    }

    #[test]
    fn tower_lsp_in_src_is_blocking() {
        let o = obs("src/lib.rs", "tower-lsp");
        let diags = evaluate(&[o], &default_config());
        assert_eq!(diags[0].code, "ANTI-LLM-SURFACE-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn tower_lsp_in_non_blocking_prefix_is_non_blocking() {
        let o = obs("docs/changelog.md", "tower-lsp");
        let cfg = config_with_non_blocking_prefix("docs/");
        let diags = evaluate(&[o], &cfg);
        assert_eq!(diags[0].code, "ANTI-LLM-SURFACE-001");
        assert!(!diags[0].blocking);
    }

    #[test]
    fn tower_lsp_underscore_variant_triggers_surface_001() {
        let o = obs("src/main.rs", "tower_lsp");
        let diags = evaluate(&[o], &default_config());
        assert_eq!(diags[0].code, "ANTI-LLM-SURFACE-001");
    }

    #[test]
    fn pack_observer_triggers_surface_003() {
        let o = obs("src/lib.rs", "PackObserver");
        let diags = evaluate(&[o], &default_config());
        assert_eq!(diags[0].code, "ANTI-LLM-SURFACE-003");
        assert!(diags[0].blocking);
    }

    #[test]
    fn initialize_without_caps_triggers_surface_005() {
        let o = obs("src/lib.rs", "initialize without 3.18 caps");
        let diags = evaluate(&[o], &default_config());
        assert_eq!(diags[0].code, "ANTI-LLM-SURFACE-005");
        assert!(diags[0].blocking);
    }

    #[test]
    fn unknown_construct_produces_no_diag() {
        let o = obs("src/lib.rs", "some_unrelated_construct");
        assert!(evaluate(&[o], &default_config()).is_empty());
    }
}
