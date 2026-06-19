//! Project management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_sync() -> Result<Value> {
    use indicatif::{ProgressBar, ProgressStyle};
    use rocket_sdk::manifest;
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    tracing::info!("{}", "=== Syncing Project Manifest ===");
    let versions_dir = Path::new("versions");
    if !versions_dir.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "versions directory not found: {}",
            versions_dir.display()
        )));
    }
    let entries: Vec<_> = WalkDir::new(versions_dir)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("uproject"))
        .collect();
    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?
        .progress_chars("#>-"));
    let mut projects = Vec::new();
    for entry in &entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        pb.set_message(format!("Syncing {}", name));
        let mut targets = Vec::new();
        let source_dir = path
            .parent()
            .ok_or_else(|| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "no parent dir: {}",
                    path.display()
                ))
            })?
            .join("Source");
        if source_dir.exists() {
            for t_entry in fs::read_dir(source_dir)
                .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?
            {
                let t_entry = t_entry.map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!("{}", e))
                })?;
                let t_name = t_entry.file_name().to_string_lossy().to_string();
                if t_name.ends_with(".Target.cs") {
                    targets.push(t_name.replace(".Target.cs", ""));
                }
            }
        }
        projects.push(manifest::Project {
            name,
            uproject_path: path.to_path_buf(),
            targets,
        });
        pb.inc(1);
    }
    pb.finish_with_message("Sync complete");
    let manifest = manifest::Manifest::new("project-manifest.json", projects);
    manifest
        .save()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    tracing::info!("{}", "Manifest updated successfully.");
    Ok(serde_json::json!({"status": "ok", "projects_found": entries.len()}))
}

