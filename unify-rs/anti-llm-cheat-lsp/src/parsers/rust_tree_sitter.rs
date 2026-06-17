use crate::observations::Observation;

/// Known oracle float ranges derived from published papers.
/// Presence of these literals may indicate answer injection.
const ORACLE_FLOATS: &[(&str, &str)] = &[
    ("0.284171835", "Pearl 1988"),
    ("0.577350269", "sqrt(1/3) oracle"),
    ("1.618033988", "phi oracle"),
    ("2.718281828", "e oracle"),
    ("3.141592653", "pi oracle"),
];

#[derive(Default)]
struct FnMetrics {
    name: String,
    line_count: usize,
    branch_count: usize,
    nesting_depth: usize,
    literal_count: usize,
    total_tokens: usize,
    callees: Vec<String>,
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
                    context: line.to_string(),
                    message: format!(
                        "Oracle float literal '{}' detected (source: {}) — potential answer injection",
                        float_str, source
                    ),
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

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect function start
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
                ..Default::default()
            });
        }

        if let Some(ref mut m) = current {
            m.line_count += 1;

            // Count branch keywords
            if trimmed.contains("if ") || trimmed.contains("match ") || trimmed.contains("while ")
            {
                m.branch_count += 1;
            }

            // Rough literal count
            m.literal_count += trimmed.matches('"').count() / 2;
            m.total_tokens += trimmed.split_whitespace().count();

            // Track callees (very rough)
            for word in trimmed.split_whitespace() {
                if word.ends_with("()") || word.ends_with("(") {
                    let callee = word.trim_end_matches('(').trim_end_matches(')');
                    if !callee.is_empty() && callee != m.name {
                        m.callees.push(callee.to_string());
                    }
                }
            }
        }

        // Track brace depth for nesting
        for ch in trimmed.chars() {
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
        }
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
                line: 1,
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
                line: 1,
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
                line: 1,
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
    detect_oracle_floats(filepath, content, &mut obs);
    detect_risky_patterns(filepath, content, &mut obs);
    let metrics = collect_fn_metrics(content);
    check_fn_thresholds(filepath, &metrics, &mut obs);
    obs
}
