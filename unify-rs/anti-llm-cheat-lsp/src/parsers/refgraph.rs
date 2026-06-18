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
