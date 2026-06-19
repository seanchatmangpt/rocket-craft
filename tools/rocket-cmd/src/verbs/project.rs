//! Project management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_sync() -> Result<Value> {
    use rocket_sdk::manifest;
    use indicatif::{ProgressBar, ProgressStyle};
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    tracing::info!("{}", "=== Syncing Project Manifest ===");
    let versions_dir = Path::new("versions");
    if !versions_dir.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("versions directory not found: {}", versions_dir.display())
        ));
    }
    let entries: Vec<_> = WalkDir::new(versions_dir).max_depth(5)
        .into_iter().filter_map(|e| e.ok())
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
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
        pb.set_message(format!("Syncing {}", name));
        let mut targets = Vec::new();
        let source_dir = path.parent()
            .ok_or_else(|| clap_noun_verb::NounVerbError::execution_error(format!("no parent dir: {}", path.display())))?
            .join("Source");
        if source_dir.exists() {
            for t_entry in fs::read_dir(source_dir).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))? {
                let t_entry = t_entry.map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
                let t_name = t_entry.file_name().to_string_lossy().to_string();
                if t_name.ends_with(".Target.cs") {
                    targets.push(t_name.replace(".Target.cs", ""));
                }
            }
        }
        projects.push(manifest::Project { name, uproject_path: path.to_path_buf(), targets });
        pb.inc(1);
    }
    pb.finish_with_message("Sync complete");
    let manifest = manifest::Manifest::new("project-manifest.json", projects);
    manifest.save().map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    tracing::info!("{}", "Manifest updated successfully.");
    Ok(serde_json::json!({"status": "ok", "projects_found": entries.len()}))
}

fn do_build(project: Option<String>, target: Option<String>, platform: Option<String>) -> Result<Value> {
    use rocket_sdk::manifest;
    use std::path::Path;
    use std::process::Command;

    let manifest = manifest::Manifest::load("project-manifest.json")
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let name = project.ok_or_else(|| clap_noun_verb::NounVerbError::execution_error("No project specified. Use --project".to_string()))?;
    let proj = manifest.projects().iter().find(|p| p.name == name)
        .ok_or_else(|| clap_noun_verb::NounVerbError::execution_error(format!("project not found: {}", name)))?;
    let target_val = target.or_else(|| proj.targets.first().cloned())
        .ok_or_else(|| clap_noun_verb::NounVerbError::execution_error(format!("no target found for project: {}", proj.name)))?;
    let platform_val = platform.unwrap_or_else(|| "Win64".to_string());
    let config = rocket_sdk::config::RocketConfig::load()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let ue4_root = config.ue4_root.ok_or_else(|| clap_noun_verb::NounVerbError::execution_error("UE4_ROOT not set. Run 'rocket env setup' first.".to_string()))?;
    let uat_name = if cfg!(windows) { "RunUAT.bat" } else { "RunUAT.sh" };
    let uat_path = ue4_root.join("Engine").join("Build").join("BatchFiles").join(uat_name);
    tracing::info!("Building {} [{}] for {}...", proj.name, target_val, platform_val);
    let status = Command::new(&uat_path)
        .arg("BuildCookRun")
        .arg(format!("-project={}", proj.uproject_path.display()))
        .arg(format!("-target={}", target_val))
        .arg(format!("-platform={}", platform_val))
        .arg("-cook").arg("-build").arg("-stage").arg("-archive")
        .arg(format!("-archivedirectory={}", Path::new("Builds").display()))
        .status().map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error("Build failed".to_string()));
    }
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_audit() -> Result<Value> {
    use rocket_sdk::{audit_affidavit::{AuditEvent, persist_receipt, record_audit}, manifest};
    use crate::compliance::ComplianceEngine;
    use knhk::AndroidKeystoreLaw;
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
                let map_count = WalkDir::new(maps_dir).into_iter().filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("umap")).count();
                tracing::info!("  [OK] Maps found: {}", map_count);
            }
            let result = engine.check_project(proj);
            if result.passed {
                tracing::info!("    [OK] All laws satisfied.");
            } else {
                for err in &result.errors {
                    tracing::info!("    [FAIL] Law '{}' violated: {}", err.law_name, err.message);
                }
            }
            audit_events.push(AuditEvent {
                project_name: result.project_name,
                passed: result.passed,
                violations: result.errors.into_iter().map(|e| (e.law_name, e.message)).collect(),
                visual_delta: None,
                combinatorial_matrix: None,
            });
        }
    }
    match record_audit(&audit_events) {
        Ok(receipt) => {
            tracing::info!("  Chain sealed — {} event(s), hash: {}", receipt.events.len(), &receipt.chain_hash.as_hex()[..16]);
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
    let status = Command::new("cargo").arg("test").arg("--workspace").current_dir("tools")
        .status().map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error("Rust workspace tests failed".to_string()));
    }
    let status = Command::new("cargo").arg("test").current_dir("chicago-tdd-tools")
        .status().map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error("Chicago TDD Tools tests failed".to_string()));
    }
    do_asset_validation()?;
    tracing::info!("\n{}", "All tests passed!");
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_asset_validation() -> Result<Value> {
    use std::fs;
    use walkdir::WalkDir;
    tracing::info!("\n--- Asset Validation (Rust Native) ---");
    let forbidden_patterns = ["Highrise", "Brm-HTML5-Shipping"];
    let ignore_dirs = [".git", ".agents", "non-project-files", "pwa-staff", "versions", "docs", "tools", "chicago-tdd-tools", "unify-rs", "target", "scratch"];
    let ignore_files = ["validate-assets.py", "ROCKET_CRAFT_AUDIT.md", "README.md", "HELP.md", "CLAUDE.md", "DFLSS.md"];
    let ignore_extensions = ["log", "lock", "json", "md", "txt", "js", "applescript", "sh", "bat", "yml", "yaml", "toml"];
    let mut issues_found = false;
    let project_root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    for entry in WalkDir::new(&project_root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        let rel_path = path.strip_prefix(&project_root).unwrap_or(path);
        if ignore_dirs.iter().any(|d| rel_path.components().any(|c| c.as_os_str() == *d)) { continue; }
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ignore_extensions.contains(&ext) { continue; }
        }
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if ignore_files.contains(&filename) { continue; }
        }
        if let Ok(content) = fs::read_to_string(path) {
            for pattern in &forbidden_patterns {
                if content.contains(pattern) {
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            tracing::warn!("  ALERT: Found reference to '{}' in {}:{}", pattern, rel_path.display(), line_idx + 1);
                            issues_found = true;
                        }
                    }
                }
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

