//! Rule for cross-product #4 — transitive failset over the reference graph.
//!
//! Emits `ANTI-LLM-REFGRAPH-001` for each site that the bounded reference
//! closure (`parsers::refgraph`) proved to transitively depend on an
//! unwitnessed symbol. The diagnostic is a CANDIDATE-level signal: it is raised
//! only on explicit reverse-reachability within the stated depth bound, so a
//! site with no chain to a seed is never flagged.

use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        if o.kind != "failset_member" {
            continue;
        }
        diags.push(AntiLlmDiagnostic {
            code: "ANTI-LLM-REFGRAPH-001".to_string(),
            category: "refgraph".to_string(),
            file_path: o.file_path.clone(),
            line: o.line,
            column: o.column,
            message: o.message.clone(),
            forbidden_implication: "DependentSite => Witnessed".to_string(),
            blocking: true,
            required_correction: "This symbol is reverse-reachable, within the bounded reference closure, from a symbol declared unwitnessed (`// @unwitnessed:`). A dependent of an unwitnessed symbol inherits its UNKNOWN status; it must not be treated as ADMITTED. Either discharge the seed symbol's witness or sever the reference dependency.".to_string(),
            required_next_proof: "Seed symbol carries an admitted witness (receipt with path/digest/boundary/negative-control), OR the reference edge to the seed is removed and re-scan shows the site absent from the failset.".to_string(),
        });
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn make_obs(kind: &str, line: usize) -> Observation {
        Observation {
            file_path: "src/foo.rs".to_string(),
            start_byte: 0,
            end_byte: 0,
            line,
            column: 1,
            kind: kind.to_string(),
            construct: "some_fn".to_string(),
            context: "".to_string(),
            message: "transitive failset".to_string(),
        }
    }

    #[test]
    fn failset_member_produces_diagnostic() {
        let obs = vec![make_obs("failset_member", 10)];
        let diags = evaluate(&obs);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "ANTI-LLM-REFGRAPH-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn non_failset_kind_is_ignored() {
        let obs = vec![make_obs("cargo_lock", 1), make_obs("other_kind", 2)];
        let diags = evaluate(&obs);
        assert!(diags.is_empty());
    }

    #[test]
    fn line_and_file_path_preserved() {
        let obs = vec![make_obs("failset_member", 42)];
        let diags = evaluate(&obs);
        assert_eq!(diags[0].line, 42);
        assert_eq!(diags[0].file_path, "src/foo.rs");
    }

    #[test]
    fn multiple_failset_members_each_produce_a_diagnostic() {
        let obs = vec![
            make_obs("failset_member", 1),
            make_obs("failset_member", 5),
            make_obs("failset_member", 9),
        ];
        let diags = evaluate(&obs);
        assert_eq!(diags.len(), 3);
    }

    #[test]
    fn empty_observations_returns_empty() {
        assert!(evaluate(&[]).is_empty());
    }

    #[test]
    fn forbidden_implication_is_correct() {
        let obs = vec![make_obs("failset_member", 1)];
        let diags = evaluate(&obs);
        assert_eq!(diags[0].forbidden_implication, "DependentSite => Witnessed");
    }
}
