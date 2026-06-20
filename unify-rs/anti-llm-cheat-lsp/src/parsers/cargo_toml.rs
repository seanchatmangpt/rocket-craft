use crate::observations::Observation;

fn is_calver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    // First part must be exactly 2 digits (YY)
    if parts[0].len() != 2 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // Remaining parts must be 1-2 digits
    for part in &parts[1..] {
        if part.is_empty() || part.len() > 2 || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    true
}

fn extract_quoted_value<'a>(line: &'a str, needle: &str) -> Option<&'a str> {
    let idx = line.find(needle)?;
    let after = &line[idx + needle.len()..];
    let start = after.find('"')? + 1;
    let rest = &after[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

pub fn parse_cargo_toml(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    let mut in_workspace_package = false;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // Track [workspace.package] section
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        } else if trimmed.starts_with('[') {
            in_workspace_package = false;
        }

        // Detect plain tower-lsp dependency
        if trimmed.contains("tower-lsp")
            && !trimmed.starts_with('#')
            && !trimmed.contains("tower-lsp-boilerplate")
        {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "cargo_toml".to_string(),
                construct: "tower-lsp dependency".to_string(),
                context: trimmed.to_string(),
                message: "Plain tower-lsp dependency found in Cargo.toml".to_string(),
            });
        }

        // Detect default template version "1.0.0"
        if let Some(ver) = extract_quoted_value(trimmed, "version") {
            if ver == "1.0.0" {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "cargo_toml".to_string(),
                    construct: "default_template_version".to_string(),
                    context: trimmed.to_string(),
                    message:
                        "Default template version '1.0.0' found — replace with CalVer (YY.M.D)"
                            .to_string(),
                });
            }
        }

        // Detect path dependencies with non-CalVer versions
        if trimmed.contains("path =") {
            if let Some(ver) = extract_quoted_value(trimmed, "version") {
                if !is_calver(ver) && ver != "1.0.0" {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: line_num,
                        column: 1,
                        kind: "cargo_toml".to_string(),
                        construct: "path_dep_semver".to_string(),
                        context: trimmed.to_string(),
                        message: format!(
                            "Path dependency uses non-CalVer version '{}'; expected YY.M.D",
                            ver
                        ),
                    });
                }
            }
        }

        // Detect [workspace.package] declaring semver instead of calver
        if in_workspace_package {
            if let Some(ver) = extract_quoted_value(trimmed, "version") {
                if !is_calver(ver) {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: line_num,
                        column: 1,
                        kind: "cargo_toml".to_string(),
                        construct: "workspace_semver".to_string(),
                        context: trimmed.to_string(),
                        message: format!(
                            "[workspace.package] declares SemVer '{}' instead of CalVer (YY.M.D)",
                            ver
                        ),
                    });
                }
            }
        }
    }

    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_calver ─────────────────────────────────────────────────────────────

    #[test]
    fn valid_calver_yy_mm_patch() {
        assert!(is_calver("25.06.1"));
        assert!(is_calver("24.12.10"));
    }

    #[test]
    fn semver_is_not_calver() {
        assert!(!is_calver("1.0.0"));
        assert!(!is_calver("0.1.2"));
    }

    #[test]
    fn calver_requires_three_parts() {
        assert!(!is_calver("25.06"));
        assert!(!is_calver("25.06.1.0"));
    }

    #[test]
    fn calver_first_part_must_be_two_digits() {
        assert!(!is_calver("2.06.1")); // single digit year
        assert!(!is_calver("125.06.1")); // three digit year
    }

    // ── extract_quoted_value ──────────────────────────────────────────────────

    #[test]
    fn extract_finds_value_after_needle() {
        let line = "version = \"1.0.0\"";
        assert_eq!(extract_quoted_value(line, "version"), Some("1.0.0"));
    }

    #[test]
    fn extract_returns_none_when_needle_absent() {
        assert!(extract_quoted_value("name = \"foo\"", "version").is_none());
    }

    #[test]
    fn extract_returns_none_when_no_quotes() {
        assert!(extract_quoted_value("version = 1", "version").is_none());
    }

    // ── parse_cargo_toml ─────────────────────────────────────────────────────

    #[test]
    fn tower_lsp_dep_produces_obs() {
        let content = "tower-lsp = \"0.20\"\n";
        let obs = parse_cargo_toml("Cargo.toml", content);
        assert!(obs.iter().any(|o| o.construct.contains("tower-lsp")));
    }

    #[test]
    fn tower_lsp_in_comment_is_ignored() {
        let content = "# tower-lsp = \"0.20\"\n";
        let obs = parse_cargo_toml("Cargo.toml", content);
        assert!(obs.iter().all(|o| !o.construct.contains("tower-lsp")));
    }

    #[test]
    fn clean_cargo_toml_produces_no_obs() {
        let content = "[package]\nname = \"foo\"\nversion = \"0.1.0\"\n";
        let obs = parse_cargo_toml("Cargo.toml", content);
        assert!(obs.is_empty());
    }
}
