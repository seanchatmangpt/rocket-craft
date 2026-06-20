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
#[allow(unused_imports)]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};
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

// ─── Polling watcher / TaskGraph (master additions) ──────────────────────────

use crate::cache::{is_binary_asset, FileFingerprint, Fingerprinter};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Snapshot + diff (pure)
// ---------------------------------------------------------------------------

/// A point-in-time map of path -> fingerprint. `BTreeMap` gives deterministic
/// iteration order, which keeps diffs and tests stable.
pub type Snapshot = BTreeMap<PathBuf, FileFingerprint>;

/// The result of diffing two snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    pub created: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

impl Changes {
    /// True when nothing changed.
    pub fn is_empty(&self) -> bool {
        self.created.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    /// Total number of changed paths.
    pub fn len(&self) -> usize {
        self.created.len() + self.modified.len() + self.deleted.len()
    }

    /// All changed paths (created + modified + deleted), useful for task resolution.
    pub fn all_paths(&self) -> Vec<PathBuf> {
        let mut v = Vec::with_capacity(self.len());
        v.extend(self.created.iter().cloned());
        v.extend(self.modified.iter().cloned());
        v.extend(self.deleted.iter().cloned());
        v
    }
}

/// Pure function: diff two snapshots into [`Changes`].
///
/// A file is *modified* if its fingerprint differs. For text/code files this
/// means a content-hash change; for binary/oversize files (where `hash` is
/// `None`) it falls back to `(mtime, size)`, which is exactly the
/// [`FileFingerprint`] equality.
pub fn diff(prev: &Snapshot, next: &Snapshot) -> Changes {
    let mut changes = Changes::default();

    for (path, next_fp) in next {
        match prev.get(path) {
            None => changes.created.push(path.clone()),
            Some(prev_fp) if prev_fp != next_fp => changes.modified.push(path.clone()),
            Some(_) => {}
        }
    }
    for path in prev.keys() {
        if !next.contains_key(path) {
            changes.deleted.push(path.clone());
        }
    }

    changes.created.sort();
    changes.modified.sort();
    changes.deleted.sort();
    changes
}

// ---------------------------------------------------------------------------
// Watcher (filesystem scanning)
// ---------------------------------------------------------------------------

/// Configuration for which files a [`Watcher`] considers.
///
/// Predicates are glob-ish substring/extension filters kept intentionally simple
/// (no external glob crate). `include_exts` of `None` means "any extension".
#[derive(Debug, Clone, Default)]
pub struct WatchFilter {
    /// If `Some`, only files whose (lowercased) extension is in this list are
    /// considered. `None` = all extensions.
    pub include_exts: Option<Vec<String>>,
    /// Any path containing one of these substrings is excluded. Matched against
    /// the full path string, so directory names like `target` work.
    pub exclude_substrings: Vec<String>,
}

impl WatchFilter {
    /// A sensible default for the rocket monorepo: watch source-ish files, skip
    /// build output, VCS metadata and dependency trees.
    pub fn rocket_defaults() -> Self {
        Self {
            include_exts: Some(
                ["rs", "toml", "ts", "tsx", "css", "json", "py", "uasset", "umap"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            exclude_substrings: [
                "/target/",
                "/node_modules/",
                "/.git/",
                "/dist/",
                "/Intermediate/",
                "/Binaries/",
                "/Saved/",
                ".rocket-cache.json",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }

    /// Pure predicate: should this path be watched?
    pub fn accepts(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        if self.exclude_substrings.iter().any(|s| path_str.contains(s.as_str())) {
            return false;
        }
        match &self.include_exts {
            None => true,
            Some(exts) => match path.extension().and_then(|e| e.to_str()) {
                Some(ext) => {
                    let ext = ext.to_ascii_lowercase();
                    exts.iter().any(|e| e == &ext)
                }
                None => false,
            },
        }
    }
}

/// Scans a set of root directories and produces snapshots.
#[derive(Debug, Clone)]
pub struct Watcher {
    pub roots: Vec<PathBuf>,
    pub filter: WatchFilter,
    pub fingerprinter: Fingerprinter,
}

impl Watcher {
    pub fn new(roots: Vec<PathBuf>, filter: WatchFilter) -> Self {
        Self { roots, filter, fingerprinter: Fingerprinter::default() }
    }

    /// Walk all roots and fingerprint every accepted file into a [`Snapshot`].
    ///
    /// Uses `walkdir` (already a workspace dependency). Unreadable files are
    /// skipped silently — the watcher is advisory, never fatal. The walk does
    /// not descend into excluded directories where possible (pruned by checking
    /// the filter on directories too).
    pub fn snapshot(&self) -> Snapshot {
        let mut snap = Snapshot::new();
        for root in &self.roots {
            for entry in walkdir::WalkDir::new(root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !self.is_pruned_dir(e.path(), e.file_type().is_dir()))
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if !self.filter.accepts(path) {
                    continue;
                }
                if let Ok(fp) = self.fingerprinter.fingerprint(path) {
                    snap.insert(path.to_path_buf(), fp);
                }
            }
        }
        snap
    }

    /// Whether a *directory* should be pruned from the walk. We reuse the
    /// exclude-substring rules so we never descend into `target/`, `.git/`, etc.
    fn is_pruned_dir(&self, path: &Path, is_dir: bool) -> bool {
        if !is_dir {
            return false;
        }
        let path_str = path.to_string_lossy();
        self.filter.exclude_substrings.iter().any(|s| {
            // Match a directory excluder like "/target/" against "/target".
            let trimmed = s.trim_matches('/');
            !trimmed.is_empty() && path_str.ends_with(trimmed)
        })
    }
}

// ---------------------------------------------------------------------------
// Task graph (pure resolution)
// ---------------------------------------------------------------------------

/// An action the watcher decided to run in response to changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Task {
    /// Run a shell-ish command (already split into program + args by caller).
    Command { description: String, program: String, args: Vec<String> },
    /// A no-op with an explanation, e.g. a binary asset that must not be acted on.
    Skip { reason: String, path: PathBuf },
}

impl Task {
    pub fn command(description: impl Into<String>, program: impl Into<String>, args: &[&str]) -> Self {
        Task::Command {
            description: description.into(),
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Resolves changed paths into the set of [`Task`]s to run.
///
/// The resolution is **pure** and deterministic: same paths in, same tasks out
/// (deduplicated, in stable order). Rules, in priority order:
///   1. Binary assets (`.uasset`/`.umap`/…)  -> `Skip` (repo binary rule).
///   2. `*.rs` under a Rust crate            -> `cargo test -p <crate>`.
///   3. `pwa-staff/**/*.{ts,tsx,css}`        -> `rocket pwa lint`.
///   4. `*.py` validators                    -> `python3 validate-assets.py`.
///   5. `project-manifest.json`              -> `rocket sync`.
#[derive(Debug, Clone)]
pub struct TaskGraph {
    /// Absolute path to the monorepo root, used to detect crate membership and
    /// relativize paths.
    pub repo_root: PathBuf,
}

impl TaskGraph {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self { repo_root: repo_root.into() }
    }

    /// Pure resolution of a batch of changed paths into deduplicated tasks.
    pub fn resolve(&self, paths: &[PathBuf]) -> Vec<Task> {
        let mut tasks: Vec<Task> = Vec::new();

        for path in paths {
            if let Some(task) = self.resolve_one(path) {
                if !tasks.contains(&task) {
                    tasks.push(task);
                }
            }
        }
        tasks
    }

    /// Resolve a single path to a task (or `None` if nothing applies).
    pub fn resolve_one(&self, path: &Path) -> Option<Task> {
        let path_str = path.to_string_lossy().replace('\\', "/");

        // Rule 1: binary assets are never acted on.
        if is_binary_asset(path) {
            return Some(Task::Skip {
                reason: "binary asset (versions/** rule): not rebuilt automatically".to_string(),
                path: path.to_path_buf(),
            });
        }

        // Rule 5: manifest change -> resync.
        if path_str.ends_with("project-manifest.json") {
            return Some(Task::command("resync project manifest", "rocket", &["sync"]));
        }

        let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());

        match ext.as_deref() {
            // Rule 2: Rust source -> test the owning crate.
            // A toml not in a known crate (e.g. pipeline.toml) maps to `None`.
            Some("rs") | Some("toml") => self.crate_name_for(path).map(|crate_name| {
                Task::command(
                    format!("cargo test for crate {crate_name}"),
                    "cargo",
                    &["test", "-p", &crate_name],
                )
            }),
            // Rule 3: PWA TypeScript/CSS -> lint the PWA.
            Some("ts") | Some("tsx") | Some("css") if path_str.contains("pwa-staff/") => {
                Some(Task::command("lint pwa-staff", "rocket", &["pwa", "lint"]))
            }
            // Rule 4: Python validators -> run the asset validator.
            Some("py") => {
                Some(Task::command("validate assets", "python3", &["validate-assets.py"]))
            }
            _ => None,
        }
    }

    /// Find the cargo crate that owns `path` by walking up to the nearest
    /// directory containing a `Cargo.toml`, returning its directory name. This
    /// is a heuristic (the package name often differs) but is good enough to
    /// scope `cargo test -p <name>`; callers can map dir->package if needed.
    ///
    /// Pure-ish: depends only on `path` text, not the filesystem, so it stays
    /// testable. It infers the crate from the path segment immediately under a
    /// known workspace root marker.
    pub fn crate_name_for(&self, path: &Path) -> Option<String> {
        let path_str = path.to_string_lossy().replace('\\', "/");

        // Known workspace -> the path segment after the workspace dir is the crate.
        // e.g. ".../tools/rocket-sdk/src/cache.rs" -> "rocket-sdk".
        const WORKSPACE_MARKERS: &[&str] = &[
            "/tools/",
            "/nexus-engine/crates/",
            "/nexus-engine/",
            "/blueprint-rs/",
            "/unify-rs/",
            "/infinity-blade-4/mud/",
            "/chicago-tdd-tools/",
            "/asset-pipeline/",
        ];

        for marker in WORKSPACE_MARKERS {
            if let Some(idx) = path_str.find(marker) {
                let after = &path_str[idx + marker.len()..];
                let seg = after.split('/').next().unwrap_or("");
                if !seg.is_empty() && seg != "src" && seg != "Cargo.toml" && !seg.ends_with(".rs") {
                    return Some(seg.to_string());
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Debounce (pure)
// ---------------------------------------------------------------------------

/// Pure debounce decision: given the time elapsed since the last *fired* event
/// and the debounce window, decide whether a new event should fire now.
///
/// Returns `true` if `elapsed >= window` (enough quiet time has passed).
pub fn should_fire(elapsed: Duration, window: Duration) -> bool {
    elapsed >= window
}

/// Stateful debouncer wrapping [`should_fire`] for the runtime loop.
#[derive(Debug, Clone)]
pub struct Debouncer {
    window: Duration,
    last_fire: Option<Instant>,
}

impl Debouncer {
    pub fn new(window: Duration) -> Self {
        Self { window, last_fire: None }
    }

    /// Returns true (and records the fire time) if the window has elapsed since
    /// the last fire. The very first call always fires.
    pub fn ready(&mut self, now: Instant) -> bool {
        let fire = match self.last_fire {
            None => true,
            Some(last) => should_fire(now.duration_since(last), self.window),
        };
        if fire {
            self.last_fire = Some(now);
        }
        fire
    }
}

// ---------------------------------------------------------------------------
// WatchLoop (thin runtime)
// ---------------------------------------------------------------------------

/// Outcome of a single poll, handed to the callback.
#[derive(Debug, Clone)]
pub struct PollResult {
    pub changes: Changes,
    pub tasks: Vec<Task>,
}

/// A thin polling runtime around [`Watcher`] + [`TaskGraph`] + [`Debouncer`].
///
/// The loop itself contains no business logic — it just sleeps, snapshots,
/// diffs, resolves and invokes the callback. All decisions live in the pure
/// functions above so they can be unit-tested without ever sleeping.
pub struct WatchLoop {
    pub watcher: Watcher,
    pub graph: TaskGraph,
    pub interval: Duration,
    pub debouncer: Debouncer,
}

impl WatchLoop {
    pub fn new(watcher: Watcher, graph: TaskGraph, interval: Duration, debounce: Duration) -> Self {
        Self { watcher, graph, interval, debouncer: Debouncer::new(debounce) }
    }

    /// Pure step: given the previous snapshot, compute the next snapshot, the
    /// diff and the resolved tasks. Separated from `run` so it is testable.
    pub fn step(&self, prev: &Snapshot) -> (Snapshot, PollResult) {
        let next = self.watcher.snapshot();
        let changes = diff(prev, &next);
        let tasks = if changes.is_empty() {
            Vec::new()
        } else {
            self.graph.resolve(&changes.all_paths())
        };
        (next, PollResult { changes, tasks })
    }

    /// Run the polling loop forever, invoking `on_change` whenever the debounced
    /// diff is non-empty. `on_change` returning `false` stops the loop.
    ///
    /// This is the only impure/blocking entry point; it is intentionally tiny.
    pub fn run<F>(&mut self, mut on_change: F)
    where
        F: FnMut(&PollResult) -> bool,
    {
        let mut prev = self.watcher.snapshot();
        loop {
            std::thread::sleep(self.interval);
            let (next, result) = self.step(&prev);
            prev = next;
            if result.changes.is_empty() {
                continue;
            }
            if !self.debouncer.ready(Instant::now()) {
                continue;
            }
            if !on_change(&result) {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fp(path: &str, mtime: u64, size: u64, hash: Option<&str>) -> FileFingerprint {
        FileFingerprint {
            path: PathBuf::from(path),
            mtime,
            size,
            hash: hash.map(|s| s.to_string()),
        }
    }

    fn snap(items: &[FileFingerprint]) -> Snapshot {
        items.iter().map(|f| (f.path.clone(), f.clone())).collect()
    }

    // --- diff -------------------------------------------------------------

    #[test]
    fn diff_detects_create() {
        let prev = snap(&[fp("a", 1, 1, Some("x"))]);
        let next = snap(&[fp("a", 1, 1, Some("x")), fp("b", 1, 1, Some("y"))]);
        let c = diff(&prev, &next);
        assert_eq!(c.created, vec![PathBuf::from("b")]);
        assert!(c.modified.is_empty() && c.deleted.is_empty());
    }

    #[test]
    fn diff_detects_modify_via_hash() {
        let prev = snap(&[fp("a", 1, 10, Some("x"))]);
        let next = snap(&[fp("a", 1, 10, Some("y"))]); // same mtime+size, new hash
        let c = diff(&prev, &next);
        assert_eq!(c.modified, vec![PathBuf::from("a")]);
    }

    #[test]
    fn diff_detects_modify_via_metadata_for_binary() {
        // binary asset has hash=None, change detected by size
        let prev = snap(&[fp("Level.uasset", 1, 100, None)]);
        let next = snap(&[fp("Level.uasset", 2, 200, None)]);
        let c = diff(&prev, &next);
        assert_eq!(c.modified, vec![PathBuf::from("Level.uasset")]);
    }

    #[test]
    fn diff_detects_delete() {
        let prev = snap(&[fp("a", 1, 1, Some("x")), fp("b", 1, 1, Some("y"))]);
        let next = snap(&[fp("a", 1, 1, Some("x"))]);
        let c = diff(&prev, &next);
        assert_eq!(c.deleted, vec![PathBuf::from("b")]);
    }

    #[test]
    fn diff_empty_when_identical() {
        let s = snap(&[fp("a", 1, 1, Some("x"))]);
        assert!(diff(&s, &s).is_empty());
    }

    // --- filter -----------------------------------------------------------

    #[test]
    fn filter_excludes_target_and_node_modules() {
        let f = WatchFilter::rocket_defaults();
        assert!(!f.accepts(Path::new("/repo/tools/target/debug/foo.rs")));
        assert!(!f.accepts(Path::new("/repo/pwa-staff/node_modules/x.ts")));
        assert!(f.accepts(Path::new("/repo/tools/rocket-sdk/src/cache.rs")));
    }

    #[test]
    fn filter_respects_include_exts() {
        let f = WatchFilter::rocket_defaults();
        assert!(f.accepts(Path::new("/repo/a.rs")));
        assert!(f.accepts(Path::new("/repo/a.ts")));
        assert!(!f.accepts(Path::new("/repo/a.lock")));
        assert!(!f.accepts(Path::new("/repo/README"))); // no extension
    }

    // --- watcher snapshot (real fs) --------------------------------------

    #[test]
    fn watcher_snapshots_and_diffs_real_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::write(root.join("a.rs"), b"fn a() {}").unwrap();

        let watcher = Watcher::new(vec![root.clone()], WatchFilter::rocket_defaults());
        let s1 = watcher.snapshot();
        assert_eq!(s1.len(), 1);

        // create + modify
        fs::write(root.join("b.rs"), b"fn b() {}").unwrap();
        fs::write(root.join("a.rs"), b"fn a() { 1; }").unwrap();
        let s2 = watcher.snapshot();
        let c = diff(&s1, &s2);
        assert!(c.created.iter().any(|p| p.ends_with("b.rs")));
        assert!(c.modified.iter().any(|p| p.ends_with("a.rs")));
    }

    #[test]
    fn watcher_prunes_target_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/junk.rs"), b"x").unwrap();
        fs::write(root.join("keep.rs"), b"y").unwrap();

        let watcher = Watcher::new(vec![root], WatchFilter::rocket_defaults());
        let s = watcher.snapshot();
        assert_eq!(s.len(), 1, "files under target/ must be pruned");
    }

    // --- task graph -------------------------------------------------------

    #[test]
    fn task_graph_resolves_rust_to_cargo_test() {
        let tg = TaskGraph::new("/repo");
        let task = tg.resolve_one(Path::new("/repo/tools/rocket-sdk/src/cache.rs")).unwrap();
        match task {
            Task::Command { program, args, .. } => {
                assert_eq!(program, "cargo");
                assert_eq!(args, vec!["test", "-p", "rocket-sdk"]);
            }
            _ => panic!("expected a cargo command"),
        }
    }

    #[test]
    fn task_graph_resolves_nexus_crate() {
        let tg = TaskGraph::new("/repo");
        let task = tg
            .resolve_one(Path::new("/repo/nexus-engine/crates/nexus-combat/src/machine.rs"))
            .unwrap();
        match task {
            Task::Command { args, .. } => assert_eq!(args, vec!["test", "-p", "nexus-combat"]),
            _ => panic!("expected cargo command"),
        }
    }

    #[test]
    fn task_graph_skips_binary_assets() {
        let tg = TaskGraph::new("/repo");
        let task = tg
            .resolve_one(Path::new("/repo/versions/4.24-Shooter/Content/Map.uasset"))
            .unwrap();
        assert!(matches!(task, Task::Skip { .. }), "uasset must be a Skip");
    }

    #[test]
    fn task_graph_resolves_pwa_lint() {
        let tg = TaskGraph::new("/repo");
        let task = tg.resolve_one(Path::new("/repo/pwa-staff/src/auth.ts")).unwrap();
        match task {
            Task::Command { program, args, .. } => {
                assert_eq!(program, "rocket");
                assert_eq!(args, vec!["pwa", "lint"]);
            }
            _ => panic!("expected pwa lint command"),
        }
    }

    #[test]
    fn task_graph_resolves_manifest_to_sync() {
        let tg = TaskGraph::new("/repo");
        let task = tg.resolve_one(Path::new("/repo/project-manifest.json")).unwrap();
        match task {
            Task::Command { args, .. } => assert_eq!(args, vec!["sync"]),
            _ => panic!("expected sync"),
        }
    }

    #[test]
    fn task_graph_resolves_python_validator() {
        let tg = TaskGraph::new("/repo");
        let task = tg.resolve_one(Path::new("/repo/validate-assets.py")).unwrap();
        match task {
            Task::Command { program, args, .. } => {
                assert_eq!(program, "python3");
                assert_eq!(args, vec!["validate-assets.py"]);
            }
            _ => panic!("expected python validator"),
        }
    }

    #[test]
    fn task_graph_dedups_same_crate() {
        let tg = TaskGraph::new("/repo");
        let tasks = tg.resolve(&[
            PathBuf::from("/repo/tools/rocket-sdk/src/cache.rs"),
            PathBuf::from("/repo/tools/rocket-sdk/src/watch.rs"),
        ]);
        assert_eq!(tasks.len(), 1, "two files in one crate -> one test task");
    }

    #[test]
    fn task_graph_unknown_path_yields_nothing() {
        let tg = TaskGraph::new("/repo");
        assert!(tg.resolve_one(Path::new("/repo/random/notes.md")).is_none());
    }

    #[test]
    fn crate_name_ignores_loose_toml() {
        let tg = TaskGraph::new("/repo");
        // a toml not under a known workspace -> no crate
        assert!(tg.crate_name_for(Path::new("/repo/pipeline.toml")).is_none());
    }

    // --- debounce ---------------------------------------------------------

    #[test]
    fn should_fire_respects_window() {
        let window = Duration::from_millis(100);
        assert!(!should_fire(Duration::from_millis(50), window));
        assert!(should_fire(Duration::from_millis(100), window));
        assert!(should_fire(Duration::from_millis(150), window));
    }

    #[test]
    fn debouncer_first_call_fires_then_gates() {
        let mut d = Debouncer::new(Duration::from_millis(100));
        let t0 = Instant::now();
        assert!(d.ready(t0), "first call always fires");
        assert!(!d.ready(t0 + Duration::from_millis(50)), "too soon");
        assert!(d.ready(t0 + Duration::from_millis(120)), "after window");
    }

    // --- step (pure pipeline over real fs) -------------------------------

    #[test]
    fn watchloop_step_produces_tasks_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("tools/rocket-sdk/src")).unwrap();
        let src = root.join("tools/rocket-sdk/src/lib.rs");
        fs::write(&src, b"// v1").unwrap();

        let watcher = Watcher::new(vec![root.clone()], WatchFilter::rocket_defaults());
        let graph = TaskGraph::new(root.clone());
        let wl = WatchLoop::new(watcher, graph, Duration::from_millis(1), Duration::from_millis(0));

        let prev = wl.watcher.snapshot();
        fs::write(&src, b"// v2 changed").unwrap();
        let (_next, result) = wl.step(&prev);

        assert!(!result.changes.is_empty());
        assert!(result.tasks.iter().any(|t| matches!(
            t,
            Task::Command { program, .. } if program == "cargo"
        )));
    }

    #[test]
    fn watchloop_step_no_tasks_when_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::write(root.join("a.rs"), b"x").unwrap();
        let watcher = Watcher::new(vec![root.clone()], WatchFilter::rocket_defaults());
        let graph = TaskGraph::new(root);
        let wl = WatchLoop::new(watcher, graph, Duration::from_millis(1), Duration::from_millis(0));
        let prev = wl.watcher.snapshot();
        let (_n, result) = wl.step(&prev);
        assert!(result.changes.is_empty());
        assert!(result.tasks.is_empty());
    }
}
