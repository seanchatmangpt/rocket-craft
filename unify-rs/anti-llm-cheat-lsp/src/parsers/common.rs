use crate::observations::Observation;

pub const ORACLE_FLOATS: &[(&str, &str)] = &[
    ("0.284171835", "Pearl 1988"),
    ("0.577350269", "sqrt(1/3) oracle"),
    ("1.618033988", "phi oracle"),
    ("2.718281828", "e oracle"),
    ("3.141592653", "pi oracle"),
];

pub fn detect_oracle_floats(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        for (float_str, source) in ORACLE_FLOATS {
            if line.contains(float_str) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_idx + 1,
                    column: 1,
                    kind: "oracle_float".to_string(),
                    construct: float_str.to_string(),
                    context: trimmed.to_string(),
                    message: format!(
                        "Oracle float literal '{}' ({}) — potential hardcoded answer injection",
                        float_str, source
                    ),
                });
            }
        }
    }
}

/// Iterate `{` and `}` chars in `s`, skipping those inside string or char literals.
/// Calls `f` for each effective brace character in left-to-right order.
pub fn for_effective_braces(s: &str, mut f: impl FnMut(char)) {
    let mut in_string = false;
    let mut in_char_lit = false;
    let mut escape = false;
    for ch in s.chars() {
        if escape {
            escape = false;
            continue;
        }
        if in_string || in_char_lit {
            if ch == '\\' {
                escape = true;
            } else if in_string && ch == '"' {
                in_string = false;
            } else if in_char_lit && ch == '\'' {
                in_char_lit = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '\'' => in_char_lit = true,
            '{' | '}' => f(ch),
            _ => { /* handled */ }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_oracle_floats ──────────────────────────────────────────────────

    #[test]
    fn detects_pi_oracle_float() {
        let mut obs = Vec::new();
        detect_oracle_floats("src/lib.rs", "let x = 3.141592653;", &mut obs);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].construct, "3.141592653");
        assert_eq!(obs[0].kind, "oracle_float");
    }

    #[test]
    fn detects_phi_oracle_float() {
        let mut obs = Vec::new();
        detect_oracle_floats("src/lib.rs", "const PHI: f64 = 1.618033988;", &mut obs);
        assert_eq!(obs.len(), 1);
        assert!(obs[0].message.contains("phi oracle"));
    }

    #[test]
    fn clean_code_produces_no_oracle_obs() {
        let mut obs = Vec::new();
        detect_oracle_floats("src/lib.rs", "let x = 42.0;", &mut obs);
        assert!(obs.is_empty());
    }

    #[test]
    fn line_number_is_1_indexed() {
        let mut obs = Vec::new();
        detect_oracle_floats("src/lib.rs", "fn foo() {}\nlet x = 3.141592653;", &mut obs);
        assert_eq!(obs[0].line, 2);
    }

    // ── for_effective_braces ─────────────────────────────────────────────────

    #[test]
    fn counts_braces_outside_strings() {
        let mut found = Vec::new();
        for_effective_braces("{ let x = \"{\"; }", |c| found.push(c));
        // Only the outer { and } should be counted; the one inside "" is skipped
        assert_eq!(found, vec!['{', '}']);
    }

    #[test]
    fn counts_nested_braces() {
        let mut found = Vec::new();
        for_effective_braces("{{x}}", |c| found.push(c));
        assert_eq!(found, vec!['{', '{', '}', '}']);
    }

    #[test]
    fn no_braces_in_string_literal() {
        let mut found = Vec::new();
        for_effective_braces("\"{}\"", |c| found.push(c));
        assert!(found.is_empty());
    }

    #[test]
    fn handles_escaped_quote_in_string() {
        let mut found = Vec::new();
        // The \" is an escape inside the string, so the } is still inside the string
        for_effective_braces(r#""foo \"bar\" {}" "#, |c| found.push(c));
        assert!(found.is_empty());
    }
}