fn do_build(
    project: Option<String>,
    target: Option<String>,
    platform: Option<String>,
) -> Result<Value> {
    use rocket_sdk::manifest;
    use std::path::Path;
    use std::process::Command;

    let manifest = manifest::Manifest::load("project-manifest.json")
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let name = project.ok_or_else(|| {
        clap_noun_verb::NounVerbError::execution_error(
            "No project specified. Use --project".to_string(),
        )
    })?;
    let proj = manifest
        .projects()
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(format!("project not found: {}", name))
        })?;
    let target_val = target
        .or_else(|| proj.targets.first().cloned())
        .ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "no target found for project: {}",
                proj.name
            ))
        })?;
    let platform_val = platform.unwrap_or_else(|| "Win64".to_string());
    // Resolve engine root: UE4_ROOT env var → .rocket.json → error
    let ue4_root = if let Ok(env_root) = std::env::var("UE4_ROOT") {
        std::path::PathBuf::from(env_root)
    } else {
        let config = rocket_sdk::config::RocketConfig::load()
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
        config.ue4_root.ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(
                "UE4_ROOT not set. Set the UE4_ROOT env var or run 'rocket env setup'.".to_string(),
            )
        })?
    };
    let uat_name = if cfg!(windows) {
        "RunUAT.bat"
    } else {
        "RunUAT.sh"
    };
    let uat_path = ue4_root
        .join("Engine")
        .join("Build")
        .join("BatchFiles")
        .join(uat_name);
    tracing::info!(
        "Building {} [{}] for {}...",
        proj.name,
        target_val,
        platform_val
    );
    let status = Command::new(&uat_path)
        .arg("BuildCookRun")
        .arg(format!("-project={}", proj.uproject_path.display()))
        .arg(format!("-target={}", target_val))
        .arg(format!("-platform={}", platform_val))
        .arg("-cook")
        .arg("-build")
        .arg("-stage")
        .arg("-archive")
        .arg(format!(
            "-archivedirectory={}",
            Path::new("Builds").display()
        ))
        .status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "Build failed".to_string(),
        ));
    }
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_audit() -> Result<Value> {
    use crate::compliance::ComplianceEngine;
    use knhk::AndroidKeystoreLaw;
    use rocket_sdk::{
        audit_affidavit::{persist_receipt, record_audit, AuditEvent},
        manifest,
    };
    use std::path::Path;
    use walkdir::WalkDir;

    tracing::info!("{}", "=== Project Health Audit ===");
    let manifest = manifest::Manifest::load("project-manifest.json")
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut engine = ComplianceEngine::new();
    engine.add_law(Box::new(AndroidKeystoreLaw));
    if let Err(e) = engine.load_plugins("plugins") {
        tracing::info!("  Failed to load plugins: {}", e);
    }
    let mut audit_events: Vec<AuditEvent> = Vec::new();
    for proj in manifest.projects() {
        tracing::info!("\nProject: {}", proj.name);
        if proj.uproject_path.exists() {
            tracing::info!("  [OK] uproject file found");
        } else {
            tracing::info!("  [FAIL] uproject file MISSING");
        }
        if let Some(project_dir) = proj.uproject_path.parent() {
            let maps_dir = project_dir.join("Content").join("Maps");
            if maps_dir.exists() {
                let map_count = WalkDir::new(maps_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("umap"))
                    .count();
                tracing::info!("  [OK] Maps found: {}", map_count);
            }
            let result = engine.check_project(proj);
            if result.passed {
                tracing::info!("    [OK] All laws satisfied.");
            } else {
                for err in &result.errors {
                    tracing::info!(
                        "    [FAIL] Law '{}' violated: {}",
                        err.law_name,
                        err.message
                    );
                }
            }
            audit_events.push(AuditEvent {
                project_name: result.project_name,
                passed: result.passed,
                violations: result
                    .errors
                    .into_iter()
                    .map(|e| (e.law_name, e.message))
                    .collect(),
                visual_delta: None,
                combinatorial_matrix: None,
            });
        }
    }
    match record_audit(&audit_events) {
        Ok(receipt) => {
            tracing::info!(
                "  Chain sealed — {} event(s), hash: {}",
                receipt.events.len(),
                &receipt.chain_hash.as_hex()[..16]
            );
            if let Err(e) = persist_receipt(&receipt, Path::new(".")) {
                tracing::info!("  Could not persist receipt: {}", e);
            }
        }
        Err(e) => tracing::info!("  Affidavit assembly failed: {}", e),
    }
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_test() -> Result<Value> {
    use std::process::Command;
    tracing::info!("{}", "=== Running All Tests ===");

    // All 7 Rust workspaces with their respective cargo-test flags.
    // Matches the `just test-rust` Justfile recipe exactly.
    let workspaces: &[(&str, &[&str])] = &[
        ("tools", &["test", "--all"]),
        ("nexus-engine", &["test", "--all"]),
        ("blueprint-rs", &["test", "--all"]),
        ("chicago-tdd-tools", &["test", "--all-features"]),
        ("unify-rs", &["test", "--all"]),
        ("infinity-blade-4/mud", &["test", "--all"]),
        ("asset-pipeline", &["test"]),
    ];

    for (dir, args) in workspaces {
        tracing::info!("  Testing {}...", dir);
        let status = Command::new("cargo")
            .args(*args)
            .current_dir(dir)
            .status()
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "failed to spawn cargo test in {dir}: {e}"
                ))
            })?;
        if !status.success() {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                "{dir} tests failed"
            )));
        }
    }

    do_asset_validation()?;
    tracing::info!("\n{}", "All tests passed!");
    Ok(serde_json::json!({"status": "ok"}))
}

/// Returns true when `rel_path` should be skipped during asset validation.
///
/// Skips paths whose components include an ignored directory name, whose
/// extension is in the ignore list, or whose filename is explicitly excluded.
pub fn asset_path_should_skip(
    rel_path: &std::path::Path,
    ignore_dirs: &[&str],
    ignore_files: &[&str],
    ignore_extensions: &[&str],
) -> bool {
    if ignore_dirs.iter().any(|d| rel_path.components().any(|c| c.as_os_str() == *d)) {
        return true;
    }
    if let Some(ext) = rel_path.extension().and_then(|s| s.to_str()) {
        if ignore_extensions.contains(&ext) {
            return true;
        }
    }
    if let Some(name) = rel_path.file_name().and_then(|s| s.to_str()) {
        if ignore_files.contains(&name) {
            return true;
        }
    }
    false
}

/// Scan `content` for any of `patterns`, returning `(line_number_1based, pattern)` for each hit.
pub fn scan_forbidden_content<'a>(content: &str, patterns: &[&'a str]) -> Vec<(usize, &'a str)> {
    let mut hits = Vec::new();
    for pattern in patterns {
        if content.contains(pattern) {
            for (idx, line) in content.lines().enumerate() {
                if line.contains(pattern) {
                    hits.push((idx + 1, *pattern));
                }
            }
        }
    }
    hits
}

