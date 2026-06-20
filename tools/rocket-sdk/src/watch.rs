//! Live-rebuild watcher for all Rust workspaces and the PWA.
//!
//! Watches `.rs` and `.toml` files across all known workspace roots, plus
//! `pwa-staff/src/**/*.ts`.  After a 500 ms debounce, it identifies the
//! affected workspace by path prefix and runs `cargo check --workspace`
//! (or `cargo test --workspace` when a test file changed, or
//! `npm run build:ts` for the PWA).

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

// ─── Workspace registry ───────────────────────────────────────────────────────

/// A watched workspace entry.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Human-readable label shown in log output.
    pub name: &'static str,
    /// Relative (or absolute) root directory of the workspace.
    pub root: &'static str,
    /// `true` for Rust workspaces (uses `cargo check`), `false` for PWA (uses npm).
    pub is_rust: bool,
}

/// All workspaces that `watch` monitors by default.
pub const ALL_WORKSPACES: &[Workspace] = &[
    Workspace { name: "tools",              root: "tools",                  is_rust: true  },
    Workspace { name: "nexus-engine",       root: "nexus-engine",           is_rust: true  },
    Workspace { name: "blueprint-rs",       root: "blueprint-rs",           is_rust: true  },
    Workspace { name: "unify-rs",           root: "unify-rs",               is_rust: true  },
    Workspace { name: "infinity-blade-4/mud", root: "infinity-blade-4/mud", is_rust: true  },
    Workspace { name: "chicago-tdd-tools",  root: "chicago-tdd-tools",      is_rust: true  },
    Workspace { name: "pwa-staff",          root: "pwa-staff",              is_rust: false },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

/// Returns `true` when the file extension is one we care about for a given workspace.
fn extension_matches(path: &Path, is_rust: bool) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if is_rust {
        matches!(ext, "rs" | "toml")
    } else {
        ext == "ts"
    }
}

/// Returns `true` when the changed path looks like a test file.
fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.ends_with("_test.rs") {
        return true;
    }
    // Any file inside a `tests/` directory
    path.components().any(|c| c.as_os_str() == "tests")
}

/// Run a command, stream its stdout/stderr to the terminal, and return whether
/// it succeeded plus the elapsed wall time.
fn run_and_report(label: &str, args: &[&str], cwd: &Path) -> (bool, Duration) {
    let start = Instant::now();
    let status = Command::new(args[0])
        .args(&args[1..])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .status();
    let elapsed = start.elapsed();

    match status {
        Ok(s) if s.success() => (true, elapsed),
        Ok(_) => {
            eprintln!(
                "[{}] {} failed ({:.1}s)",
                timestamp().yellow(),
                label,
                elapsed.as_secs_f64()
            );
            (false, elapsed)
        }
        Err(e) => {
            eprintln!(
                "[{}] could not spawn `{}`: {}",
                timestamp().red(),
                args[0],
                e
            );
            (false, elapsed)
        }
    }
}

// ─── Core watch logic ─────────────────────────────────────────────────────────

/// Configuration for the watcher.
pub struct WatchConfig {
    /// Workspace root of the entire monorepo.
    pub repo_root: PathBuf,
    /// If `Some`, restrict watching to a single workspace by name.
    pub only_workspace: Option<String>,
    /// Debounce window.  Defaults to 500 ms.
    pub debounce: Duration,
}

impl WatchConfig {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
            only_workspace: None,
            debounce: Duration::from_millis(500),
        }
    }

    pub fn only(mut self, name: impl Into<String>) -> Self {
        self.only_workspace = Some(name.into());
        self
    }
}

