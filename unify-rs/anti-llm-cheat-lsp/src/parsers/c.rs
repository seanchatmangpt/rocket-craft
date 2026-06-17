use crate::observations::Observation;

const ORACLE_FLOATS: &[(&str, &str)] = &[
    ("0.284171835", "Pearl 1988"),
    ("0.577350269", "sqrt(1/3) oracle"),
    ("1.618033988", "phi oracle"),
    ("2.718281828", "e oracle"),
    ("3.141592653", "pi oracle"),
];

// Unsafe C string/buffer functions that lack bounds checking
const UNSAFE_STRING_FNS: &[&str] = &["strcpy(", "strcat(", "gets(", "sprintf(", "scanf("];

// Debug output calls that should not be in production code
const DEBUG_CALLS: &[&str] = &["printf(", "fprintf(stderr", "fprintf(stdout"];

fn is_test_path(path: &str) -> bool {
    path.contains("test/") || path.contains("tests/") || path.ends_with("_test.c")
        || path.ends_with("_spec.c") || path.contains("check/")
}

struct FnState {
    name: String,
    start_line: usize,
    line_count: usize,
    branch_count: usize,
    nesting_depth: usize,
    current_depth: usize,
    body_tokens: usize,
}

fn detect_oracle_floats(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    for (line_idx, line) in content.lines().enumerate() {
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
                    context: line.trim().to_string(),
                    message: format!(
                        "Oracle float literal '{}' ({}) — potential hardcoded answer injection",
                        float_str, source
                    ),
                });
            }
        }
    }
}

fn detect_victory_comments(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    const VICTORY_TERMS: &[&str] = &[
        "fully implemented", "complete", "works perfectly", "all done",
        "production ready", "verified correct", "tested and working",
    ];
    for (line_idx, line) in content.lines().enumerate() {
        let lower = line.to_lowercase();
        if !lower.contains("//") && !lower.contains("/*") {
            continue;
        }
        for term in VICTORY_TERMS {
            if lower.contains(term) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_idx + 1,
                    column: 1,
                    kind: "c_claim".to_string(),
                    construct: term.to_string(),
                    context: line.trim().to_string(),
                    message: format!("Victory/overclaim language '{}' in comment", term),
                });
            }
        }
    }
}

fn detect_todo_comments(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    const MARKERS: &[&str] = &["TODO", "FIXME", "HACK", "STUB", "XXX", "TEMP"];
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with("*") {
            continue;
        }
        let upper = trimmed.to_uppercase();
        for marker in MARKERS {
            if upper.contains(marker) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_idx + 1,
                    column: 1,
                    kind: "c_todo".to_string(),
                    construct: marker.to_string(),
                    context: trimmed.to_string(),
                    message: format!("Unresolved '{}' comment — placeholder or stub left in code", marker),
                });
                break;
            }
        }
    }
}

fn detect_unsafe_fns(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        for func in UNSAFE_STRING_FNS {
            if trimmed.contains(func) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_idx + 1,
                    column: 1,
                    kind: "c_unsafe".to_string(),
                    construct: func.trim_end_matches('(').to_string(),
                    context: trimmed.to_string(),
                    message: format!(
                        "Unsafe C function '{}' — no bounds checking, use safe alternatives",
                        func.trim_end_matches('(')
                    ),
                });
            }
        }
    }
}

fn detect_debug_artifacts(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    if is_test_path(filepath) {
        return;
    }
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        for call in DEBUG_CALLS {
            if trimmed.contains(call) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_idx + 1,
                    column: 1,
                    kind: "c_debug_artifact".to_string(),
                    construct: call.trim_end_matches('(').to_string(),
                    context: trimmed.to_string(),
                    message: format!(
                        "Debug output call '{}' left in production code",
                        call.trim_end_matches('(')
                    ),
                });
            }
        }
    }
}

fn detect_getenv(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    if is_test_path(filepath) {
        return;
    }
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        if trimmed.contains("getenv(") {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_idx + 1,
                column: 1,
                kind: "c_oracle".to_string(),
                construct: "getenv".to_string(),
                context: trimmed.to_string(),
                message: "getenv() in production code — environment oracle injection channel".to_string(),
            });
        }
    }
}

fn detect_malloc_without_check(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        if !trimmed.contains("malloc(") && !trimmed.contains("calloc(") && !trimmed.contains("realloc(") {
            continue;
        }
        // Check if the result is checked: next non-empty line should contain if/assert/NULL check
        let next_check = lines.iter().skip(i + 1).take(3).any(|l| {
            let t = l.trim();
            t.starts_with("if") || t.contains("== NULL") || t.contains("!= NULL")
                || t.starts_with("assert(") || t.contains("NULL)")
        });
        if !next_check {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: i + 1,
                column: 1,
                kind: "c_unsafe".to_string(),
                construct: "malloc_unchecked".to_string(),
                context: trimmed.to_string(),
                message: "malloc/calloc/realloc result not checked for NULL — potential null deref on allocation failure".to_string(),
            });
        }
    }
}

