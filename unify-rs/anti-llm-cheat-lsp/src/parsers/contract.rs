use crate::observations::Observation;
use std::collections::{HashMap, HashSet};

/// Extract breed_id from a file path containing `breeds/<breed_id>`.
fn extract_breed_id(path: &str) -> Option<&str> {
    let idx = path.find("breeds/")?;
    let after = &path[idx + "breeds/".len()..];
    let end = after
        .find('/')
        .or_else(|| after.find(".rs"))
        .unwrap_or(after.len());
    Some(&after[..end])
}

fn is_src_path(path: &str) -> bool {
    (path.contains("/src/") || path.contains("src/breeds")) && !path.contains("tests/")
}

fn is_test_path(path: &str) -> bool {
    path.contains("tests/") || path.ends_with("_test.rs") || path.contains("/test/")
}

/// Cross-file contract schism detection (A9).
///
/// Groups fn_definition observations by breed_id and compares the set of
/// function names in src/breeds/<b>.rs vs oracle test files for that breed.
pub fn detect_contract_schism(all_obs: &[Observation]) -> Vec<Observation> {
    let mut obs = Vec::new();

    // Collect fn_definition observations
    let fn_defs: Vec<&Observation> = all_obs
        .iter()
        .filter(|o| o.kind == "fn_definition")
        .collect();

    // Group by breed_id × (src vs test)
    let mut breed_src_fns: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut breed_test_fns: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut breed_src_path: HashMap<&str, &str> = HashMap::new();

    for o in &fn_defs {
        if let Some(breed_id) = extract_breed_id(&o.file_path) {
            if is_src_path(&o.file_path) {
                breed_src_fns
                    .entry(breed_id)
                    .or_default()
                    .insert(&o.construct);
                breed_src_path.entry(breed_id).or_insert(&o.file_path);
            } else if is_test_path(&o.file_path) {
                breed_test_fns
                    .entry(breed_id)
                    .or_default()
                    .insert(&o.construct);
            }
        }
    }

    // CONTRACT-001: zero function name overlap between impl and oracle test
    for (breed_id, src_fns) in &breed_src_fns {
        if let Some(test_fns) = breed_test_fns.get(breed_id) {
            let overlap: HashSet<&&str> = src_fns.intersection(test_fns).collect();
            // Only flag if BOTH sides have substantial functions and ZERO overlap
            if overlap.is_empty() && src_fns.len() >= 3 && test_fns.len() >= 3 {
                let file_path = breed_src_path.get(breed_id).copied().unwrap_or("unknown");
                obs.push(Observation {
                    file_path: file_path.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: 1,
                    column: 1,
                    kind: "contract_schism".to_string(),
                    construct: "contract_vocab_divergence".to_string(),
                    context: breed_id.to_string(),
                    message: format!(
                        "Breed '{}': zero function name overlap between src ({} fns) and oracle test ({} fns) — A9 contract schism",
                        breed_id, src_fns.len(), test_fns.len()
                    ),
                });
            }
        }
    }

    // CONTRACT-002: same function name defined in BOTH src and test for the same breed
    for (breed_id, src_fns) in &breed_src_fns {
        if let Some(test_fns) = breed_test_fns.get(breed_id) {
            let shadows: Vec<&&str> = src_fns.intersection(test_fns).collect();
            // If a non-trivial named fn appears in both (not just "run" which is expected in trait)
            for &shadow_fn in &shadows {
                if !matches!(*shadow_fn, "run" | "new" | "default") {
                    let file_path = breed_src_path.get(breed_id).copied().unwrap_or("unknown");
                    obs.push(Observation {
                        file_path: file_path.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: 1,
                        column: 1,
                        kind: "contract_schism".to_string(),
                        construct: "contract_fn_shadow".to_string(),
                        context: format!("breed={} fn={}", breed_id, shadow_fn),
                        message: format!(
                            "Breed '{}': function '{}' defined in BOTH src and test — shadow override cheat (A9)",
                            breed_id, shadow_fn
                        ),
                    });
                }
            }
        }
    }

    obs
}

