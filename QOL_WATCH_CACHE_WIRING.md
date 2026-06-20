# Rocket QoL: Watch Mode + Content-Hash Build Cache — Wiring Guide

New, self-contained modules added to `rocket-sdk` (no existing files touched):

- `tools/rocket-sdk/src/cache.rs` — content-addressed build cache.
- `tools/rocket-sdk/src/watch.rs` — polling watcher + diff + task graph.

Everything is **std-only** (FNV-1a content hashing, no `notify`/`sha2`/`blake3`)
plus `serde`/`serde_json` (already deps) for the cache file and `walkdir`
(already a dep) for scanning. **No new dependencies are required.**

All 37 unit tests pass (`cargo test -p rocket-sdk`) and both modules are
clippy-clean. The single flaky `doctor::tests::test_non_passing_checks_have_fix`
failure observed under full-suite parallelism is pre-existing and unrelated
(it depends on PWA `node_modules` being present); it passes in isolation.

---

## 1. `lib.rs` — register the modules

Add to `tools/rocket-sdk/src/lib.rs` (alongside the other `pub mod` lines):

```rust
pub mod watch;
pub mod cache;
```

## 2. `Cargo.toml` — no changes needed

`serde`, `serde_json`, `walkdir`, and `tempfile` (dev) are already declared.
Nothing to add.

## 3. `rocket-cmd/src/main.rs` — new `watch` subcommand

Add a clap variant:

```rust
/// Watch the repo and auto-run tests/lint on change (polling, std-only).
Watch {
    /// Poll interval in milliseconds.
    #[arg(long, default_value_t = 500)]
    interval_ms: u64,
    /// Debounce window in milliseconds (coalesces rapid saves).
    #[arg(long, default_value_t = 300)]
    debounce_ms: u64,
    /// Print resolved tasks but do not execute them.
    #[arg(long)]
    dry_run: bool,
}
```

Handler:

```rust
use rocket_sdk::watch::{TaskGraph, Task, WatchFilter, WatchLoop, Watcher};
use std::time::Duration;

Commands::Watch { interval_ms, debounce_ms, dry_run } => {
    let root = std::env::current_dir()?;
    let watcher = Watcher::new(vec![root.clone()], WatchFilter::rocket_defaults());
    let graph = TaskGraph::new(root);
    let mut wl = WatchLoop::new(
        watcher,
        graph,
        Duration::from_millis(interval_ms),
        Duration::from_millis(debounce_ms),
    );

    println!("rocket watch: polling every {interval_ms}ms (Ctrl-C to stop)");
    wl.run(|result| {
        println!(
            "changed: +{} ~{} -{}",
            result.changes.created.len(),
            result.changes.modified.len(),
            result.changes.deleted.len(),
        );
        for task in &result.tasks {
            match task {
                Task::Skip { reason, path } => {
                    println!("  skip {}: {reason}", path.display());
                }
                Task::Command { description, program, args } => {
                    println!("  run  {description}: {program} {}", args.join(" "));
                    if !dry_run {
                        let _ = std::process::Command::new(program).args(args).status();
                    }
                }
            }
        }
        true // return false to stop the loop
    });
    Ok(())
}
```

## 4. Cache-aware `rocket build` / `rocket test`

Wrap the existing build/test handlers so identical inputs+command are skipped.
`is_fresh` short-circuits work; `record` is written after a successful run.

```rust
use rocket_sdk::cache::{BuildCache, CacheKey, FileFingerprint, Fingerprinter};
use std::path::Path;

fn cached_run(
    root: &Path,
    command: &str,
    input_files: &[std::path::PathBuf],
    run: impl FnOnce() -> anyhow::Result<bool>, // returns success
) -> anyhow::Result<()> {
    let fpr = Fingerprinter::default();
    let inputs: Vec<FileFingerprint> = input_files
        .iter()
        .filter_map(|p| fpr.fingerprint(p).ok())
        .collect();
    let key = CacheKey::derive(command, &inputs);

    let mut cache = BuildCache::load_from_dir(root);
    if cache.is_fresh(&key) {
        println!("cache hit ({command}) — skipping; inputs unchanged");
        return Ok(());
    }

    let success = run()?;
    cache.record(&key, command, success);
    cache.save_to_dir(root)?;
    Ok(())
}
```