fn detect_hardcoded_lookup_tables(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    // Detect large static arrays initialized with many literals — oracle memo table signal
    let mut in_array = false;
    let mut array_start = 0usize;
    let mut entry_count = 0usize;
    let mut array_name = String::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if !in_array {
            // Detect static array declaration with initializer
            if (trimmed.contains("static") || trimmed.contains("const"))
                && trimmed.contains('[')
                && trimmed.contains(']')
                && trimmed.contains('=')
                && trimmed.contains('{')
            {
                in_array = true;
                array_start = line_idx + 1;
                entry_count = 0;
                // Extract array name
                let eq_pos = trimmed.find('=').unwrap_or(0);
                let before_eq = &trimmed[..eq_pos];
                array_name = before_eq
                    .split_whitespace()
                    .last()
                    .unwrap_or("unknown")
                    .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_string();
            }
        } else {
            // Count entries (commas as proxy)
            entry_count += trimmed.matches(',').count();
            if trimmed.contains('}') {
                in_array = false;
                if entry_count > 20 {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: array_start,
                        column: 1,
                        kind: "c_oracle".to_string(),
                        construct: "large_static_array".to_string(),
                        context: format!("array '{}' with {} entries", array_name, entry_count),
                        message: format!(
                            "Large static array '{}' (~{} entries) — potential oracle lookup table",
                            array_name, entry_count
                        ),
                    });
                }
                entry_count = 0;
            }
        }
    }
}

fn detect_stub_functions(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        // Detect C function definition opening: non-empty line ending with `{` that looks like a fn
        // Heuristic: line contains `(` and `)` and ends with `{`, not a control flow keyword
        let is_fn_open = trimmed.ends_with('{')
            && trimmed.contains('(')
            && trimmed.contains(')')
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("switch ")
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("*")
            && !trimmed.is_empty();

        if is_fn_open {
            // Scan the body until matching `}` at depth 0
            let mut depth = 1usize;
            let mut body_lines = Vec::new();
            let mut j = i + 1;
            while j < lines.len() && depth > 0 {
                let bl = lines[j].trim();
                for ch in bl.chars() {
                    match ch {
                        '{' => depth += 1,
                        '}' => { if depth > 0 { depth -= 1; } }
                        _ => {}
                    }
                }
                if depth > 0 {
                    body_lines.push(bl);
                }
                j += 1;
            }

            // A function is a stub if its body is empty or only contains a single return constant
            let non_empty: Vec<&&str> = body_lines.iter().filter(|l| !l.is_empty() && !l.starts_with("//")).collect();
            let is_stub = non_empty.is_empty()
                || (non_empty.len() == 1 && (
                    non_empty[0].starts_with("return 0")
                    || non_empty[0].starts_with("return NULL")
                    || non_empty[0].starts_with("return -1")
                    || non_empty[0].starts_with("return false")
                    || non_empty[0].starts_with("return true")
                    || *non_empty[0] == "return;"
                ));

            if is_stub {
                // Extract function name (last identifier before `(`)
                let paren = trimmed.find('(').unwrap_or(trimmed.len());
                let before = &trimmed[..paren];
                let name = before.split_whitespace().last().unwrap_or("unknown");
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: i + 1,
                    column: 1,
                    kind: "c_stub".to_string(),
                    construct: "stub_function".to_string(),
                    context: trimmed.to_string(),
                    message: format!(
                        "Function '{}' is a stub — body is empty or returns only a constant",
                        name
                    ),
                });
            }

            i = j;
            continue;
        }
        i += 1;
    }
}

