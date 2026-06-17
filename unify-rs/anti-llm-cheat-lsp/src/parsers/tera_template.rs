use crate::observations::Observation;
use std::collections::HashSet;
use std::path::Path;

/// Tera framework variables that are not projected from SPARQL queries.
const BUILTIN_TERA_VARS: &[&str] = &["loop", "self", "config", "now"];

/// Extract `{{ varname }}` variable references from a Tera template.
/// Returns root names only (before `.`, space, or `|`).
fn extract_tera_variables(content: &str) -> HashSet<String> {
    let mut vars = HashSet::new();
    let mut i = 0;
    let bytes = content.as_bytes();

    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Find closing }}
            if let Some(end) = content[i + 2..].find("}}") {
                let inner = content[i + 2..i + 2 + end].trim();
                // Extract root name (before . / | / space)
                let root: String = inner
                    .chars()
                    .take_while(|&c| c.is_alphanumeric() || c == '_')
                    .collect();
                if !root.is_empty()
                    && root.chars().next().map(|c| !c.is_ascii_digit()).unwrap_or(false)
                    && !BUILTIN_TERA_VARS.contains(&root.as_str())
                {
                    vars.insert(root.to_lowercase());
                }
                i += 2 + end + 2;
                continue;
            }
        }
        i += 1;
    }

    vars
}

/// Extract projected variable names from a SPARQL SELECT query.
/// Returns empty set for wildcard queries (`SELECT *`).
fn extract_sparql_select_vars(query: &str) -> HashSet<String> {
    let mut vars = HashSet::new();
    let upper = query.to_uppercase();

    // Find SELECT clause
    if let Some(select_idx) = upper.find("SELECT") {
        let after_select = &query[select_idx + 6..];
        let trimmed = after_select.trim_start();

        // Wildcard: return empty (skip validation)
        if trimmed.starts_with('*') {
            return vars;
        }

        // Find WHERE keyword
        let end = upper[select_idx + 6..]
            .find("WHERE")
            .unwrap_or(trimmed.len());
        let select_vars = &after_select[..end];

        // Extract ?variable names
        for word in select_vars.split_whitespace() {
            if let Some(name) = word.strip_prefix('?') {
                let clean: String = name
                    .chars()
                    .take_while(|&c| c.is_alphanumeric() || c == '_')
                    .collect();
                if !clean.is_empty() {
                    vars.insert(clean.to_lowercase());
                }
            }
        }
    }

    vars
}

pub fn parse_tera_template(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();

    // Locate sibling .rq file with same stem
    let path = Path::new(filepath);
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return obs,
    };
    let parent = path.parent().unwrap_or(Path::new("."));
    let rq_path = parent.join(format!("{}.rq", stem));

    let rq_content = match std::fs::read_to_string(&rq_path) {
        Ok(c) => c,
        Err(_) => return obs, // No paired .rq file; skip validation
    };

    let sparql_vars = extract_sparql_select_vars(&rq_content);

    // Wildcard SELECT: skip validation
    if sparql_vars.is_empty() {
        return obs;
    }

    let template_vars = extract_tera_variables(content);

    // Report template vars not projected by SPARQL
    for var in &template_vars {
        if !sparql_vars.contains(var) {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: 1,
                column: 1,
                kind: "tera_template".to_string(),
                construct: var.clone(),
                context: format!("rq={}", rq_path.display()),
                message: format!(
                    "TPL-001: Template variable '{}' not projected by paired SPARQL query '{}'",
                    var,
                    rq_path.display()
                ),
            });
        }
    }

    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_tera_variables_from_filter_expr() {
        let tmpl = "Hello {{ user.name | upper }}, your score is {{ score }}!";
        let vars = extract_tera_variables(tmpl);
        assert!(vars.contains("user"));
        assert!(vars.contains("score"));
    }

    #[test]
    fn extracts_sparql_projection() {
        let query = "SELECT ?name ?score WHERE { ?x ex:name ?name ; ex:score ?score . }";
        let vars = extract_sparql_select_vars(query);
        assert!(vars.contains("name"));
        assert!(vars.contains("score"));
    }

    #[test]
    fn wildcard_select_returns_empty() {
        let query = "SELECT * WHERE { ?x ?y ?z }";
        let vars = extract_sparql_select_vars(query);
        assert!(vars.is_empty());
    }
}
