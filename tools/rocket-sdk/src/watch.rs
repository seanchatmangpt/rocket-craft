//! Polling file watcher + task graph for the `rocket` CLI.
//!
//! This module powers `rocket watch`: it scans a set of roots, builds a snapshot
//! of [`FileFingerprint`]s, and on each poll computes a [`Changes`] diff against
//! the previous snapshot. Changed paths are then resolved through a [`TaskGraph`]
//! into a set of [`Task`]s to run (e.g. `cargo test -p <crate>` for Rust files,
//! `rocket pwa lint` for `pwa-staff` TypeScript).
//!
//! ### Why polling (and not `notify`)
//! The mission requires std-only operation with no external fs-events crate.
//! Polling with content-hash fingerprints is robust across network filesystems,
//! containers and editors that do atomic-rename saves, at the cost of latency
//! bounded by the poll interval. All non-IO logic lives in **pure functions**
//! (`snapshot`, `diff`, `TaskGraph::resolve`, debounce) so it is fully testable.
//!
//! Honors the repo's binary-asset rule: `.uasset`/`.umap` and other binaries are
//! fingerprinted by metadata only (see [`crate::cache::is_binary_asset`]) and the
//! task graph emits a *skip/warn* action for them rather than acting on them.

use crate::cache::{is_binary_asset, FileFingerprint, Fingerprinter};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
