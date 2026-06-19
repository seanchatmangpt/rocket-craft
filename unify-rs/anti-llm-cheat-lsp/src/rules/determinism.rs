use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

fn is_test_path(path: &str) -> bool {
    // Negative-control fixtures must trigger diagnostics — do not exclude them.
    if path.contains("negative_controls/") || path.contains("negative_controls\\") {
        return false;
    }
    path.contains("tests/")
        || path.ends_with("_test.rs")
        || path.contains("/test/")
        || path.contains("fixtures/")
}

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // CHEAT-001: hardcoded metrics (let fitness =, let score =, etc.)
        if o.kind == "raw_text"
            && matches!(
                o.construct.as_str(),
                "let fitness ="
                    | "let score ="
                    | "let precision ="
                    | "let recall ="
                    | "let f1_score ="
            )
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-CHEAT-001".to_string(),
                category: "determinism".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: format!(
                    "Hardcoded metric variable '{}' detected — oracle law violation. Metrics must be computed, not assigned.",
                    o.construct
                ),
                forbidden_implication: "HardcodedMetric => ComputedResult".to_string(),
                blocking: true,
                required_correction: "Remove hardcoded metric and compute from algorithm output.".to_string(),
                required_next_proof: "Run conformance check and capture live metric output.".to_string(),
            });
        }

        // CHEAT-002: seeded RNG (not in test paths)
        let is_seeded_rng = (o.kind == "ast_node"
            && matches!(
                o.construct.as_str(),
                "seed_from_u64"
                    | "SmallRng::from_seed"
                    | "StdRng::from_seed"
                    | "SeedableRng::seed_from_u64"
                    | "ChaCha8Rng::from_seed"
                    | "from_seed"
            ))
            || (o.kind == "raw_text"
                && matches!(
                    o.construct.as_str(),
                    "seed_from_u64" | "SmallRng::from_seed" | "StdRng::from_seed"
                ));
        if is_seeded_rng && !is_test_path(&o.file_path) {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-CHEAT-002".to_string(),
                category: "determinism".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Seeded RNG detected in non-test code — oracle law violation. Seeded RNG produces predictable output that can be gamed.".to_string(),
                forbidden_implication: "SeededRNG => DeterministicOutput".to_string(),
                blocking: true,
                required_correction: "Remove seeded RNG from production code. Use crypto-secure random or algorithmic output.".to_string(),
                required_next_proof: "Verify no seeded RNG in production paths.".to_string(),
            });
        }

        // CHEAT-003: copied output hash
        if o.kind == "raw_text" && o.construct == "\"output_hash\": \"" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-CHEAT-003".to_string(),
                category: "determinism".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Hardcoded output_hash literal detected — receipt fabrication risk."
                    .to_string(),
                forbidden_implication: "HardcodedHash => ValidReceipt".to_string(),
                blocking: true,
                required_correction: "Compute output_hash from actual algorithm output at runtime."
                    .to_string(),
                required_next_proof: "Verify output_hash is computed, not copied.".to_string(),
            });
        }

        // STRANGE-010: #[allow(...)] suppression cheat (not in test paths)
        if o.kind == "ast_node" && o.construct == "allow_cheat_attr" && !is_test_path(&o.file_path)
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-010".to_string(),
                category: "determinism".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Suppression attribute #[allow(...)] for dead_code/unused/warnings detected — silences real violations.".to_string(),
                forbidden_implication: "AllowSuppression => CleanBuild".to_string(),
                blocking: false,
                required_correction: "Remove suppression attribute and fix the underlying issue.".to_string(),
                required_next_proof: "Build without suppression attribute and verify zero warnings.".to_string(),
            });
        }

        // STRANGE-011: unsafe block or function (not in test paths)
        if o.kind == "ast_node"
            && matches!(o.construct.as_str(), "unsafe_block" | "unsafe_fn_or_impl")
            && !is_test_path(&o.file_path)
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-STRANGE-011".to_string(),
                category: "determinism".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: format!(
                    "Unsafe {} detected — potential oracle law bypass or memory safety violation.",
                    if o.construct == "unsafe_block" { "block" } else { "function/impl" }
                ),
                forbidden_implication: "UnsafeCode => SafeExecution".to_string(),
                blocking: false,
                required_correction: "Replace unsafe code with safe alternatives or document necessity with a receipt.".to_string(),
                required_next_proof: "Audit unsafe usage and confirm necessity with process evidence.".to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(file: &str, kind: &str, construct: &str) -> Observation {
        Observation {
            file_path: file.into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: kind.into(), construct: construct.into(),
            context: String::new(), message: String::new(),
        }
    }

    #[test]
    fn empty_returns_no_diags() {
        assert!(evaluate(&[]).is_empty());
    }

    // ── CHEAT-001 ─────────────────────────────────────────────────────────────

    #[test]
    fn hardcoded_metric_triggers_cheat_001() {
        let diags = evaluate(&[obs("src/breed.rs", "raw_text", "let fitness =")]);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "ANTI-LLM-CHEAT-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn hardcoded_score_triggers_cheat_001() {
        let diags = evaluate(&[obs("src/lib.rs", "raw_text", "let score =")]);
        assert_eq!(diags[0].code, "ANTI-LLM-CHEAT-001");
    }

    #[test]
    fn unknown_raw_text_does_not_trigger_cheat_001() {
        let diags = evaluate(&[obs("src/lib.rs", "raw_text", "let result =")]);
        assert!(diags.iter().all(|d| d.code != "ANTI-LLM-CHEAT-001"));
    }

    // ── CHEAT-002 ─────────────────────────────────────────────────────────────

    #[test]
    fn seeded_rng_in_prod_triggers_cheat_002() {
        let diags = evaluate(&[obs("src/lib.rs", "ast_node", "seed_from_u64")]);
        assert_eq!(diags[0].code, "ANTI-LLM-CHEAT-002");
        assert!(diags[0].blocking);
    }

    #[test]
    fn seeded_rng_in_test_file_is_allowed() {
        let diags = evaluate(&[obs("tests/foo.rs", "ast_node", "seed_from_u64")]);
        assert!(diags.iter().all(|d| d.code != "ANTI-LLM-CHEAT-002"));
    }

    #[test]
    fn seeded_rng_in_negative_controls_fires() {
        // negative_controls/ is explicitly NOT excluded
        let diags = evaluate(&[obs("negative_controls/cheat.rs", "ast_node", "seed_from_u64")]);
        assert_eq!(diags[0].code, "ANTI-LLM-CHEAT-002");
    }

    // ── CHEAT-003 ─────────────────────────────────────────────────────────────

    #[test]
    fn hardcoded_output_hash_triggers_cheat_003() {
        let diags = evaluate(&[obs("src/lib.rs", "raw_text", "\"output_hash\": \"")]);
        assert_eq!(diags[0].code, "ANTI-LLM-CHEAT-003");
        assert!(diags[0].blocking);
    }

    // ── STRANGE-010 ───────────────────────────────────────────────────────────

    #[test]
    fn allow_cheat_attr_in_prod_triggers_strange_010() {
        let diags = evaluate(&[obs("src/lib.rs", "ast_node", "allow_cheat_attr")]);
        assert_eq!(diags[0].code, "ANTI-LLM-STRANGE-010");
        assert!(!diags[0].blocking);
    }

    #[test]
    fn allow_cheat_attr_in_test_is_allowed() {
        let diags = evaluate(&[obs("tests/foo.rs", "ast_node", "allow_cheat_attr")]);
        assert!(diags.iter().all(|d| d.code != "ANTI-LLM-STRANGE-010"));
    }

    // ── STRANGE-011 ───────────────────────────────────────────────────────────

    #[test]
    fn unsafe_block_in_prod_triggers_strange_011() {
        let diags = evaluate(&[obs("src/lib.rs", "ast_node", "unsafe_block")]);
        assert_eq!(diags[0].code, "ANTI-LLM-STRANGE-011");
        assert!(!diags[0].blocking);
    }

    #[test]
    fn unsafe_block_in_test_is_allowed() {
        let diags = evaluate(&[obs("tests/foo.rs", "ast_node", "unsafe_block")]);
        assert!(diags.iter().all(|d| d.code != "ANTI-LLM-STRANGE-011"));
    }
}
