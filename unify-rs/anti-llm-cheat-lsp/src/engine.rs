use crate::config::AntiLlmConfig;
use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;
use crate::parsers::{
    c as c_parser, cargo_lock, cargo_toml, contract, fitness_report, ggen_toml, json_rpc,
    markdown_claims, receipt_json, refgraph, rust_tree_sitter, tera_template, typescript,
};
use crate::rules::{
    authority, claims, complexity, contract as contract_rules, determinism, ggen, lsp318, mutation,
    ocel_rules, oracle, receipts, refgraph as refgraph_rules, routes, rust_smells, surface, test,
    trace, typescript as ts_rules, version,
};
use aho_corasick::AhoCorasick;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

fn build_line_index(content: &[u8]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(content.len() / 40 + 1);
    offsets.push(0);
    for pos in memchr::memchr_iter(b'\n', content) {
        offsets.push(pos + 1);
    }
    offsets
}

fn byte_to_line(line_index: &[usize], byte_offset: usize) -> usize {
    match line_index.partition_point(|&start| start <= byte_offset) {
        0 => 1,
        n => n,
    }
}

const RAW_SMELL_PATTERNS: &[&str] = &[
    "tower-lsp",
    "tower_lsp",
    "CLAP",
    "Routing to PackPlan",
    "test result: ok",
    "v1.0.0",
    "version = \"1.0.0\"",
    "CLAP-DEBUG",
    "CLAP-DEBUG-PATH",
    "Content was:",
    "Path was:",
    "static scan as route proof",
    "static scan",
    "route proof",
    "ChangelogCoverage(15 rows) => SpecCoverage(LSP 3.18)",
    "ChangelogCoverage(15 rows) \u{21d2} SpecCoverage(LSP 3.18)",
    "15-row changelog matrix is being treated as full LSP 3.18 combinatorial coverage",
    "ANTI-LLM-OCEL-001-TRIGGER",
    "ANTI-LLM-OCEL-002-TRIGGER",
    "\"bypassed_compat\": true",
    "use wasm4pm::",
];

fn raw_smell_ac() -> &'static AhoCorasick {
    static AC: OnceLock<AhoCorasick> = OnceLock::new();
    AC.get_or_init(|| {
        aho_corasick::AhoCorasickBuilder::new()
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .build(RAW_SMELL_PATTERNS)
            .unwrap()
    })
}

fn classify_contains(line: &str) -> &'static str {
    let Some(pos) = line.find(".contains(") else {
        return "assert_contains";
    };
    let after = line[pos + ".contains(".len()..].trim_start();
    if after.starts_with('"') || after.starts_with("r\"") || after.starts_with("r#\"") {
        "assert_contains_string"
    } else if after.starts_with('&') || after.starts_with("&&") {
        "assert_contains_structural"
    } else if after.starts_with("format!") || after.starts_with("&format!") {
        "assert_contains_string"
    } else {
        "assert_contains"
    }
}

