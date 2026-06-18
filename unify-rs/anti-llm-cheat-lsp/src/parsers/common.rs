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