Example for `rocket test`:

```rust
Commands::Test => {
    let root = std::env::current_dir()?;
    let inputs = /* e.g. walk tools/**/*.rs + Cargo.toml via walkdir */;
    cached_run(&root, "cargo test -p rocket-sdk", &inputs, || {
        let status = std::process::Command::new("cargo")
            .args(["test", "-p", "rocket-sdk"]).status()?;
        Ok(status.success())
    })?;
    Ok(())
}
```

Add a `--no-cache` flag that calls `cache.invalidate_all()` (or just skips the
`is_fresh` check) for forced rebuilds. The cache file `.rocket-cache.json`
should be added to `.gitignore`.

---

## API surface (stable entry points)

`cache.rs`
- `Fnv1aHasher`, `hash_bytes(&[u8]) -> String`, `hash_components(iter) -> String`
- `is_binary_asset(&Path) -> bool` (`.uasset`/`.umap`/… → true)
- `FileFingerprint { path, mtime, size, hash: Option<String> }`
- `Fingerprinter { max_hash_bytes }` → `.fingerprint(&Path)`, `.should_content_hash(...)`
- `CacheKey::derive(command, &[FileFingerprint])` (set-order-independent)
- `BuildCache`: `load_from_dir` / `save_to_dir` / `is_fresh` / `get` / `record` /
  `invalidate` / `invalidate_all`

`watch.rs`
- `Snapshot` (= `BTreeMap<PathBuf, FileFingerprint>`), `Changes { created, modified, deleted }`
- `diff(prev, next) -> Changes` (pure)
- `WatchFilter::rocket_defaults()` + `.accepts(&Path)`
- `Watcher::new(roots, filter)` → `.snapshot()`
- `TaskGraph::new(repo_root)` → `.resolve(&[PathBuf])`, `.resolve_one(&Path)`
- `Task::{Command, Skip}`
- `should_fire(elapsed, window) -> bool`, `Debouncer`
- `WatchLoop::new(watcher, graph, interval, debounce)` → `.step(&Snapshot)` (pure)
  / `.run(callback)` (thin loop)

---

## What changed & why it's "1000x"

1. **Incremental cache (skip the build entirely).** `rocket build`/`test` today
   re-runs unconditionally. With a content-addressed `CacheKey` over input
   fingerprints + the command string, an unchanged tree is a sub-millisecond
   `is_fresh` lookup instead of a multi-minute UE4 cook or full `cargo test`.
   The hash is **FNV-1a** (stable across toolchains/platforms, unlike
   `DefaultHasher`), so cache entries survive Rust upgrades.

2. **Unified watch loop (no manual re-runs).** One `rocket watch` replaces
   per-workspace `cargo make watch` + ad-hoc PWA/Python loops. The polling
   watcher works on network FS, containers and atomic-rename editors where
   inotify-based tools miss events — and needs **zero new crates**.

3. **Smart task routing.** Changing `tools/rocket-sdk/src/cache.rs` runs only
   `cargo test -p rocket-sdk`; touching `pwa-staff/src/auth.ts` runs `rocket pwa
   lint`; editing `project-manifest.json` runs `rocket sync`. Tasks are
   **deduplicated**, so 50 edits in one crate produce one test run.

4. **Binary-asset safety baked in.** `.uasset`/`.umap` and other binaries are
   fingerprinted by `(mtime, size)` only — never read into memory or
   content-hashed — and the task graph emits a `Skip` for them, honoring the
   monorepo rule that `versions/**` binaries must never be diffed/merged. A
   configurable size cap (default 8 MiB) also keeps large files metadata-only.

5. **Pure, testable core.** Hashing, key derivation, freshness, snapshot diffing,
   task resolution and debounce are all pure functions with 37 unit tests
   (`tempfile`, no real builds). The only impure surface is the tiny `WatchLoop::run`
   sleep loop.