pub fn scan_file(filepath: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    let path = Path::new(filepath);
    if !path.is_file() {
        return obs;
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return obs,
    };
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default();

    let is_self_excluded = filepath.ends_with("src/rules/lsp318.rs")
        || filepath.ends_with("src/engine.rs")
        || filepath.ends_with("rules/lsp318.rs")
        || filepath.ends_with("engine.rs");

    if !is_self_excluded {
        let line_index = build_line_index(content.as_bytes());
        for mat in raw_smell_ac().find_iter(&content) {
            let pattern_idx = mat.pattern().as_usize();
            let smell = RAW_SMELL_PATTERNS[pattern_idx];
            let idx = mat.start();
            if pattern_idx == 0 || pattern_idx == 1 {
                let suffix = &content[idx + smell.len()..];
                if suffix.starts_with("-max") || suffix.starts_with("_max") || suffix.starts_with("::max") {
                    continue;
                }
            }
            let line_count = byte_to_line(&line_index, idx);
            obs.push(Observation {
                file_path: filepath.to_string(),
                start_byte: idx,
                end_byte: idx + smell.len(),
                line: line_count,
                column: 1,
                kind: "raw_text".to_string(),
                construct: smell.to_string(),
                context: smell.to_string(),
                message: format!("Raw text pattern '{}' detected", smell),
            });
        }
    }

    if !is_self_excluded {
        obs.extend(claims::scan_for_victory(filepath, &content, "raw_text", &[]));
    }

    let is_test_file = filepath.contains("tests/") || filepath.ends_with("_test.rs") || filepath.contains("/test/");
    if is_test_file {
        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            if line.contains("assert") && line.contains(".contains") {
                let construct = classify_contains(line);
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "test_smell".to_string(),
                    construct: construct.to_string(),
                    context: line.to_string(),
                    message: format!(".contains() assertion classified as '{}'", construct),
                });
            }
            if !filepath.contains("dogfood.rs") && line.contains("negative_controls") {
                obs.push(Observation {
                    file_path: filepath.to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    line: line_num,
                    column: 1,
                    kind: "test_smell".to_string(),
                    construct: "negative_control_reference".to_string(),
                    context: line.to_string(),
                    message: "Standard test references negative controls directory".to_string(),
                });
            }
        }
    }

    if filename == "Cargo.toml" {
        obs.extend(cargo_toml::parse_cargo_toml(filepath, &content));
    } else if filename == "Cargo.lock" {
        obs.extend(cargo_lock::parse_cargo_lock(filepath, &content));
    } else if filename.ends_with(".rs") {
        obs.extend(rust_tree_sitter::parse_rust_source(filepath, &content));
    } else if filename.ends_with(".md") {
        obs.extend(markdown_claims::parse_markdown_claims(filepath, &content));
    } else if (filename.ends_with(".json") || filename.ends_with(".jsonl")) && filepath.contains("transcripts") {
        obs.extend(json_rpc::parse_json_rpc_transcript(filepath, &content));
    } else if filename.ends_with(".json") && filepath.contains("receipts") {
        obs.extend(receipt_json::parse_receipt_json(filepath, &content));
    } else if filename.ends_with(".c") || filename.ends_with(".h") || filename.ends_with(".cpp") || filename.ends_with(".cc") {
        obs.extend(c_parser::parse_c_source(filepath, &content));
    } else if filename.ends_with(".ts") || filename.ends_with(".tsx") || filename.ends_with(".js") || filename.ends_with(".jsx") || filename.ends_with(".mjs") || filename.ends_with(".cjs") {
        obs.extend(typescript::parse_typescript(filepath, &content));
    } else if filename == "ggen.toml" {
        obs.extend(ggen_toml::parse_ggen_toml(filepath, &content));
    } else if filename.ends_with(".tera") {
        obs.extend(tera_template::parse_tera_template(filepath, &content));
    } else if filename.ends_with(".json") && (filepath.contains("ocel/reports") || filepath.contains("fitness_reports")) {
        obs.extend(fitness_report::parse_fitness_report(filepath, &content));
    } else if filename.ends_with(".json") && filepath.contains("refgraph") {
        obs.extend(refgraph::parse_refgraph_json(filepath, &content));
    } else if filename.ends_with(".json") && filepath.contains("contract") {
        obs.extend(contract::parse_contract_json(filepath, &content));
    }

    obs
}

pub fn scan_directory(dirpath: &str) -> Vec<Observation> {
    let mut obs = Vec::new();
    let path = Path::new(dirpath);
    if !path.is_dir() {
        return obs;
    }
    let walker = ignore::WalkBuilder::new(path)
        .hidden(false)
        .add_custom_ignore_filename(".anti-llm-ignore")
        .filter_entry(|e| e.file_name().to_string_lossy() != "fixtures")
        .build();
    for entry in walker.flatten() {
        if entry.path().is_file() {
            obs.extend(scan_file(&entry.path().to_string_lossy()));
        }
    }
    obs.extend(ggen_toml::detect_competing_authority(&[]));
    obs.extend(contract::detect_contract_schism(&obs.clone()));
    obs.extend(refgraph::detect_transitive_failset(&obs.clone()));
    obs
}

pub fn evaluate_diagnostics(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    evaluate_diagnostics_with_config(obs, &AntiLlmConfig::default())
}

pub fn evaluate_diagnostics_with_config(obs: &[Observation], config: &AntiLlmConfig) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();
    diags.extend(surface::evaluate(obs, config));
    diags.extend(authority::evaluate(obs));
    diags.extend(receipts::evaluate(obs));
    diags.extend(routes::evaluate(obs));
    diags.extend(mutation::evaluate(obs));
    diags.extend(version::evaluate(obs));
    diags.extend(test::evaluate(obs));
    diags.extend(rust_smells::evaluate(obs));
    diags.extend(determinism::evaluate(obs));
    diags.extend(lsp318::evaluate(obs));
    diags.extend(ocel_rules::evaluate(obs));
    diags.extend(ts_rules::evaluate(obs));
    diags.extend(ggen::evaluate(obs));
    diags.extend(complexity::evaluate(obs));
    diags.extend(oracle::evaluate(obs));
    diags.extend(trace::evaluate(obs));
    diags.extend(contract_rules::evaluate(obs));
    diags.extend(refgraph_rules::evaluate(obs));
    let has_non_victory_errors = diags.iter().any(|d| d.code != "ANTI-LLM-CLAIM-004");
    diags.extend(claims::evaluate(obs, &config.claim.domain_terms, has_non_victory_errors));
    let mut seen = std::collections::HashSet::new();
    diags.retain(|d| seen.insert((d.file_path.clone(), d.line, d.code.clone())));
    diags
}
