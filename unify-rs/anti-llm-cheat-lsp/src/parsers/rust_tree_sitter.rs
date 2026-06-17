use crate::observations::Observation;
use super::common;

const STUB_MACROS: &[&str] = &["todo!(", "unimplemented!(", "unreachable!("];
const DEBUG_MACROS: &[&str] = &["println!(", "eprintln!(", "dbg!(", "print!("];
const ALLOW_SUPPRESSION: &[&str] = &[
    "#[allow(dead_code)]",
    "#[allow(unused)]",
    "#[allow(unused_variables)]",
    "#[allow(unused_imports)]",
    "#[allow(unused_mut)]",
    "#[allow(clippy::",
];

fn is_test_path(path: &str) -> bool {
    path.contains("tests/") || path.ends_with("_test.rs") || path.contains("/test/")
}

#[derive(Default)]
struct FnMetrics {
    name: String,
    start_line: usize,
    line_count: usize,
    branch_count: usize,
    nesting_depth: usize,
    literal_count: usize,
    total_tokens: usize,
    callees: Vec<String>,
}

fn detect_stub_patterns(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    let in_test = is_test_path(filepath);

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // todo!/unimplemented!/unreachable! macro stubs
        for stub in STUB_MACROS {
            if trimmed.contains(stub) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "rust_stub".to_string(),
                    construct: stub.trim_end_matches('(').to_string(),
                    context: trimmed.to_string(),
                    message: format!("Stub macro '{}' — function not implemented", stub.trim_end_matches('(')),
                });
            }
        }

        // Debug print macros outside test files
        if !in_test {
            for mac in DEBUG_MACROS {
                if trimmed.contains(mac) && !trimmed.trim_start().starts_with("//") {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: line_num,
                        column: 1,
                        kind: "rust_debug_artifact".to_string(),
                        construct: mac.trim_end_matches('(').to_string(),
                        context: trimmed.to_string(),
                        message: format!("Debug macro '{}' left in production code", mac.trim_end_matches('(')),
                    });
                }
            }
        }

        // #[allow(...)] suppression attributes
        for attr in ALLOW_SUPPRESSION {
            if trimmed.starts_with(attr) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "rust_suppression".to_string(),
                    construct: "allow_attr".to_string(),
                    context: trimmed.to_string(),
                    message: format!("Suppression attribute '{}' silences real warnings", attr),
                });
            }
        }

        // TODO/FIXME/HACK/STUB comments
        let comment_upper = trimmed.to_uppercase();
        if trimmed.starts_with("//") {
            for marker in &["TODO", "FIXME", "HACK", "STUB", "XXX"] {
                if comment_upper.contains(marker) {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: line_num,
                        column: 1,
                        kind: "rust_todo_comment".to_string(),
                        construct: marker.to_string(),
                        context: trimmed.to_string(),
                        message: format!("Unresolved '{}' comment — placeholder left in code", marker),
                    });
                    break;
                }
            }
        }

        // Catch-all wildcard arms that do nothing: `_ => {}`  or `_ => ()`
        if (trimmed == "_ => {}" || trimmed == "_ => ()") && !in_test {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "rust_stub".to_string(),
                construct: "empty_wildcard_arm".to_string(),
                context: trimmed.to_string(),
                message: "Empty catch-all match arm `_ => {}` silently swallows unhandled cases".to_string(),
            });
        }
    }

    // Detect single-line stub functions: `fn foo(...) { <literal> }` or `fn foo() {}`
    detect_stub_functions(filepath, content, obs);
}

fn detect_stub_functions(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    // Match single-line fn bodies that contain only a literal return, or are empty
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn "))
        {
            continue;
        }
        // Single-line function: everything on one line
        if trimmed.contains('{') && trimmed.ends_with('}') {
            let body_start = trimmed.find('{').unwrap_or(0) + 1;
            let body = trimmed[body_start..trimmed.len() - 1].trim();
            let is_stub = body.is_empty()
                || body == "Ok(())"
                || body == "Err(())"
                || body.starts_with("panic!(")
                || body.starts_with("todo!(")
                || body.starts_with("unimplemented!(")
                || (body.starts_with('"') && body.ends_with('"'))
                || body.parse::<i64>().is_ok()
                || body.parse::<f64>().is_ok()
                || body == "true"
                || body == "false"
                || body == "None"
                || body == "vec![]"
                || body == "String::new()";
            if is_stub {
                let name_start = trimmed.find("fn ").map(|p| p + 3).unwrap_or(0);
                let name_end = trimmed[name_start..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(trimmed[name_start..].len());
                let name = &trimmed[name_start..name_start + name_end];
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: i + 1,
                    column: 1,
                    kind: "rust_stub".to_string(),
                    construct: "stub_function".to_string(),
                    context: trimmed.to_string(),
                    message: format!("Function '{}' is a single-line stub returning a constant or empty body", name),
                });
            }
        }
    }
}