fn collect_fn_metrics(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    let mut current: Option<FnState> = None;
    let mut brace_depth = 0i32;
    let mut fn_brace_start = 0i32;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // Detect function definition (same heuristic as above)
        let is_fn_open = trimmed.ends_with('{')
            && trimmed.contains('(')
            && trimmed.contains(')')
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("switch ")
            && !trimmed.starts_with("//")
            && !trimmed.is_empty();

        if is_fn_open && current.is_none() {
            let paren = trimmed.find('(').unwrap_or(trimmed.len());
            let before = &trimmed[..paren];
            let name = before.split_whitespace().last().unwrap_or("unknown").to_string();
            fn_brace_start = brace_depth;
            current = Some(FnState {
                name,
                start_line: line_num,
                line_count: 0,
                branch_count: 0,
                nesting_depth: 0,
                current_depth: 0,
                body_tokens: 0,
            });
        }

        if let Some(ref mut st) = current {
            st.line_count += 1;
            st.body_tokens += trimmed.split_whitespace().count();

            // Branch keywords
            for kw in &["if (", "else if", "for (", "while (", "switch (", "case "] {
                if trimmed.contains(kw) {
                    st.branch_count += 1;
                }
            }
        }

        // Track brace depth
        for ch in trimmed.chars() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    if let Some(ref mut st) = current {
                        st.current_depth = (brace_depth - fn_brace_start) as usize;
                        if st.current_depth > st.nesting_depth {
                            st.nesting_depth = st.current_depth;
                        }
                    }
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth <= fn_brace_start {
                        if let Some(st) = current.take() {
                            emit_metric_obs(filepath, &st, obs);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(st) = current {
        emit_metric_obs(filepath, &st, obs);
    }
}

fn emit_metric_obs(filepath: &str, st: &FnState, obs: &mut Vec<Observation>) {
    if st.line_count > 80 {
        obs.push(Observation {
            file_path: filepath.to_string(),
            start_byte: 0,
            end_byte: 0,
            line: st.start_line,
            column: 1,
            kind: "c_fn_metric".to_string(),
            construct: st.name.clone(),
            context: format!("lines={}", st.line_count),
            message: format!("Function '{}' exceeds 80-line threshold ({} lines)", st.name, st.line_count),
        });
    }
    if st.branch_count > 10 {
        obs.push(Observation {
            file_path: filepath.to_string(),
            start_byte: 0,
            end_byte: 0,
            line: st.start_line,
            column: 1,
            kind: "c_fn_metric".to_string(),
            construct: st.name.clone(),
            context: format!("cyclomatic={}", st.branch_count),
            message: format!("Function '{}' has high cyclomatic complexity ({} branches)", st.name, st.branch_count),
        });
    }
    if st.nesting_depth > 4 {
        obs.push(Observation {
            file_path: filepath.to_string(),
            start_byte: 0,
            end_byte: 0,
            line: st.start_line,
            column: 1,
            kind: "c_fn_metric".to_string(),
            construct: st.name.clone(),
            context: format!("nesting={}", st.nesting_depth),
            message: format!("Function '{}' exceeds nesting depth threshold ({} levels)", st.name, st.nesting_depth),
        });
    }
}

pub fn parse_c_source(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    detect_oracle_floats(filepath, content, &mut obs);
    detect_victory_comments(filepath, content, &mut obs);
    detect_todo_comments(filepath, content, &mut obs);
    detect_unsafe_fns(filepath, content, &mut obs);
    detect_debug_artifacts(filepath, content, &mut obs);
    detect_getenv(filepath, content, &mut obs);
    detect_malloc_without_check(filepath, content, &mut obs);
    detect_hardcoded_lookup_tables(filepath, content, &mut obs);
    detect_stub_functions(filepath, content, &mut obs);
    collect_fn_metrics(filepath, content, &mut obs);
    obs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_oracle_float() {
        let obs = parse_c_source("foo.c", "double x = 0.284171835; // answer");
        assert!(obs.iter().any(|o| o.kind == "oracle_float"));
    }

    #[test]
    fn detects_unsafe_strcpy() {
        let obs = parse_c_source("foo.c", "strcpy(dest, src);");
        assert!(obs.iter().any(|o| o.construct == "strcpy"));
    }

    #[test]
    fn detects_todo_comment() {
        let obs = parse_c_source("foo.c", "// TODO: implement this");
        assert!(obs.iter().any(|o| o.construct == "TODO"));
    }

    #[test]
    fn detects_stub_function() {
        let src = "int compute(int x) {\n    return 0;\n}";
        let obs = parse_c_source("foo.c", src);
        assert!(obs.iter().any(|o| o.kind == "c_stub"));
    }

    #[test]
    fn detects_unchecked_malloc() {
        let src = "void *p = malloc(64);\nmemcpy(p, src, 64);";
        let obs = parse_c_source("foo.c", src);
        assert!(obs.iter().any(|o| o.construct == "malloc_unchecked"));
    }

    #[test]
    fn detects_getenv_oracle() {
        let obs = parse_c_source("main.c", "char *val = getenv(\"SECRET\");");
        assert!(obs.iter().any(|o| o.construct == "getenv"));
    }

    #[test]
    fn clean_c_has_no_stubs() {
        let src = "int add(int a, int b) {\n    return a + b;\n}";
        let obs = parse_c_source("math.c", src);
        assert!(!obs.iter().any(|o| o.kind == "c_stub"));
    }
}
