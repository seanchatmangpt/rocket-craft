use super::common;
use crate::observations::Observation;

const UNSAFE_STRING_FNS: &[&str] = &["strcpy(", "strcat(", "gets(", "sprintf(", "scanf("];
const DEBUG_CALLS: &[&str] = &["printf(", "fprintf(stderr", "fprintf(stdout"];

fn is_test_path(path: &str) -> bool {
    path.contains("test/")
        || path.contains("tests/")
        || path.ends_with("_test.c")
        || path.ends_with("_spec.c")
        || path.contains("check/")
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

/// True if the trimmed line looks like a C function-definition opener.
fn is_fn_open(trimmed: &str) -> bool {
    trimmed.ends_with('{')
        && trimmed.contains('(')
        && trimmed.contains(')')
        && !trimmed.starts_with("if ")
        && !trimmed.starts_with("} ")   // } else if / } else {
        && !trimmed.starts_with("else")
        && !trimmed.starts_with("for ")
        && !trimmed.starts_with("while ")
        && !trimmed.starts_with("switch ")
        && !trimmed.starts_with("do ")
        && !trimmed.starts_with("//")
        && !trimmed.starts_with('*')
        && !trimmed.starts_with('#')
        && !trimmed.is_empty()
}

/// Extract the function name before the first `(`, stripping leading type-qualifier chars
/// (e.g. `*` from pointer-returning functions like `int *compute(...)`).
fn extract_c_fn_name(trimmed: &str) -> &str {
    let paren = trimmed.find('(').unwrap_or(trimmed.len());
    let before = &trimmed[..paren];
    let raw = before.split_whitespace().last().unwrap_or("unknown");
    raw.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '_')
}

fn detect_victory_comments(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    const VICTORY_TERMS: &[&str] = &[
        "fully implemented",
        "complete",
        "works perfectly",
        "all done",
        "production ready",
        "verified correct",
        "tested and working",
    ];
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let lower = line.to_lowercase();
        // Include // inline comments, /* block openers, and * body lines inside /* */
        if !lower.contains("//") && !lower.contains("/*") && !trimmed.starts_with('*') {
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
                    context: trimmed.to_string(),
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
        if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with('*') {
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
                    message: format!(
                        "Unresolved '{}' comment — placeholder or stub left in code",
                        marker
                    ),
                });
                break;
            }
        }
    }
}

fn detect_unsafe_fns(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
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
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
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
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
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
                message: "getenv() in production code — environment oracle injection channel"
                    .to_string(),
            });
        }
    }
}

fn detect_malloc_without_check(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }
        if !trimmed.contains("malloc(")
            && !trimmed.contains("calloc(")
            && !trimmed.contains("realloc(")
        {
            continue;
        }
        // The current line may already be a conditional allocation: if ((p = malloc(n)) != NULL)
        let current_ok = trimmed.contains("!= NULL")
            || trimmed.contains("== NULL")
            || trimmed.contains("if (")
            || trimmed.contains("if(");
        let next_check = current_ok
            || lines.iter().skip(i + 1).take(3).any(|l| {
                let t = l.trim();
                t.starts_with("if")
                    || t.contains("== NULL")
                    || t.contains("!= NULL")
                    || t.starts_with("assert(")
                    || t.contains("NULL)")
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
    let mut in_array = false;
    let mut array_depth = 0usize;
    let mut array_start = 0usize;
    let mut entry_count = 0usize;
    let mut array_name = String::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }

        if !in_array {
            if (trimmed.contains("static") || trimmed.contains("const"))
                && trimmed.contains('[')
                && trimmed.contains(']')
                && trimmed.contains('=')
                && trimmed.contains('{')
            {
                in_array = true;
                array_depth = 0;
                array_start = line_idx + 1;
                entry_count = 0;
                // Use rfind on the part before '{' to avoid matching '=' inside array dimensions
                let brace_pos = trimmed.find('{').unwrap_or(trimmed.len());
                let before_brace = trimmed[..brace_pos].trim_end();
                let eq_pos = before_brace.rfind('=').unwrap_or(0);
                let before_eq = before_brace[..eq_pos].trim_end();
                array_name = before_eq
                    .split_whitespace()
                    .last()
                    .unwrap_or("unknown")
                    .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .trim_start_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_string();

                // Process braces on the opening line itself (handles single-line arrays)
                entry_count += trimmed.matches(',').count();
                common::for_effective_braces(trimmed, |ch| {
                    match ch {
                        '{' => array_depth += 1,
                        '}' => {
                            if array_depth > 0 {
                                array_depth -= 1;
                            }
                        }
                        _ => { /* handled */ }
                    }
                });
                if array_depth == 0 {
                    in_array = false;
                    if entry_count > 20 {
                        obs.push(make_lookup_obs(
                            filepath,
                            array_start,
                            &array_name,
                            entry_count,
                        ));
                    }
                    entry_count = 0;
                    array_name = String::new();
                    array_depth = 0;
                }
            }
        } else {
            entry_count += trimmed.matches(',').count();
            common::for_effective_braces(trimmed, |ch| {
                match ch {
                    '{' => array_depth += 1,
                    '}' => {
                        if array_depth > 0 {
                            array_depth -= 1;
                        }
                    }
                    _ => { /* handled */ }
                }
            });
            if array_depth == 0 {
                in_array = false;
                if entry_count > 20 {
                    obs.push(make_lookup_obs(
                        filepath,
                        array_start,
                        &array_name,
                        entry_count,
                    ));
                }
                entry_count = 0;
                array_name = String::new();
                array_depth = 0;
            }
        }
    }
}