fn detect_risky_patterns(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        if trimmed.contains("mem::transmute") {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "risky_pattern".to_string(),
                construct: "mem::transmute".to_string(),
                context: trimmed.to_string(),
                message: "Unsafe mem::transmute usage detected".to_string(),
            });
        }

        if trimmed.contains("std::env::var") || trimmed.contains("env::var(") {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "risky_pattern".to_string(),
                construct: "env_var_read".to_string(),
                context: trimmed.to_string(),
                message: "Environment variable read in production path detected".to_string(),
            });
        }

        if trimmed.contains("lazy_static!") || trimmed.contains("once_cell") {
            if trimmed.contains("env::var") || trimmed.contains("std::env") {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "risky_pattern".to_string(),
                    construct: "lazy_static_env".to_string(),
                    context: trimmed.to_string(),
                    message: "lazy_static/once_cell initialization from env var detected".to_string(),
                });
            }
        }

        if trimmed.starts_with("unsafe ") || trimmed.contains("unsafe {") {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "risky_pattern".to_string(),
                construct: "unsafe_block".to_string(),
                context: trimmed.to_string(),
                message: "Unsafe block or function detected".to_string(),
            });
        }

        if trimmed.contains(".unwrap()") || trimmed.contains("panic!(") {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "risky_pattern".to_string(),
                construct: "panic_unwrap".to_string(),
                context: trimmed.to_string(),
                message: "Panic/unwrap call detected".to_string(),
            });
        }

        if trimmed.contains("fs::write")
            || trimmed.contains("fs::remove")
            || trimmed.contains("File::create")
        {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: line_num,
                column: 1,
                kind: "risky_pattern".to_string(),
                construct: "fs_mutation".to_string(),
                context: trimmed.to_string(),
                message: "File system mutation detected".to_string(),
            });
        }
    }
}

fn collect_fn_metrics(content: &str) -> Vec<FnMetrics> {
    let mut metrics = Vec::new();
    let mut current: Option<FnMetrics> = None;
    let mut brace_depth = 0i32;
    let mut fn_start_depth = 0i32;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        if trimmed.starts_with("pub fn ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            if let Some(m) = current.take() {
                metrics.push(m);
            }
            let name_start = trimmed.find("fn ").map(|i| i + 3).unwrap_or(0);
            let name_end = trimmed[name_start..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(trimmed[name_start..].len());
            let name = trimmed[name_start..name_start + name_end].to_string();
            fn_start_depth = brace_depth;
            current = Some(FnMetrics {
                name,
                start_line: line_num,
                ..Default::default()
            });
        }

        if let Some(ref mut m) = current {
            m.line_count += 1;
            if trimmed.contains("if ") || trimmed.contains("match ") || trimmed.contains("while ") {
                m.branch_count += 1;
            }
            m.literal_count += trimmed.matches('"').count() / 2;
            m.total_tokens += trimmed.split_whitespace().count();
            for word in trimmed.split_whitespace() {
                if word.ends_with("()") || word.ends_with("(") {
                    let callee = word.trim_end_matches('(').trim_end_matches(')');
                    if !callee.is_empty() && callee != m.name {
                        m.callees.push(callee.to_string());
                    }
                }
            }
        }

        // String-aware brace depth tracking
        common::for_effective_braces(trimmed, |ch| {
            match ch {
                '{' => {
                    brace_depth += 1;
                    if let Some(ref mut m) = current {
                        let depth = (brace_depth - fn_start_depth) as usize;
                        if depth > m.nesting_depth {
                            m.nesting_depth = depth;
                        }
                    }
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth <= fn_start_depth {
                        if let Some(m) = current.take() {
                            metrics.push(m);
                        }
                    }
                }
                _ => {}
            }
        });
    }

    if let Some(m) = current {
        metrics.push(m);
    }

    metrics
}

fn check_fn_thresholds(
    filepath: &str,
    metrics: &[FnMetrics],
    obs: &mut Vec<Observation>,
) {
    for m in metrics {
        if m.line_count > 80 {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: m.start_line,
                column: 1,
                kind: "fn_metric".to_string(),
                construct: m.name.clone(),
                context: format!("lines={}", m.line_count),
                message: format!(
                    "Function '{}' exceeds 80-line threshold ({} lines)",
                    m.name, m.line_count
                ),
            });
        }

        if m.branch_count > 10 {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: m.start_line,
                column: 1,
                kind: "fn_metric".to_string(),
                construct: m.name.clone(),
                context: format!("cyclomatic={}", m.branch_count),
                message: format!(
                    "Function '{}' exceeds cyclomatic complexity threshold ({} branches)",
                    m.name, m.branch_count
                ),
            });
        }

        if m.nesting_depth > 4 {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: 0,
                end_byte: 0,
                line: m.start_line,
                column: 1,
                kind: "fn_metric".to_string(),
                construct: m.name.clone(),
                context: format!("nesting={}", m.nesting_depth),
                message: format!(
                    "Function '{}' exceeds nesting depth threshold ({} levels)",
                    m.name, m.nesting_depth
                ),
            });
        }
    }
}

pub fn parse_rust_source(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    common::detect_oracle_floats(filepath, content, &mut obs);
    detect_risky_patterns(filepath, content, &mut obs);
    detect_stub_patterns(filepath, content, &mut obs);
    let metrics = collect_fn_metrics(content);
    check_fn_thresholds(filepath, &metrics, &mut obs);
    obs
}