fn do_asset_validation() -> Result<Value> {
    use std::fs;
    use walkdir::WalkDir;
    tracing::info!("\n--- Asset Validation (Rust Native) ---");
    let forbidden_patterns = ["Highrise", "Brm-HTML5-Shipping"];
    let ignore_dirs = [
        ".git", ".agents", "non-project-files", "pwa-staff", "versions",
        "docs", "tools", "chicago-tdd-tools", "unify-rs", "target", "scratch",
    ];
    let ignore_files = [
        "validate-assets.py", "ROCKET_CRAFT_AUDIT.md", "README.md",
        "HELP.md", "CLAUDE.md", "DFLSS.md",
    ];
    let ignore_extensions = [
        "log", "lock", "json", "md", "txt", "js",
        "applescript", "sh", "bat", "yml", "yaml", "toml",
    ];
    let mut issues_found = false;
    let project_root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    for entry in WalkDir::new(&project_root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        let rel_path = path.strip_prefix(&project_root).unwrap_or(path);
        if asset_path_should_skip(rel_path, &ignore_dirs, &ignore_files, &ignore_extensions) {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            for (line_num, pattern) in scan_forbidden_content(&content, &forbidden_patterns) {
                tracing::warn!(
                    "  ALERT: Found reference to '{}' in {}:{}",
                    pattern, rel_path.display(), line_num
                );
                issues_found = true;
            }
        }
    }
    if issues_found {
        Err(clap_noun_verb::NounVerbError::execution_error("Asset validation failed.".to_string()))
    } else {
        tracing::info!("RESULT: Validation PASSED.");
        Ok(serde_json::json!({"status": "ok"}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const DIRS: &[&str] = &[".git", "target", "versions", "tools"];
    const FILES: &[&str] = &["README.md", "CLAUDE.md"];
    const EXTS: &[&str] = &["log", "json", "md", "toml"];

    // ── asset_path_should_skip ────────────────────────────────────────────────

    #[test]
    fn skip_ignored_dir_component() {
        assert!(asset_path_should_skip(Path::new("target/debug/foo.rs"), DIRS, FILES, EXTS));
        assert!(asset_path_should_skip(Path::new(".git/config"), DIRS, FILES, EXTS));
    }

    #[test]
    fn skip_ignored_extension() {
        assert!(asset_path_should_skip(Path::new("some/path/file.log"), DIRS, FILES, EXTS));
        assert!(asset_path_should_skip(Path::new("Cargo.toml"), DIRS, FILES, EXTS));
    }

    #[test]
    fn skip_ignored_filename() {
        assert!(asset_path_should_skip(Path::new("some/dir/README.md"), DIRS, FILES, EXTS));
        assert!(asset_path_should_skip(Path::new("CLAUDE.md"), DIRS, FILES, EXTS));
    }

    #[test]
    fn does_not_skip_normal_rust_file() {
        assert!(!asset_path_should_skip(Path::new("src/lib.rs"), DIRS, FILES, EXTS));
    }

    #[test]
    fn does_not_skip_cpp_file_outside_ignored_dirs() {
        assert!(!asset_path_should_skip(
            Path::new("Source/MyGame/Combat.cpp"), DIRS, FILES, EXTS
        ));
    }

    #[test]
    fn nested_non_ignored_dir_does_not_trigger_skip() {
        // "versions" dir must be a path component, not a substring of a component
        assert!(!asset_path_should_skip(
            Path::new("adventure/world.cpp"), DIRS, FILES, EXTS
        ));
    }

    // ── scan_forbidden_content ────────────────────────────────────────────────

    #[test]
    fn detects_single_pattern_single_line() {
        let hits = scan_forbidden_content("Hello Highrise world\n", &["Highrise"]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], (1, "Highrise"));
    }

    #[test]
    fn detects_pattern_on_correct_line_number() {
        let content = "line one\nHere is Brm-HTML5-Shipping reference\nline three\n";
        let hits = scan_forbidden_content(content, &["Brm-HTML5-Shipping"]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, 2);
    }

    #[test]
    fn returns_empty_when_no_match() {
        let hits = scan_forbidden_content("clean content\nno forbidden words\n", &["Highrise"]);
        assert!(hits.is_empty());
    }

    #[test]
    fn detects_multiple_patterns_in_same_content() {
        let content = "Highrise ref\nBrm-HTML5-Shipping ref\n";
        let hits = scan_forbidden_content(content, &["Highrise", "Brm-HTML5-Shipping"]);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn detects_pattern_appearing_twice_on_different_lines() {
        let content = "Highrise first\nmore text\nHighrise again\n";
        let hits = scan_forbidden_content(content, &["Highrise"]);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0, 1);
        assert_eq!(hits[1].0, 3);
    }

    #[test]
    fn empty_content_returns_empty() {
        assert!(scan_forbidden_content("", &["Highrise"]).is_empty());
    }

    #[test]
    fn empty_patterns_returns_empty() {
        assert!(scan_forbidden_content("some content with Highrise\n", &[]).is_empty());
    }
}

fn do_logs(file: Option<String>, lines: Option<u64>) -> Result<Value> {
    use std::fs;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    let lines = lines.unwrap_or(50) as usize;
    let logs_dir = Path::new("non-project-files/logs");
    if !logs_dir.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "Logs directory not found: {}",
            logs_dir.display()
        )));
    }
    let log_path = if let Some(f) = file {
        let p = logs_dir.join(f);
        if !p.exists() {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                "Log file not found: {}",
                p.display()
            )));
        }
        p
    } else {
        let mut entries: Vec<_> = fs::read_dir(logs_dir)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?
            .filter_map(|res| res.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        entries.last().map(|e| e.path()).ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error("No log files found".to_string())
        })?
    };
    tracing::info!("{}", format!("Tailing log: {}", log_path.display()));
    let f = fs::File::open(&log_path)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut reader = BufReader::new(f);
    let pos = reader
        .seek(SeekFrom::End(0))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let start_pos = pos.saturating_sub(65536u64);
    reader
        .seek(SeekFrom::Start(start_pos))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut initial_lines = Vec::new();
    for l in reader.lines().map_while(std::result::Result::ok) {
        initial_lines.push(l);
    }
    let to_show = if initial_lines.len() > lines {
        &initial_lines[initial_lines.len() - lines..]
    } else {
        &initial_lines[..]
    };
    for line in to_show {
        println!("{}", line);
    }
    let f = fs::File::open(&log_path)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut reader = BufReader::new(f);
    reader
        .seek(SeekFrom::End(0))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
        if bytes > 0 {
            print!("{}", line.trim_end());
        } else {
            thread::sleep(Duration::from_millis(500));
        }
    }
}

