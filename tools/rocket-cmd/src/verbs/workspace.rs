//! Workspace management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use colored::Colorize;
use serde_json::Value;

fn do_lock() -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    crate::lock::run_lock(&root)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_clean() -> Result<Value> {
    use std::fs;
    use walkdir::WalkDir;
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    tracing::info!("{}", "=== Cleaning Workspace ===");
    let targets = ["Binaries", "Intermediate", "Saved"];
    for entry in WalkDir::new(root.join("versions"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_dir() && targets.contains(&e.file_name().to_string_lossy().as_ref())
        })
    {
        fs::remove_dir_all(entry.path())
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    }
    tracing::info!("{}", "Cleanup complete.");
    Ok(serde_json::json!({"status": "ok"}))
}

/// Recursively map dependencies of all local workspaces and enforce deterministic lock
#[verb("lock", "workspace")]
fn lock_workspace() -> Result<Value> {
    do_lock()
}

/// Clean build artifacts (Binaries, Intermediate, Saved)
#[verb("clean", "workspace")]
fn clean_workspace() -> Result<Value> {
    do_clean()
}

/// List all Rust workspaces in this monorepo with their crate counts and paths.
///
/// # Arguments
/// * `json` - Output machine-readable JSON
#[verb("list", "workspace")]
fn list_workspace(json: bool) -> Result<Value> {
    do_list(json)
}

/// Run cargo check across all workspaces in parallel and report pass/fail.
///
/// # Arguments
/// * `json` - Output machine-readable JSON
#[verb("check", "workspace")]
fn check_workspace(json: bool) -> Result<Value> {
    do_check(json)
}

// ─── workspace list ──────────────────────────────────────────────────────────

struct WorkspaceInfo {
    name: &'static str,
    path: &'static str,
    crate_count: usize,
    exists: bool,
}

