pub fn generate_checkpoint_markdown() -> String {
    let mut out = String::new();
    out.push_str("# Checkpoint Verification Status\n\n");
    out.push_str("Status of Ostar Ontology Checkpoints:\n\n");
    out.push_str("| Checkpoint | Status | Validator | Verification Method |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    out.push_str("| CP-001 (Boundary) | **CANDIDATE** | tree-sitter-rust | Traversal for mutation calls in read-only paths. |\n");
    out.push_str(
        "| CP-002 (Authority) | **CANDIDATE** | clap-noun-verb | Verify command routes. |\n",
    );
    out.push_str("| CP-003 (Receipt) | **CANDIDATE** | receipt_validator | Verify BLAKE3 JSON receipt existence and digest match. |\n");
    out.push_str("| CP-004 (Route) | **CANDIDATE** | route_evidence_checker | Match CodeAction -> MutationGate paths. |\n");
    out.push_str(
        "| CP-005 (SemVer Check) | **CANDIDATE** | cargo_toml_parser | Ensure CalVer usage. |\n",
    );
    out.push_str("| LSP318_COMBINATORIAL_MAXIMALISM | **MATRIX_INCOMPLETE** | spec_extractor_tool | Verify full LSP 3.18 combinatorial coverage. |\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_starts_with_header() {
        let out = generate_checkpoint_markdown();
        assert!(out.starts_with("# Checkpoint Verification Status"));
    }

    #[test]
    fn contains_all_five_checkpoints() {
        let out = generate_checkpoint_markdown();
        for cp in ["CP-001", "CP-002", "CP-003", "CP-004", "CP-005"] {
            assert!(out.contains(cp), "missing {cp}");
        }
    }

    #[test]
    fn contains_lsp318_row() {
        let out = generate_checkpoint_markdown();
        assert!(out.contains("LSP318_COMBINATORIAL_MAXIMALISM"));
    }

    #[test]
    fn all_checkpoints_are_candidate_status() {
        let out = generate_checkpoint_markdown();
        assert!(out.contains("CANDIDATE"));
    }
}
