# Doctor 1000x — Wiring Guide

This doc tells you exactly what to change OUTSIDE of `tools/rocket-sdk/src/doctor.rs`
to expose the upgraded `rocket doctor`. The doctor module itself is fully
self-contained and backward compatible — nothing is required for the existing
`rocket doctor` to keep working.

## 1. `tools/rocket-sdk/src/lib.rs`

**No change required.** `pub mod doctor;` already exists and everything lives in
the single `doctor.rs` file (std-only, no submodules).

## 2. `tools/rocket-sdk/Cargo.toml`

**No new dependencies.** Everything is built on `std` (`std::thread::scope`,
`std::process::Command`, hand-rolled version parsing, `df -kP` for disk space)
plus the already-present `serde`, `serde_json`, `chrono` (serde feature already
enabled), and `git2`. Nothing to add.

## 3. `tools/rocket-cmd/src/main.rs`

The new public API on the doctor module:

```rust
// types
rocket_sdk::doctor::RocketDoctor::new(PathBuf) -> RocketDoctor
RocketDoctor::run_diagnostics(&self) -> DiagnosticReport      // unchanged signature
RocketDoctor::run_fixes(&self, dry_run: bool) -> Vec<FixResult>

// DiagnosticReport
report.health_score() -> u8            // 0..=100, weighted (Optional checks weigh 0.5)
report.overall_status() -> CheckStatus // Pass / Warn / Fail
report.counts() -> (usize, usize, usize)  // (pass, warn, fail)
report.to_json() -> String             // pretty JSON incl. score + overall status
report.compact_summary(use_color: bool) -> String  // one line
report.pretty(use_color: bool) -> String           // sectioned human report

// FixResult { check_name, command, outcome: FixOutcome, message }
// FixOutcome = Planned | Applied | Skipped | Failed
```

### 3a. Add flags to the `doctor` subcommand (clap derive)

Find the `Doctor` variant of your `Commands` enum and replace it with:

```rust
/// Run environment & project health diagnostics.
Doctor {
    /// Emit the report as JSON instead of the pretty table.
    #[arg(long)]
    json: bool,

    /// One-line summary only.
    #[arg(long)]
    quiet: bool,

    /// Attempt safe auto-fixes (cargo fetch, create missing dirs, …).
    #[arg(long)]
    fix: bool,

    /// With --fix: only print what WOULD be done (this is the default for safety).
    /// Pass --no-dry-run to actually apply fixes.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    dry_run: bool,

    /// Disable ANSI color output.
    #[arg(long)]
    no_color: bool,
},
```

(If your clap setup doesn't like a defaulted bool flag, use
`#[arg(long)] apply: bool,` and treat `dry_run = !apply` — see handler below.)

### 3b. Handler

```rust
Commands::Doctor { json, quiet, fix, dry_run, no_color } => {
    let root = std::env::current_dir()?;
    let doctor = rocket_sdk::doctor::RocketDoctor::new(root);
    let use_color = !no_color && atty_stdout(); // or just `!no_color`

    if fix {
        let fixes = doctor.run_fixes(dry_run);
        for f in &fixes {
            println!("[{:?}] {} -> {}", f.outcome, f.command, f.message);
        }
        if dry_run {
            println!("(dry-run: nothing was applied; pass --no-dry-run to apply)");
        }
        return Ok(());
    }

    let report = doctor.run_diagnostics();

    if json {
        println!("{}", report.to_json());
    } else if quiet {
        println!("{}", report.compact_summary(use_color));
    } else {
        print!("{}", report.pretty(use_color));
    }

    // Non-zero exit when overall health failed — handy for CI/pre-commit.
    use rocket_sdk::doctor::CheckStatus;
    if report.overall_status() == CheckStatus::Fail {
        std::process::exit(1);
    }
    Ok(())
}
```

`atty_stdout()` is optional; if you don't already depend on a tty crate, just use
`!no_color`.

## 4. What changed & why it's 1000x

| Before | After |
|---|---|
| 9 sequential checks | 19 checks run **concurrently** via `std::thread::scope` (deterministic output order) |
| version *presence* only | **version-range validation** (rustc ≥1.80, node ≥20, python ≥3) + cargo/clippy/rustfmt/blender(BLENDER_PATH)/node detection |
| no project-wide awareness | checks **all 6 Rust workspace manifests**, **pwa-staff/node_modules**, **disk space** (`df`), **network/offline flag**, and flags the **absolute-path dep gotchas** (`wasm4pm-compat` / `clap-noun-verb`) |
| flat list, no severity | **categories** (Environment/Toolchain/Project/Git/Optional) + **weighted 0-100 health score** + **overall status** |
| messages only | every Warn/Fail carries a concrete **`fix_command`** |
| no remediation | **`run_fixes(dry_run)`** executes a whitelisted, non-destructive subset (cargo fetch, mkdir) — dry-run by default, never runs arbitrary commands |
| `{:?}` debug print | three output modes: **pretty** (glyphs, sections, optional ANSI color), **compact** one-liner, **JSON** (serde, includes score + counts) |

Backward compatibility: `RocketDoctor::new`, `run_diagnostics`,
`DiagnosticReport { timestamp, checks }`, `CheckResult { name, status, message,
details, .. }` and `CheckStatus { Pass, Warn, Fail }` are all preserved. New
struct fields (`category`, `fix_command`) are additive with serde defaults, so
existing serialization consumers keep working. All original unit tests pass
unchanged.