/// Count `[workspace]` members in a `Cargo.toml` by scanning for quoted entries
/// inside the `members = [...]` block.  Not a full TOML parser but reliable for
/// the repo's formatting conventions.
fn count_workspace_members(cargo_toml_path: &std::path::Path) -> usize {
    let content = match std::fs::read_to_string(cargo_toml_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    // Locate the `members` array inside a `[workspace]` section.
    let in_workspace = content.contains("[workspace]");
    if !in_workspace {
        return 0;
    }

    let mut count = 0usize;
    let mut in_members = false;
    let mut depth = 0i32;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
        }
        if in_members {
            for ch in trimmed.chars() {
                match ch {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth <= 0 {
                            in_members = false;
                            break;
                        }
                    }
                    '"' => {
                        // Each opening quote that is not a closing quote starts a path string.
                        // We count pairs: every two '"' characters = one member.
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    // Each member path is delimited by a pair of quotes → divide by 2.
    count / 2
}

fn do_list(json: bool) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    // Known workspaces from CLAUDE.md
    let definitions: &[(&str, &str)] = &[
        ("tools", "tools"),
        ("nexus-engine", "nexus-engine"),
        ("blueprint-rs", "blueprint-rs"),
        ("unify-rs", "unify-rs"),
        ("infinity-blade-4/mud", "infinity-blade-4/mud"),
        ("chicago-tdd-tools", "chicago-tdd-tools"),
        ("asset-pipeline", "asset-pipeline"),
    ];

    let workspaces: Vec<WorkspaceInfo> = definitions
        .iter()
        .map(|(name, rel_path)| {
            let ws_path = root.join(rel_path);
            let exists = ws_path.exists();
            let crate_count = if exists {
                count_workspace_members(&ws_path.join("Cargo.toml"))
            } else {
                0
            };
            WorkspaceInfo {
                name,
                path: rel_path,
                crate_count,
                exists,
            }
        })
        .collect();

    if json {
        let json_list: Vec<Value> = workspaces
            .iter()
            .map(|ws| {
                serde_json::json!({
                    "name": ws.name,
                    "path": ws.path,
                    "crate_count": ws.crate_count,
                    "exists": ws.exists,
                })
            })
            .collect();
        let out = serde_json::json!({ "workspaces": json_list });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return Ok(out);
    }

    println!("\n{}", "Rust Workspaces".bold());
    println!("{}", "─".repeat(60));
    for ws in &workspaces {
        let name_col = format!("{}/", ws.name);
        let status = if ws.exists {
            "[✓ exists]".green().to_string()
        } else {
            "[✗ missing]".red().to_string()
        };
        let crates = if ws.crate_count > 0 {
            format!("{:>3} crates", ws.crate_count)
        } else {
            "  ? crates".to_string()
        };
        println!(
            "  {:<40} {}   {}",
            name_col.bold(),
            crates,
            status
        );
    }
    println!();

    let total: usize = workspaces.iter().map(|w| w.crate_count).sum();
    let present = workspaces.iter().filter(|w| w.exists).count();
    println!(
        "  {} of {} workspaces present, {} total crates",
        present.to_string().cyan(),
        workspaces.len(),
        total.to_string().cyan()
    );

    Ok(serde_json::json!({
        "count": workspaces.len(),
        "present": present,
        "total_crates": total,
    }))
}

// ─── workspace check ─────────────────────────────────────────────────────────

struct CheckOutcome {
    name: String,
    path: String,
    passed: bool,
    elapsed_ms: u64,
    stderr_snippet: String,
}

fn do_check(json: bool) -> Result<Value> {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Instant;

    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let definitions: &[(&str, &str)] = &[
        ("tools", "tools"),
        ("nexus-engine", "nexus-engine"),
        ("blueprint-rs", "blueprint-rs"),
        ("unify-rs", "unify-rs"),
        ("infinity-blade-4/mud", "infinity-blade-4/mud"),
        ("chicago-tdd-tools", "chicago-tdd-tools"),
        ("asset-pipeline", "asset-pipeline"),
    ];

    // Filter to only workspaces that exist on disk.
    let existing: Vec<(String, std::path::PathBuf)> = definitions
        .iter()
        .filter_map(|(name, rel)| {
            let p = root.join(rel);
            if p.exists() { Some((name.to_string(), p)) } else { None }
        })
        .collect();

    if !json {
        println!(
            "\n{}",
            format!("Running `cargo check --workspace` on {} workspaces…", existing.len()).bold()
        );
    }

    let results: Arc<Mutex<Vec<CheckOutcome>>> = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = existing
        .into_iter()
        .map(|(name, ws_path)| {
            let results = Arc::clone(&results);
            thread::spawn(move || {
                let start = Instant::now();
                let output = std::process::Command::new("cargo")
                    .args(["check", "--workspace", "--quiet"])
                    .current_dir(&ws_path)
                    .output();
                let elapsed_ms = start.elapsed().as_millis() as u64;

                let (passed, stderr_snippet) = match output {
                    Ok(out) => {
                        let passed = out.status.success();
                        let snippet = if !passed {
                            String::from_utf8_lossy(&out.stderr)
                                .chars()
                                .take(400)
                                .collect()
                        } else {
                            String::new()
                        };
                        (passed, snippet)
                    }
                    Err(e) => (false, format!("could not run cargo: {e}")),
                };

                results.lock().unwrap().push(CheckOutcome {
                    name,
                    path: ws_path.display().to_string(),
                    passed,
                    elapsed_ms,
                    stderr_snippet,
                });
            })
        })
        .collect();

    for h in handles {
        h.join().map_err(|_| {
            clap_noun_verb::NounVerbError::execution_error("worker thread panicked".to_string())
        })?;
    }

    let mut outcomes = results.lock().unwrap();
    // Sort deterministically by name.
    outcomes.sort_by(|a, b| a.name.cmp(&b.name));

    if json {
        let json_list: Vec<Value> = outcomes
            .iter()
            .map(|o| {
                serde_json::json!({
                    "name": o.name,
                    "path": o.path,
                    "passed": o.passed,
                    "elapsed_ms": o.elapsed_ms,
                    "error": if o.stderr_snippet.is_empty() { None } else { Some(&o.stderr_snippet) },
                })
            })
            .collect();
        let all_pass = outcomes.iter().all(|o| o.passed);
        let out = serde_json::json!({ "workspaces": json_list, "all_pass": all_pass });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return Ok(out);
    }

    println!("{}", "─".repeat(60));
    for o in outcomes.iter() {
        let status = if o.passed {
            "PASS".green().bold().to_string()
        } else {
            "FAIL".red().bold().to_string()
        };
        println!(
            "  {}  {:<40}  {:>5} ms",
            status,
            o.name.bold(),
            o.elapsed_ms
        );
        if !o.stderr_snippet.is_empty() {
            for line in o.stderr_snippet.lines().take(6) {
                println!("       {}", line.dimmed());
            }
        }
    }
    println!("{}", "─".repeat(60));

    let pass_count = outcomes.iter().filter(|o| o.passed).count();
    let total = outcomes.len();
    let summary = format!("{pass_count}/{total} workspaces pass `cargo check`");
    if pass_count == total {
        println!("  {}", summary.green().bold());
    } else {
        println!("  {}", summary.red().bold());
    }

    Ok(serde_json::json!({
        "pass": pass_count,
        "total": total,
        "all_pass": pass_count == total,
    }))
}
