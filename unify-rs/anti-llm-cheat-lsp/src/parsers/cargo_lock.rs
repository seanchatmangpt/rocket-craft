use crate::observations::Observation;

pub fn parse_cargo_lock(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();

    // We scan for lockfile entries: name = "tower-lsp"
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.replace(" ", "").contains("name=\"tower-lsp\"") {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_idx + 1,
                column: 1,
                kind: "cargo_lock".to_string(),
                construct: "tower-lsp lock entry".to_string(),
                context: trimmed.to_string(),
                message: "Plain tower-lsp dependency found in Cargo.lock".to_string(),
            });
        }
    }

    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_tower_lsp_entry() {
        let content = "name = \"tower-lsp\"\nversion = \"0.20.0\"\n";
        let obs = parse_cargo_lock("Cargo.lock", content);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].kind, "cargo_lock");
        assert_eq!(obs[0].line, 1);
        assert!(obs[0].message.contains("tower-lsp"));
    }

    #[test]
    fn no_observations_when_tower_lsp_absent() {
        let content = "name = \"tokio\"\nversion = \"1.0.0\"\n";
        let obs = parse_cargo_lock("Cargo.lock", content);
        assert!(obs.is_empty());
    }

    #[test]
    fn file_path_preserved_in_observation() {
        let content = "name = \"tower-lsp\"\n";
        let obs = parse_cargo_lock("path/to/Cargo.lock", content);
        assert_eq!(obs[0].file_path, "path/to/Cargo.lock");
    }

    #[test]
    fn detects_tower_lsp_with_spaces_around_equals() {
        // "name = \"tower-lsp\"" with extra spaces compresses to same after replace
        let content = "name  =  \"tower-lsp\"\n";
        let obs = parse_cargo_lock("Cargo.lock", content);
        assert_eq!(obs.len(), 1);
    }

    #[test]
    fn correct_line_number_for_second_occurrence() {
        let content = "name = \"serde\"\nname = \"tower-lsp\"\n";
        let obs = parse_cargo_lock("Cargo.lock", content);
        assert_eq!(obs[0].line, 2);
    }

    #[test]
    fn empty_content_returns_empty() {
        let obs = parse_cargo_lock("Cargo.lock", "");
        assert!(obs.is_empty());
    }
}