pub fn parse_contract_json(_fp: &str, _c: &str) -> Vec<Observation> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn fn_def(file: &str, name: &str) -> Observation {
        Observation {
            file_path: file.into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "fn_definition".into(),
            construct: name.into(), context: String::new(), message: String::new(),
        }
    }

    // ── extract_breed_id ──────────────────────────────────────────────────────

    #[test]
    fn extracts_breed_id_from_src_path() {
        assert_eq!(extract_breed_id("src/breeds/my_breed/lib.rs"), Some("my_breed"));
    }

    #[test]
    fn extracts_breed_id_from_rs_file() {
        assert_eq!(extract_breed_id("breeds/foo.rs"), Some("foo"));
    }

    #[test]
    fn returns_none_when_no_breeds_segment() {
        assert!(extract_breed_id("src/lib.rs").is_none());
    }

    // ── detect_contract_schism ────────────────────────────────────────────────

    #[test]
    fn no_observations_produces_no_schism() {
        assert!(detect_contract_schism(&[]).is_empty());
    }

    #[test]
    fn zero_fn_overlap_triggers_contract_001() {
        let src_path = "src/breeds/alpha/src/lib.rs";
        let test_path = "src/breeds/alpha/tests/oracle.rs";
        let obs: Vec<Observation> = vec![
            fn_def(src_path, "compute"),
            fn_def(src_path, "normalize"),
            fn_def(src_path, "validate"),
            fn_def(test_path, "check_x"),
            fn_def(test_path, "check_y"),
            fn_def(test_path, "check_z"),
        ];
        let result = detect_contract_schism(&obs);
        assert!(result.iter().any(|o| o.construct == "contract_vocab_divergence"));
    }

    #[test]
    fn shared_fn_overlap_does_not_trigger_001() {
        let src_path = "src/breeds/beta/src/lib.rs";
        let test_path = "src/breeds/beta/tests/oracle.rs";
        let obs: Vec<Observation> = vec![
            fn_def(src_path, "compute"),
            fn_def(src_path, "normalize"),
            fn_def(src_path, "validate"),
            fn_def(test_path, "compute"),   // overlap!
            fn_def(test_path, "check_y"),
            fn_def(test_path, "check_z"),
        ];
        let result = detect_contract_schism(&obs);
        assert!(!result.iter().any(|o| o.construct == "contract_vocab_divergence"));
    }

    #[test]
    fn shadow_fn_triggers_contract_002() {
        let src_path = "src/breeds/gamma/src/lib.rs";
        let test_path = "src/breeds/gamma/tests/oracle.rs";
        let obs: Vec<Observation> = vec![
            fn_def(src_path, "compute"),
            fn_def(src_path, "normalize"),
            fn_def(src_path, "validate"),
            fn_def(test_path, "compute"),   // shadow!
            fn_def(test_path, "check_y"),
            fn_def(test_path, "check_z"),
        ];
        let result = detect_contract_schism(&obs);
        assert!(result.iter().any(|o| o.construct == "contract_fn_shadow"));
    }

    #[test]
    fn new_and_run_shadows_are_not_flagged() {
        let src_path = "src/breeds/delta/src/lib.rs";
        let test_path = "src/breeds/delta/tests/oracle.rs";
        let obs: Vec<Observation> = vec![
            fn_def(src_path, "new"),
            fn_def(src_path, "run"),
            fn_def(src_path, "default"),
            fn_def(test_path, "new"),   // allowed shadow
            fn_def(test_path, "run"),   // allowed shadow
            fn_def(test_path, "default"),
        ];
        let result = detect_contract_schism(&obs);
        // no shadow violation for allowed fns
        assert!(!result.iter().any(|o| o.construct == "contract_fn_shadow"));
    }

    #[test]
    fn threshold_below_three_fns_does_not_trigger_001() {
        let src_path = "src/breeds/tiny/src/lib.rs";
        let test_path = "src/breeds/tiny/tests/oracle.rs";
        let obs: Vec<Observation> = vec![
            fn_def(src_path, "a"),
            fn_def(src_path, "b"),
            fn_def(test_path, "x"),
            fn_def(test_path, "y"),
        ];
        // only 2 fns each — under the threshold of 3
        let result = detect_contract_schism(&obs);
        assert!(!result.iter().any(|o| o.construct == "contract_vocab_divergence"));
    }
}
