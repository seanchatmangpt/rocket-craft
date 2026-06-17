use crate::observations::Observation;
use regex::Regex;
use std::sync::OnceLock;

// ── Compiled-once regex statics ───────────────────────────────────────────────

fn ts_ignore_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"//\s*@ts-ignore|//\s*@ts-nocheck|/\*\s*eslint-disable").unwrap())
}

fn as_any_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bas\s+any\b").unwrap())
}

fn todo_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"//.*\b(TODO|FIXME|HACK|STUB|XXX)\b|\bunimplemented\b").unwrap())
}

fn claims_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(done|complete|fully\s+covered|production\s+ready|all\s+fixed|victory|fully\s+admitted|victory\s+confirmed)\b").unwrap()
    })
}

fn naming_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b(nitro\s*lsp)\b").unwrap())
}

fn vocab_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(GALL|checkpoint|failset|residual|andon|receipt|candidate|blocked|accepted|private\s+doctrine|internal\s+IP)\b").unwrap()
    })
}

// JS/TS-specific smell regexes
fn console_log_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bconsole\.(log|warn|error|debug|info)\s*\(").unwrap())
}

fn eval_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\beval\s*\(|new\s+Function\s*\(").unwrap())
}

fn var_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*var\s+\w").unwrap())
}

fn settimeout_hack_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bsetTimeout\s*\(.+,\s*0\s*\)").unwrap())
}

fn json_clone_hack_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"JSON\.parse\s*\(\s*JSON\.stringify\s*\(").unwrap())
}

fn hardcoded_base64_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Long base64-looking string literal (40+ chars of base64 alphabet)
    RE.get_or_init(|| Regex::new(r#"["'][A-Za-z0-9+/=]{40,}["']"#).unwrap())
}

fn promise_no_catch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\.then\s*\([^)]+\)\s*;").unwrap())
}

fn oracle_float_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"0\.284171835|0\.577350269|1\.618033988|2\.718281828|3\.141592653").unwrap())
}

fn stub_fn_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Arrow or regular function that returns only a literal: () => 0  /  function foo() { return null; }
    RE.get_or_init(|| {
        Regex::new(r#"=>\s*(null|undefined|false|true|0|''|""|``)(\s*[;,]|$)|\{\s*return\s+(null|undefined|false|true|0|''|"")\s*;\s*\}"#).unwrap()
    })
}

fn is_js_test_path(filepath: &str) -> bool {
    filepath.contains(".test.") || filepath.contains(".spec.")
        || filepath.contains("/test/") || filepath.contains("/tests/")
        || filepath.contains("__tests__")
        || filepath.contains("fixtures/")
}

// ── Parser ────────────────────────────────────────────────────────────────────