fn do_logs(file: Option<String>, lines: Option<u64>) -> Result<Value> {
    use std::fs;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    let lines = lines.unwrap_or(50) as usize;
    let logs_dir = Path::new("non-project-files/logs");
    if !logs_dir.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!("Logs directory not found: {}", logs_dir.display())));
    }
    let log_path = if let Some(f) = file {
        let p = logs_dir.join(f);
        if !p.exists() {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!("Log file not found: {}", p.display())));
        }
        p
    } else {
        let mut entries: Vec<_> = fs::read_dir(logs_dir)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?
            .filter_map(|res| res.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        entries.last().map(|e| e.path())
            .ok_or_else(|| clap_noun_verb::NounVerbError::execution_error("No log files found".to_string()))?
    };
    tracing::info!("{}", format!("Tailing log: {}", log_path.display()));
    let f = fs::File::open(&log_path).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut reader = BufReader::new(f);
    let pos = reader.seek(SeekFrom::End(0)).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let start_pos = pos.saturating_sub(65536u64);
    reader.seek(SeekFrom::Start(start_pos)).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut initial_lines = Vec::new();
    for l in reader.lines().map_while(std::result::Result::ok) { initial_lines.push(l); }
    let to_show = if initial_lines.len() > lines { &initial_lines[initial_lines.len() - lines..] } else { &initial_lines[..] };
    for line in to_show { println!("{}", line); }
    let f = fs::File::open(&log_path).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    let mut reader = BufReader::new(f);
    reader.seek(SeekFrom::End(0)).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
        if bytes > 0 { print!("{}", line.trim_end()); } else { thread::sleep(Duration::from_millis(500)); }
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
fn build_project(project: Option<String>, target: Option<String>, platform: Option<String>) -> Result<Value> {
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