/// Entry point: run the watcher loop until Ctrl-C.
pub fn run(cfg: WatchConfig) -> Result<()> {
    let repo_root = cfg.repo_root.canonicalize().context("resolving repo root")?;

    // ── Select workspaces ──────────────────────────────────────────────────
    let workspaces: Vec<&Workspace> = if let Some(ref name) = cfg.only_workspace {
        let ws = ALL_WORKSPACES
            .iter()
            .find(|w| w.name == name.as_str())
            .with_context(|| format!("unknown workspace '{}'. valid: {}", name,
                ALL_WORKSPACES.iter().map(|w| w.name).collect::<Vec<_>>().join(", ")))?;
        vec![ws]
    } else {
        ALL_WORKSPACES.iter().collect()
    };

    println!(
        "[{}] {} Watching {} workspace(s) — press Ctrl-C to stop.",
        timestamp().cyan(),
        "rocket watch".bold(),
        workspaces.len(),
    );
    for ws in &workspaces {
        let abs = repo_root.join(ws.root);
        if abs.exists() {
            println!("  {} {}", "»".dimmed(), ws.name.bold());
        } else {
            println!("  {} {} (directory not found — skipping)", "!".yellow(), ws.name);
        }
    }

    // ── Build prefix → workspace map ──────────────────────────────────────
    // Sorted longest-first so most-specific prefix wins.
    let mut prefix_map: Vec<(PathBuf, &Workspace)> = workspaces
        .iter()
        .filter_map(|ws| {
            let abs = repo_root.join(ws.root);
            abs.exists().then(|| {
                let canon = abs.canonicalize().unwrap_or(abs);
                (canon, *ws)
            })
        })
        .collect();
    prefix_map.sort_by(|(a, _), (b, _)| b.as_os_str().len().cmp(&a.as_os_str().len()));

    // ── Set up notify watcher ─────────────────────────────────────────────
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    for (abs, _) in &prefix_map {
        watcher.watch(abs, RecursiveMode::Recursive)?;
    }

    // ── Debounce state ────────────────────────────────────────────────────
    // workspace_name → (last_event_time, set of changed paths, any_test_file)
    let mut pending: HashMap<String, (Instant, Vec<PathBuf>, bool)> = HashMap::new();

    loop {
        // Drain all immediately available events into `pending`.
        loop {
            match rx.try_recv() {
                Ok(Ok(event)) => {
                    handle_event(&event, &prefix_map, &mut pending);
                }
                Ok(Err(e)) => eprintln!("[{}] watch error: {}", timestamp().red(), e),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(anyhow::anyhow!("watcher channel disconnected"));
                }
            }
        }

        // Check for workspaces whose debounce window has expired.
        let now = Instant::now();
        let ready: Vec<String> = pending
            .iter()
            .filter(|(_, (t, _, _))| now.duration_since(*t) >= cfg.debounce)
            .map(|(k, _)| k.clone())
            .collect();

        for key in ready {
            if let Some((_, paths, has_tests)) = pending.remove(&key) {
                // Find the workspace entry.
                let Some((_, ws)) = prefix_map.iter().find(|(_, w)| w.name == key) else {
                    continue;
                };
                let ws_abs = repo_root.join(ws.root);

                // Report the triggering file (show just the first one for brevity).
                let trigger = paths
                    .first()
                    .and_then(|p| p.strip_prefix(&repo_root).ok())
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "<unknown>".into());

                println!(
                    "[{}] {} changed ({})",
                    timestamp().cyan(),
                    ws.name.bold(),
                    trigger
                );

                if ws.is_rust {
                    // Always run `cargo check`.
                    println!(
                        "[{}] Running: {}",
                        timestamp().cyan(),
                        "cargo check --workspace".dimmed()
                    );
                    let (ok, elapsed) =
                        run_and_report(ws.name, &["cargo", "check", "--workspace"], &ws_abs);
                    if ok {
                        println!(
                            "[{}] {} {} ({:.1}s)",
                            timestamp().cyan(),
                            "✓".green().bold(),
                            ws.name.green().bold(),
                            elapsed.as_secs_f64()
                        );
                    } else {
                        println!(
                            "[{}] {} {} ({:.1}s)",
                            timestamp().cyan(),
                            "✗".red().bold(),
                            ws.name.red().bold(),
                            elapsed.as_secs_f64()
                        );
                    }

                    // Optionally run tests for test-file changes.
                    if has_tests && ok {
                        println!(
                            "[{}] Running: {} (test files changed)",
                            timestamp().cyan(),
                            "cargo test --workspace".dimmed()
                        );
                        let (tok, telapsed) = run_and_report(
                            ws.name,
                            &["cargo", "test", "--workspace"],
                            &ws_abs,
                        );
                        if tok {
                            println!(
                                "[{}] {} tests {} ({:.1}s)",
                                timestamp().cyan(),
                                "✓".green().bold(),
                                ws.name.green().bold(),
                                telapsed.as_secs_f64()
                            );
                        } else {
                            println!(
                                "[{}] {} tests {} ({:.1}s)",
                                timestamp().cyan(),
                                "✗".red().bold(),
                                ws.name.red().bold(),
                                telapsed.as_secs_f64()
                            );
                        }
                    }
                } else {
                    // PWA: run npm build:ts
                    println!(
                        "[{}] Running: {}",
                        timestamp().cyan(),
                        "npm run build:ts".dimmed()
                    );
                    let (ok, elapsed) = run_and_report(
                        ws.name,
                        &["npm", "run", "build:ts"],
                        &ws_abs,
                    );
                    if ok {
                        println!(
                            "[{}] {} {} ({:.1}s)",
                            timestamp().cyan(),
                            "✓".green().bold(),
                            ws.name.green().bold(),
                            elapsed.as_secs_f64()
                        );
                    } else {
                        println!(
                            "[{}] {} {} ({:.1}s)",
                            timestamp().cyan(),
                            "✗".red().bold(),
                            ws.name.red().bold(),
                            elapsed.as_secs_f64()
                        );
                    }
                }
            }
        }

        // Brief sleep so we don't spin the CPU.
        std::thread::sleep(Duration::from_millis(50));
    }
}

