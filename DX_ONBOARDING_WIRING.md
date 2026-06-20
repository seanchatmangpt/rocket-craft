# DX Onboarding Wizard — Wiring Guide

A first-run onboarding / setup wizard for the `rocket` CLI lives in a single new
file: `tools/rocket-sdk/src/wizard.rs`. It is self-contained, fully unit-tested,
and depends only on crates already in `tools/Cargo.lock` (`serde`, `serde_json`,
`anyhow`, `inquire`). **No new dependencies are required.**

This document lists the small, mechanical edits needed to expose it through the
CLI. The wizard author owns only `wizard.rs`; the edits below are for the
maintainer who wires it in.

---

## 1. `tools/rocket-sdk/src/lib.rs`

Add the module next to the other `pub mod` declarations:

```rust
pub mod wizard;
```

That's the only change to `lib.rs`. Everything the CLI needs is re-exported from
`rocket_sdk::wizard`.

---

## 2. Dependencies

None to add. The wizard reuses:

| Crate        | Already in `rocket-sdk/Cargo.toml`? | Used for                         |
|--------------|-------------------------------------|----------------------------------|
| `serde`      | yes                                 | `WizardConfig` (de)serialization |
| `serde_json` | yes                                 | merge-safe `.rocket.json` rewrite|
| `anyhow`     | yes                                 | error propagation                |
| `inquire`    | yes (0.7.5)                         | `StdinPrompter` real prompts     |
| `tempfile`   | yes (dev-dependency)                | tests                            |

`inquire` 0.7.5 and `tempfile` 3.x are confirmed present in `tools/Cargo.lock`.

---

## 3. `tools/rocket-cmd/src/main.rs`

### 3a. Add an `init` subcommand and enhance `setup`

In the clap `Commands` enum:

```rust
/// First-run onboarding wizard: detect tools, configure .rocket.json, print next steps.
Init {
    /// Take all smart defaults; never prompt.
    #[arg(long)]
    non_interactive: bool,
    /// Show what would change without writing .rocket.json.
    #[arg(long)]
    dry_run: bool,
},

/// Validate / configure the UE4 + SDK environment.
Setup {
    /// Run the guided interactive wizard (same engine as `init`).
    #[arg(long)]
    interactive: bool,
    /// Take all smart defaults; never prompt.
    #[arg(long)]
    non_interactive: bool,
},
```

> If `Setup` already exists as a unit variant, change it to the struct form above.
> The existing `rocket_sdk::setup::run_setup()` remains valid for the legacy,
> UE4-only path; the wizard is the superset.

### 3b. Dispatch handlers

```rust
use rocket_sdk::wizard::{
    detect_env, next_steps, render_env, render_next_steps, StdinPrompter, Wizard,
};
use std::path::PathBuf;

fn run_wizard(non_interactive: bool, dry_run: bool) -> anyhow::Result<()> {
    let env = detect_env();
    println!("{}", render_env(&env));

    // StdinPrompter is fine for both modes: in non-interactive mode every step
    // short-circuits on `self.non_interactive` and the prompter is never called.
    let mut wiz = Wizard::new(StdinPrompter, env.clone(), PathBuf::from(".rocket.json"))
        .non_interactive(non_interactive);

    let plan = wiz.plan()?;

    if dry_run {
        println!("Planned changes (dry run, nothing written):");
        for c in &plan.changes {
            println!("  {} : {:?} -> {}", c.key, c.from, c.to);
        }
    } else {
        wiz.apply(&plan)?;
        println!("Wrote {}", plan.config_path.display());
    }

    println!("{}", render_next_steps(&next_steps(&env, &plan.config)));
    Ok(())
}
```

Match arms:

```rust
Commands::Init { non_interactive, dry_run } => run_wizard(non_interactive, dry_run)?,
Commands::Setup { interactive, non_interactive } => {
    if interactive || non_interactive {
        run_wizard(non_interactive, /*dry_run=*/ false)?;
    } else {
        rocket_sdk::setup::run_setup()?; // legacy UE4-only path
    }
}
```

---

## 4. Public API surface (what you can call)

From `rocket_sdk::wizard`:

- `detect_env() -> DetectedEnv` — real autodetection (Rust, Cargo, Node, Python,
  Blender, Docker, Java, UE4 candidates). `detect_env_with(probe, validator)` is
  the testable seam.
- `Wizard::new(prompter, env, config_path)` `.non_interactive(bool)`.
- `Wizard::plan() -> Plan` — pure, no writes (dry-run-able).
- `Wizard::apply(&Plan) -> WizardConfig` — merge-writes `.rocket.json`.
- `Wizard::run() -> (Plan, WizardConfig)` — plan + apply.
- `next_steps(&DetectedEnv, &WizardConfig) -> Vec<NextStep>` and
  `render_next_steps`, `render_env`, `summary` for output.
- `Prompter` trait with `StdinPrompter` (real) and `MockPrompter` (tests).
- `WizardConfig` — the wizard-owned slice of `.rocket.json`
  (`ue4_root`, `blender_path`, `build_html5`, `build_android`, `supabase_mode`,
  `html5_port` (default 8889), `projects`).

---

## 5. What changed & why it's 1000x

**Before:** `rocket setup` only located a UE4 root and wrote `ue4_root` to
`.rocket.json`, prompting via raw `inquire` calls baked directly into the logic
(untestable without stdin). Everything else — Node, Python, Blender, Docker,
Java, HTML5/Android intent, Supabase mode, which projects you care about — was
tribal knowledge spread across the root `CLAUDE.md`. A new contributor had to
read docs, install tools by hand, guess the HTML5 port, and discover `doctor`,
`crypto generate`, and `supabase start` on their own.

**After:** one command (`rocket init`, or `rocket setup --interactive`) takes a
fresh clone to productive:

1. **Autodetects the whole toolchain** in one structured `DetectedEnv`, with
   versions, locations, and exact install commands for anything missing.
2. **Guides** through the handful of real decisions (UE4 root, Blender, HTML5 /
   Android targets, Supabase local vs remote, project selection) — every step has
   a smart default from detection and is skippable.
3. **Writes `.rocket.json` without clobbering** keys it does not own (merge over
   the raw JSON value, so team/custom settings survive).
4. **Emits a personalized next-steps checklist** — blocking items first (install
   missing required tools, configure UE4), then the exact commands to build a
   first project, generate an Android keystore, start Supabase, etc.
5. **Idempotent & resumable**: re-running reads the existing config, fills only
   the gaps, and is a verified no-op when complete — safe to run anytime.
6. **Fully testable**: `plan()`/`apply()` separation makes it dry-run-able, and
   the `Prompter` trait + `CommandProbe` seam mean the entire flow (detection,
   resume, merge, idempotency, next-steps) is unit-tested with **zero stdin,
   network, or real UE4 install** (13 tests, all green).

The multiplier is in removing the human round-trips: detection replaces "read the
docs and check each tool", smart defaults replace "guess the right value", the
merge replaces "don't break my config", and the checklist replaces "now what?".
Zero-to-productive collapses from an afternoon of doc-spelunking into a single
command.
