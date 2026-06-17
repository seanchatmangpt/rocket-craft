use crate::observations::Observation;
use std::collections::HashMap;

fn extract_toml_string_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{} =", key);
    let idx = line.find(&needle)?;
    let after = &line[idx + needle.len()..];
    let start = after.find('"')? + 1;
    let rest = &after[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

pub fn parse_ggen_toml(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    let mut output_files: Vec<(String, usize)> = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // Detect second-class paths in output_file
        if let Some(val) = extract_toml_string_value(trimmed, "output_file") {
            output_files.push((val.to_string(), line_num));

            if val.contains("/generated/") || val.contains("/output/") || val.contains("/gen/") {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "ggen_toml".to_string(),
                    construct: "second_class_path".to_string(),
                    context: trimmed.to_string(),
                    message: format!(
                        "output_file '{}' contains second-class path segment (/generated/, /output/, or /gen/)",
                        val
                    ),
                });
            }

            // Layer violation: output_file without directory separator
            if !val.contains('/') {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "ggen_toml".to_string(),
                    construct: "layer_violation".to_string(),
                    context: trimmed.to_string(),
                    message: format!(
                        "output_file '{}' lacks directory separator — layer boundary violation",
                        val
                    ),
                });
            }
        }

        // Detect remote fetches in source/ontology fields
        for field in &["source", "ontology"] {
            if let Some(val) = extract_toml_string_value(trimmed, field) {
                if val.starts_with("http://") || val.starts_with("https://") {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: line_num,
                        column: 1,
                        kind: "ggen_toml".to_string(),
                        construct: "remote_fetch".to_string(),
                        context: trimmed.to_string(),
                        message: format!(
                            "Field '{}' contains remote URL '{}' — remote fetches are forbidden in ggen.toml",
                            field, val
                        ),
                    });
                }
            }
        }
    }

    obs
}

/// Cross-file: detect when multiple manifests declare identical output_file paths.
pub fn detect_competing_authority(all_files: &[(&str, &str)]) -> Vec<Observation> {
    let mut obs = Vec::new();
    let mut output_file_owners: HashMap<String, Vec<&str>> = HashMap::new();

    for (filepath, content) in all_files {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(val) = extract_toml_string_value(trimmed, "output_file") {
                output_file_owners
                    .entry(val.to_string())
                    .or_default()
                    .push(filepath);
            }
        }
    }

    for (output_file, owners) in &output_file_owners {
        if owners.len() > 1 {
            for filepath in owners {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: 1,
                    column: 1,
                    kind: "ggen_toml".to_string(),
                    construct: "competing_authority".to_string(),
                    context: output_file.clone(),
                    message: format!(
                        "output_file '{}' claimed by {} manifests — competing authority conflict",
                        output_file,
                        owners.len()
                    ),
                });
            }
        }
    }

    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_second_class_path() {
        let content = r#"output_file = "src/generated/foo.rs""#;
        let obs = parse_ggen_toml("ggen.toml", content);
        assert!(obs.iter().any(|o| o.construct == "second_class_path"));
    }

    #[test]
    fn detects_remote_fetch() {
        let content = r#"ontology = "https://example.org/onto.ttl""#;
        let obs = parse_ggen_toml("ggen.toml", content);
        assert!(obs.iter().any(|o| o.construct == "remote_fetch"));
    }

    #[test]
    fn clean_config_no_obs() {
        let content = r#"output_file = "src/domain/foo.rs"
source = "ontologies/local.ttl"
"#;
        let obs = parse_ggen_toml("ggen.toml", content);
        assert!(obs.is_empty());
    }

    #[test]
    fn detects_competing_authority() {
        let a = ("a/ggen.toml", r#"output_file = "src/domain/foo.rs""#);
        let b = ("b/ggen.toml", r#"output_file = "src/domain/foo.rs""#);
        let obs = detect_competing_authority(&[a, b]);
        assert!(!obs.is_empty());
        assert!(obs.iter().all(|o| o.construct == "competing_authority"));
    }
}