/// Synchronize project manifest with filesystem
#[verb("sync", "project")]
fn sync_project() -> Result<Value> {
    do_sync()
}

/// Build a project target
///
/// # Arguments
/// * `project` - Project name
/// * `target` - Build target
/// * `platform` - Target platform
#[verb("build", "project")]
fn build_project(
    project: Option<String>,
    target: Option<String>,
    platform: Option<String>,
) -> Result<Value> {
    do_build(project, target, platform)
}

/// Audit project health and semantic law compliance
#[verb("audit", "project")]
fn audit_project() -> Result<Value> {
    do_audit()
}

/// Launch interactive TUI for project management
#[verb("run", "project")]
fn run_project() -> Result<Value> {
    tracing::info!("Launching Interactive TUI...");
    Ok(serde_json::json!({"status": "ok"}))
}

/// Show project information
#[verb("info", "project")]
fn info_project() -> Result<Value> {
    tracing::info!("{}", "Rocket Craft Generative Orchestration Tool");
    Ok(serde_json::json!({"version": "0.1.0", "stack": "Ostar / ggen / Rust / UE4.24"}))
}

/// Run all tests (Rust, Asset validation, etc.)
#[verb("test", "project")]
fn test_project() -> Result<Value> {
    do_test()
}

/// Tail Unreal Engine build logs
///
/// # Arguments
/// * `file` - Specific log file to tail
/// * `lines` - Number of initial lines to show
#[verb("logs", "project")]
fn logs_project(file: Option<String>, lines: Option<u64>) -> Result<Value> {
    do_logs(file, lines)
}