fn make_lookup_obs(filepath: &str, line: usize, name: &str, count: usize) -> Observation {
    Observation {
        file_path: filepath.to_string(),
        start_byte: 0,
        end_byte: 0,
        line,
        column: 1,
        kind: "c_oracle".to_string(),
        construct: "large_static_array".to_string(),
        context: format!("array '{}' with ~{} entries", name, count),
        message: format!(
            "Large static array '{}' (~{} entries) — potential oracle lookup table",
            name, count
        ),
    }
}

fn detect_stub_functions(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        if is_fn_open(trimmed) {
            let mut depth = 1usize;
            let mut body_lines: Vec<&str> = Vec::new();
            let mut j = i + 1;
            while j < lines.len() && depth > 0 {
                let bl = lines[j].trim();
                common::for_effective_braces(bl, |ch| {
                    match ch {
                        '{' => depth += 1,
                        '}' => {
                            if depth > 0 {
                                depth -= 1;
                            }
                        }
                        _ => { /* handled */ }
                    }
                });
                if depth > 0 {
                    body_lines.push(bl);
                }
                j += 1;
            }

            let non_empty: Vec<&&str> = body_lines
                .iter()
                .filter(|l| !l.is_empty() && !l.starts_with("//"))
                .collect();
            let is_stub = non_empty.is_empty()
                || (non_empty.len() == 1
                    && (non_empty[0].starts_with("return 0")
                        || non_empty[0].starts_with("return NULL")
                        || non_empty[0].starts_with("return -1")
                        || non_empty[0].starts_with("return false")
                        || non_empty[0].starts_with("return true")
                        || *non_empty[0] == "return;"));

            if is_stub {
                let name = extract_c_fn_name(trimmed);
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

        if is_fn_open(trimmed) && current.is_none() {
            let name = extract_c_fn_name(trimmed).to_string();
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
            for kw in &["if (", "else if", "for (", "while (", "switch (", "case "] {
                if trimmed.contains(kw) {
                    st.branch_count += 1;
                }
            }
        }

        common::for_effective_braces(trimmed, |ch| {
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
                _ => { /* handled */ }
            }
        });
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
            message: format!(
                "Function '{}' exceeds 80-line threshold ({} lines)",
                st.name, st.line_count
            ),
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
            message: format!(
                "Function '{}' has high cyclomatic complexity ({} branches)",
                st.name, st.branch_count
            ),
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
            message: format!(
                "Function '{}' exceeds nesting depth threshold ({} levels)",
                st.name, st.nesting_depth
            ),
        });
    }
}

pub fn parse_c_source(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    common::detect_oracle_floats(filepath, content, &mut obs);
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

    #[test]
    fn else_if_not_misidentified_as_function() {
        let src = "void f(int x) {\n    if (x > 0) {\n        x = 1;\n    } else if (x < 0) {\n        x = -1;\n    }\n}";
        let obs = parse_c_source("foo.c", src);
        // f() returns nothing constant — not a stub. else-if must not be treated as a fn.
        assert!(!obs
            .iter()
            .any(|o| o.kind == "c_stub" && o.message.contains("else")));
    }

    #[test]
    fn nested_struct_array_detected() {
        // 21 comma-separated struct initializers — should exceed threshold
        let mut src = "static const Point pts[] = {\n".to_string();
        for i in 0..21 {
            src.push_str(&format!("    {{{}, {}}},\n", i, i + 1));
        }
        src.push_str("};\n");
        let obs = parse_c_source("lookup.c", &src);
        assert!(
            obs.iter().any(|o| o.construct == "large_static_array"),
            "should detect large nested struct array"
        );
    }

    #[test]
    fn victory_in_block_comment_body_detected() {
        let src =
            "/*\n * This is fully implemented and works perfectly.\n */\nint foo() { return 1; }";
        let obs = parse_c_source("foo.c", src);
        assert!(
            obs.iter().any(|o| o.kind == "c_claim"),
            "should detect victory language in block comment body"
        );
    }

    #[test]
    fn string_with_brace_does_not_confuse_stub_detection() {
        let src = "int f(int x) {\n    printf(\"hello {\");\n    return x + 1;\n}";
        let obs = parse_c_source("foo.c", src);
        assert!(
            !obs.iter().any(|o| o.kind == "c_stub"),
            "brace inside string should not confuse stub detection"
        );
    }
}