pub fn parse_typescript(filepath: &str, content: &str) -> Vec<Observation> {
    let mut obs = Vec::new();

    let is_whitelisted = filepath.contains("diagnostics.ts")
        || filepath.contains("fixtures/")
        || filepath.contains("test/")
        || filepath.contains("tests/");

    let in_test = is_js_test_path(filepath);
    let is_js = filepath.ends_with(".js") || filepath.ends_with(".jsx")
        || filepath.ends_with(".mjs") || filepath.ends_with(".cjs");

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Skip comment-only lines for most checks
        let is_comment = trimmed.starts_with("//") || trimmed.starts_with("*") || trimmed.starts_with("/*");

        if let Some(mat) = ts_ignore_re().find(line) {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: mat.start(),
                end_byte: mat.end(),
                line: line_num,
                column: mat.start() + 1,
                kind: "ts_smell".to_string(),
                construct: "ts-ignore".to_string(),
                context: line.to_string(),
                message: "TypeScript ignore or ESLint disable comment detected".to_string(),
            });
        }

        if let Some(mat) = as_any_re().find(line) {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: mat.start(),
                end_byte: mat.end(),
                line: line_num,
                column: mat.start() + 1,
                kind: "ts_smell".to_string(),
                construct: "as any".to_string(),
                context: line.to_string(),
                message: "Unsafe type cast 'as any' detected (type laundering)".to_string(),
            });
        }

        if let Some(mat) = todo_re().find(line) {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: mat.start(),
                end_byte: mat.end(),
                line: line_num,
                column: mat.start() + 1,
                kind: "ts_smell".to_string(),
                construct: mat.as_str().to_string(),
                context: line.to_string(),
                message: format!(
                    "Unimplemented stub or placeholder '{}' detected",
                    mat.as_str()
                ),
            });
        }

        if let Some(mat) = claims_re().find(line) {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: mat.start(),
                end_byte: mat.end(),
                line: line_num,
                column: mat.start() + 1,
                kind: "ts_claim".to_string(),
                construct: mat.as_str().to_string(),
                context: line.to_string(),
                message: format!(
                    "Forbidden claim word '{}' found on TS surface",
                    mat.as_str()
                ),
            });
        }

        if let Some(mat) = naming_re().find(line) {
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: mat.start(),
                end_byte: mat.end(),
                line: line_num,
                column: mat.start() + 1,
                kind: "ts_leak".to_string(),
                construct: mat.as_str().to_string(),
                context: line.to_string(),
                message: format!(
                    "Naming Fence violation: Unauthorized name '{}' detected",
                    mat.as_str()
                ),
            });
        }

        if !is_whitelisted {
            if let Some(mat) = vocab_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "ts_leak".to_string(),
                    construct: mat.as_str().to_string(),
                    context: line.to_string(),
                    message: format!(
                        "Scope Fence violation: Leaked internal term '{}'",
                        mat.as_str()
                    ),
                });
            }
        }

        // ── JS/TS extended cheat detection ────────────────────────────────────

        // console.log/warn/error debug artifacts (skip test files)
        if !in_test && !is_comment {
            if let Some(mat) = console_log_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_debug_artifact".to_string(),
                    construct: "console.log".to_string(),
                    context: trimmed.to_string(),
                    message: "console.log/warn/error debug artifact left in production code".to_string(),
                });
            }
        }

        // eval() / new Function() — dynamic execution oracle
        if !is_comment {
            if let Some(mat) = eval_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_oracle".to_string(),
                    construct: "eval".to_string(),
                    context: trimmed.to_string(),
                    message: "eval() or new Function() — dynamic code execution, potential oracle injection channel".to_string(),
                });
            }
        }

        // var declarations (legacy JS — often signals copy-pasted or LLM-stubbed code)
        if is_js && !is_comment {
            if let Some(mat) = var_decl_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_legacy".to_string(),
                    construct: "var".to_string(),
                    context: trimmed.to_string(),
                    message: "var declaration — use let/const; var signals legacy or stubbed code".to_string(),
                });
            }
        }

        // setTimeout(fn, 0) deferred execution hack
        if !is_comment {
            if let Some(mat) = settimeout_hack_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_hack".to_string(),
                    construct: "setTimeout(fn,0)".to_string(),
                    context: trimmed.to_string(),
                    message: "setTimeout(fn, 0) deferred execution hack — masks async ordering bugs".to_string(),
                });
            }
        }

        // JSON.parse(JSON.stringify(...)) deep clone hack
        if !is_comment {
            if let Some(mat) = json_clone_hack_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_hack".to_string(),
                    construct: "json_deep_clone".to_string(),
                    context: trimmed.to_string(),
                    message: "JSON.parse(JSON.stringify(...)) deep clone — loses prototypes/undefined; use structuredClone()".to_string(),
                });
            }
        }

        // Hardcoded long base64 string (potential secret or oracle value)
        if !is_comment && !in_test {
            if let Some(mat) = hardcoded_base64_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_secret".to_string(),
                    construct: "hardcoded_base64".to_string(),
                    context: trimmed.to_string(),
                    message: "Long base64-like string literal — potential hardcoded secret or oracle value".to_string(),
                });
            }
        }

        // .then() without .catch() (unhandled promise rejection)
        if !is_comment && !in_test {
            if let Some(mat) = promise_no_catch_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_unsafe".to_string(),
                    construct: "promise_no_catch".to_string(),
                    context: trimmed.to_string(),
                    message: ".then() chain without .catch() — unhandled promise rejection silently swallows errors".to_string(),
                });
            }
        }

        // Oracle float literals
        if !is_comment {
            if let Some(mat) = oracle_float_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "oracle_float".to_string(),
                    construct: mat.as_str().to_string(),
                    context: trimmed.to_string(),
                    message: format!("Oracle float literal '{}' — potential hardcoded answer injection", mat.as_str()),
                });
            }
        }

        // Stub arrow/regular functions returning only a constant literal
        if !is_comment {
            if let Some(mat) = stub_fn_re().find(line) {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: mat.start(),
                    end_byte: mat.end(),
                    line: line_num,
                    column: mat.start() + 1,
                    kind: "js_stub".to_string(),
                    construct: "stub_function".to_string(),
                    context: trimmed.to_string(),
                    message: "Function returns only a constant literal — likely a stub".to_string(),
                });
            }
        }
    }

    // Detect module-level hardcoded lookup objects (oracle memo tables)
    detect_hardcoded_objects(filepath, content, &mut obs);

    obs
}

fn detect_hardcoded_objects(filepath: &str, content: &str, obs: &mut Vec<Observation>) {
    // Scan for large object literals at module scope (const/let/var x = { ... })
    // Proxy: count lines between opening `= {` and closing `}` at same indent
    let lines: Vec<&str> = content.lines().collect();
    let mut depth = 0usize;
    let mut obj_start = 0usize;
    let mut obj_keys = 0usize;
    let mut in_obj = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !in_obj {
            // Module-scope assignment with object literal
            if (trimmed.starts_with("const ") || trimmed.starts_with("let ") || trimmed.starts_with("export const "))
                && trimmed.contains("= {")
            {
                in_obj = true;
                depth = 1;
                obj_start = i + 1;
                obj_keys = 0;
            }
        } else {
            for ch in trimmed.chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => { if depth > 0 { depth -= 1; } }
                    _ => {}
                }
            }
            // Count key-value pairs: lines containing `: ` or `:`
            if depth > 0 && trimmed.contains(':') && !trimmed.starts_with("//") {
                obj_keys += 1;
            }
            if depth == 0 {
                in_obj = false;
                if obj_keys > 15 {
                    obs.push(Observation {
                        file_path: filepath.to_string(),
                        start_byte: 0,
                        end_byte: 0,
                        line: obj_start,
                        column: 1,
                        kind: "js_oracle".to_string(),
                        construct: "hardcoded_lookup_object".to_string(),
                        context: format!("module-scope object ~{} keys", obj_keys),
                        message: format!(
                            "Module-scope object literal with ~{} entries — potential oracle lookup table",
                            obj_keys
                        ),
                    });
                }
                obj_keys = 0;
            }
        }
    }
}