// ─── Event handler ────────────────────────────────────────────────────────────

fn handle_event(
    event: &Event,
    prefix_map: &[(PathBuf, &Workspace)],
    pending: &mut HashMap<String, (Instant, Vec<PathBuf>, bool)>,
) {
    // Only care about create/modify/remove events.
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
        _ => return,
    }

    for path in &event.paths {
        // Match path to a workspace.
        let Some((_, ws)) = prefix_map.iter().find(|(prefix, _)| path.starts_with(prefix)) else {
            continue;
        };

        // Filter by extension.
        if !extension_matches(path, ws.is_rust) {
            continue;
        }

        let test_hit = ws.is_rust && is_test_file(path);
        let entry = pending
            .entry(ws.name.to_string())
            .or_insert_with(|| (Instant::now(), Vec::new(), false));
        entry.0 = Instant::now(); // reset debounce timer
        entry.1.push(path.clone());
        if test_hit {
            entry.2 = true;
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn extension_matches_rust_files() {
        assert!(extension_matches(Path::new("src/lib.rs"), true));
        assert!(extension_matches(Path::new("Cargo.toml"), true));
        assert!(!extension_matches(Path::new("src/main.ts"), true));
        assert!(!extension_matches(Path::new("package.json"), true));
    }

    #[test]
    fn extension_matches_pwa_files() {
        assert!(extension_matches(Path::new("src/auth.ts"), false));
        assert!(!extension_matches(Path::new("src/lib.rs"), false));
        assert!(!extension_matches(Path::new("package.json"), false));
    }

    #[test]
    fn is_test_file_detects_test_suffix() {
        assert!(is_test_file(Path::new("src/combat_test.rs")));
        assert!(is_test_file(Path::new("some/deep/path/foo_test.rs")));
        assert!(!is_test_file(Path::new("src/lib.rs")));
    }

    #[test]
    fn is_test_file_detects_tests_directory() {
        assert!(is_test_file(Path::new("nexus-engine/tests/integration.rs")));
        assert!(is_test_file(Path::new("tests/foo.rs")));
        assert!(!is_test_file(Path::new("src/foo.rs")));
    }

    #[test]
    fn all_workspaces_have_unique_names() {
        let names: Vec<_> = ALL_WORKSPACES.iter().map(|w| w.name).collect();
        let mut dedup = names.clone();
        dedup.sort_unstable();
        dedup.dedup();
        assert_eq!(names.len(), dedup.len(), "workspace names must be unique");
    }

    #[test]
    fn all_rust_workspaces_are_marked_rust() {
        let rust_ws: Vec<_> = ALL_WORKSPACES
            .iter()
            .filter(|w| w.is_rust)
            .map(|w| w.name)
            .collect();
        assert!(rust_ws.contains(&"tools"));
        assert!(rust_ws.contains(&"nexus-engine"));
        assert!(rust_ws.contains(&"blueprint-rs"));
    }

    #[test]
    fn pwa_workspace_not_marked_rust() {
        let pwa = ALL_WORKSPACES.iter().find(|w| w.name == "pwa-staff").unwrap();
        assert!(!pwa.is_rust);
    }
}
