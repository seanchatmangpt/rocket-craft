use crate::observations::Observation;
use std::collections::{HashMap, HashSet, VecDeque};

/// Maximum BFS traversal depth for transitive failset expansion.
const MAX_DEPTH: usize = 8;

/// Parse `// @unwitnessed: <fn_name>` annotations from source content.
/// Each such annotation marks the named symbol as a failset seed.
pub fn extract_unwitnessed_seeds(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("// @unwitnessed:") {
            let fn_name = rest.trim().to_string();
            if !fn_name.is_empty() {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_idx + 1,
                    column: 1,
                    kind: "unwitnessed_seed".to_string(),
                    construct: fn_name.clone(),
                    context: trimmed.to_string(),
                    message: format!("Symbol '{}' declared @unwitnessed — failset seed", fn_name),
                });
            }
        }
    }

    obs
}

pub fn parse_refgraph_json(_fp: &str, _c: &str) -> Vec<Observation> {
    vec![]
}

/// Bounded BFS from unwitnessed seeds through the reverse reference graph.
///
/// Direction: reverse edges only (callee → caller). This mirrors import-export
/// attachment linkage. Depth is limited to MAX_DEPTH. Only explicit
/// `fn_reference` observations establish chain membership.
pub fn detect_transitive_failset(all_obs: &[Observation]) -> Vec<Observation> {
    let mut new_obs = Vec::new();

    // Collect seeds (unwitnessed_seed observations)
    let seeds: Vec<&str> = all_obs
        .iter()
        .filter(|o| o.kind == "unwitnessed_seed")
        .map(|o| o.construct.as_str())
        .collect();

    if seeds.is_empty() {
        return new_obs;
    }

    // Build reverse adjacency: callee → set of callers
    // from fn_reference observations
    let mut reverse_adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for o in all_obs.iter().filter(|o| o.kind == "fn_reference") {
        // context encodes "caller -> callee"
        let parts: Vec<&str> = o.context.splitn(2, "->").collect();
        if parts.len() == 2 {
            let caller = parts[0].trim();
            let callee = parts[1].trim();
            reverse_adj.entry(callee).or_default().push(caller);
        }
    }

    // BFS with depth limit
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    for seed in &seeds {
        if visited.insert(seed.to_string()) {
            queue.push_back((seed.to_string(), 0));
        }
    }

    while let Some((symbol, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }

        if let Some(callers) = reverse_adj.get(symbol.as_str()) {
            for &caller in callers {
                if visited.insert(caller.to_string()) {
                    new_obs.push(Observation {
                        file_path: "refgraph".to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: 1,
                        column: 1,
                        kind: "failset_member".to_string(),
                        construct: caller.to_string(),
                        context: format!("depth={} seed_chain={}", depth + 1, symbol),
                        message: format!(
                            "Symbol '{}' is a transitive failset member (depth {}, chain from '{}')",
                            caller,
                            depth + 1,
                            symbol
                        ),
                    });
                    queue.push_back((caller.to_string(), depth + 1));
                }
            }
        }
    }

    new_obs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn ref_obs(caller: &str, callee: &str) -> Observation {
        Observation {
            file_path: "refgraph".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "fn_reference".into(),
            construct: caller.into(),
            context: format!("{} -> {}", caller, callee),
            message: String::new(),
        }
    }

    fn seed_obs(name: &str) -> Observation {
        Observation {
            file_path: "src/lib.rs".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "unwitnessed_seed".into(),
            construct: name.into(), context: String::new(), message: String::new(),
        }
    }

    // ── extract_unwitnessed_seeds ─────────────────────────────────────────────

    #[test]
    fn extracts_single_unwitnessed_annotation() {
        let content = "// @unwitnessed: compute_score\nfn compute_score() {}";
        let obs = extract_unwitnessed_seeds("src/lib.rs", content);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].construct, "compute_score");
        assert_eq!(obs[0].line, 1);
    }

    #[test]
    fn extracts_multiple_annotations() {
        let content = "// @unwitnessed: foo\n// @unwitnessed: bar\n";
        let obs = extract_unwitnessed_seeds("src/lib.rs", content);
        assert_eq!(obs.len(), 2);
    }

    #[test]
    fn ignores_lines_without_annotation() {
        let content = "// this is just a comment\nfn foo() {}";
        let obs = extract_unwitnessed_seeds("src/lib.rs", content);
        assert!(obs.is_empty());
    }

    #[test]
    fn annotation_without_name_is_ignored() {
        let content = "// @unwitnessed:\n";
        let obs = extract_unwitnessed_seeds("src/lib.rs", content);
        assert!(obs.is_empty());
    }

    // ── detect_transitive_failset ─────────────────────────────────────────────

    #[test]
    fn no_seeds_produces_no_failset() {
        let obs = detect_transitive_failset(&[ref_obs("a", "b")]);
        assert!(obs.is_empty());
    }

    #[test]
    fn direct_caller_becomes_failset_member() {
        let obs = [seed_obs("bad"), ref_obs("caller_a", "bad")];
        let result = detect_transitive_failset(&obs);
        assert!(result.iter().any(|o| o.construct == "caller_a"));
    }

    #[test]
    fn transitive_caller_becomes_failset_member() {
        let obs = [
            seed_obs("bad"),
            ref_obs("middle", "bad"),
            ref_obs("outer", "middle"),
        ];
        let result = detect_transitive_failset(&obs);
        assert!(result.iter().any(|o| o.construct == "outer"));
    }

    #[test]
    fn failset_member_has_failset_member_kind() {
        let obs = [seed_obs("bad"), ref_obs("caller", "bad")];
        let result = detect_transitive_failset(&obs);
        assert!(result.iter().all(|o| o.kind == "failset_member"));
    }

    #[test]
    fn cycle_does_not_produce_infinite_loop() {
        // a -> b -> a (cycle)
        let obs = [
            seed_obs("a"),
            ref_obs("b", "a"),
            ref_obs("a", "b"), // cycle back
        ];
        let result = detect_transitive_failset(&obs);
        // Should terminate; 'b' is a failset member
        assert!(result.iter().any(|o| o.construct == "b"));
    }
}
