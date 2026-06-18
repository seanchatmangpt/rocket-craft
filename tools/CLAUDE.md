# tools — CLAUDE.md

## Purpose

Core Rust SDK and CLI for the entire Rocket Craft monorepo. Provides project manifest
loading, UE4 environment setup/validation, Android keystore crypto, Supabase
integration, architectural law enforcement via WASM plugins (`knhk`), and an RDF
state-machine manifest for Unreal projects (`unrdf`). The `rocket-cmd` binary is the
primary developer-facing entry point for the monorepo — it wraps everything.

## Directory Structure

```
tools/
├── Cargo.toml          # Workspace root (resolver = "3")
├── Cargo.lock
├── Makefile.toml       # cargo-make task definitions
├── rocket-sdk/         # Library: SDK core
│   └── src/
│       ├── lib.rs
│       ├── manifest.rs     # project-manifest.json loader
│       ├── project.rs      # UE4 project representation
│       ├── setup.rs        # UE4 installation validation
│       ├── config.rs       # Env diagnostics, .ini parsing
│       ├── crypto.rs       # Android keystore generation (rcgen, p12-keystore)
│       ├── supabase.rs     # Supabase REST client (reqwest + tokio)
│       ├── pwa.rs          # PWA build helpers
│       ├── doctor.rs       # Environment health checks
│       └── error.rs        # Unified error types
├── rocket-cmd/         # Binary: CLI wrapper
│   └── src/
│       ├── main.rs         # clap subcommands wiring
│       └── compliance.rs   # knhk compliance checks
│   └── tests/
│       └── uat_mock_test.rs
├── knhk/               # Library: semantic law enforcer (WASM plugins)
│   └── src/
│       ├── lib.rs
│       └── plugin.rs       # wasmer WASM plugin loader
├── unrdf/              # Library: Unreal RDF state-machine manifest
│   └── src/
│       └── lib.rs          # Pending → Ingested → Validated typestate
└── un-test-utils/      # Library: test helpers shared across workspaces
    └── src/
        └── lib.rs
```

## Key Commands

### cargo-make (preferred CI runner)

```bash
# Install cargo-make if needed
cargo install cargo-make

# Full CI pipeline: fmt-check + clippy + test + audit
cargo make ci

# Development cycle: fmt + clippy + test
cargo make dev

# Format all code
cargo make fmt

# Run clippy (deny warnings)
cargo make clippy

# Run all tests
cargo make test

# Build release
cargo make build

# Security audit
cargo make audit

# Generate docs
cargo make doc

# Watch-mode tests
cargo make watch
```

### Direct cargo commands

```bash
# Run the rocket CLI (setup subcommand)
cargo run -p rocket-cmd -- setup

# Run doctor (environment health)
cargo run -p rocket-cmd -- doctor

# Run audit
cargo run -p rocket-cmd -- audit

# Run all other subcommands:
# sync | build | run | crypto | clean | pwa | info | test | logs | capabilities | wasm

# Run specific tests
cargo test -p rocket-sdk
cargo test -p rocket-cmd
```

### Via the shell wrapper

```bash
# From the monorepo root:
./rocket <subcommand>    # Linux/macOS
rocket.bat <subcommand>  # Windows
```

## Crate Responsibilities

### `rocket-sdk` (library)

| Module          | Responsibility                                                            |
|-----------------|---------------------------------------------------------------------------|
| `manifest.rs`   | Deserializes `/rocket-craft/project-manifest.json`; lists all UE4 projects|
| `project.rs`    | `UE4Project` type, target enumeration, platform list                      |
| `setup.rs`      | Validates UE4 installation path, `RunUAT.sh` presence, engine version    |
| `config.rs`     | Reads `.ini` files (UE4 `Config/` dirs), environment variable diagnostics |
| `crypto.rs`     | Generates Android `.keystore` files via `rcgen` + `p12-keystore`         |
| `supabase.rs`   | Thin async Supabase REST wrapper: auth, table queries, storage             |
| `pwa.rs`        | Helpers for `pwa-staff/` build coordination                               |
| `doctor.rs`     | Checks: UE4 path, Android SDK, NDK, Java, Blender, Node.js, Rust toolchain|
| `error.rs`      | `SdkError` enum via `thiserror`                                           |

### `rocket-cmd` (binary)

clap subcommands — each maps to one or more `rocket-sdk` functions:

| Subcommand     | What it does                                              |
|----------------|-----------------------------------------------------------|
| `setup`        | Validates UE4 + SDK environment                           |
| `sync`         | Triggers asset pipeline, pulls latest manifest            |
| `build`        | Runs `RunUAT.sh` or `UnrealBuildTool` for a target        |
| `audit`        | Security / compliance scan via `knhk` laws                |
| `run`          | Launches a packaged game or the UE4 editor                |
| `crypto`       | Generates Android keystores                               |
| `clean`        | Removes build artifacts (`Binaries/`, `Intermediate/`)    |
| `pwa`          | Builds the `pwa-staff/` TypeScript bundle                 |
| `info`         | Prints manifest summary                                   |
| `test`         | Runs workspace tests                                      |
| `logs`         | Tails UE4 log files                                       |
| `doctor`       | Runs environment health checks                            |
| `capabilities` | Reads `capabilities/CapabilityManifest.md`                |
| `wasm`         | Loads and validates knhk WASM plugins                     |

### `knhk` (library)

Loads architectural law plugins (`.wasm` files) via `wasmer`. Each plugin exposes a
`check(manifest: &str) -> bool` function. Plugins are rereaddressed from the `capabilities/`
directory. Laws enforce things like "no circular crate dependencies" or "all UE4 targets
must have a keystore."

### `unrdf` (library)

Typestate manifest for Unreal projects:

```
Pending → Ingested → Validated
```

Tracks which projects have been RDF-described and validated by `unify-rs`.

### `un-test-utils` (library)

Shared test helpers: mock Supabase responses, temp directory fixtures, assertion
macros. Add as `dev-dependency` in any workspace crate needing test scaffolding.

## Key Dependencies

| Crate            | Used in       | Purpose                          |
|------------------|---------------|----------------------------------|
| `clap`           | rocket-cmd    | CLI argument parsing             |
| `ratatui`        | rocket-cmd    | TUI progress display             |
| `wasmer`         | knhk, rocket-cmd | WASM plugin runtime           |
| `reqwest`        | rocket-sdk    | Async HTTP (Supabase)            |
| `tokio`          | rocket-sdk    | Async runtime                    |
| `rcgen`          | rocket-sdk    | Certificate generation           |
| `p12-keystore`   | rocket-sdk    | Android keystore format          |
| `walkdir`        | rocket-sdk    | Directory traversal              |
| `git2`           | rocket-sdk    | Git repo inspection              |
| `inquire`        | rocket-sdk    | Interactive prompts              |
| `ratatui`        | rocket-cmd    | Terminal UI widgets              |

## Relation to the Monorepo

- **`project-manifest.json`** (monorepo root) — `rocket-sdk/manifest.rs` is the
  canonical reader; all other tools should call this rather than parsing it directly.
- **`versions/`** — `rocket-cmd build` targets the UE4 projects in this directory.
- **`pwa-staff/`** — `rocket-cmd pwa` drives the TypeScript build there.
- **`unify-rs/unrdf`** — the `unrdf` crate here is the simpler state-machine; the
  fuller RDF triple store lives in `unify-rs/unify-rdf`.
- **`knhk` laws** reference `capabilities/CapabilityManifest.md` and WASM plugins
  at paths configured in the manifest.

## Caveats and Gotchas

- **Resolver 3**: This workspace uses `resolver = "3"` (Rust 2024 edition resolver).
  Other workspaces in the monorepo use resolver 2. Do not copy the workspace
  `Cargo.toml` between them without adjusting the resolver field.
- **External paths in `rocket-cmd`**: `wasm4pm-compat` and `clap-noun-verb` are
  currently referenced as external paths (`/Users/sac/...`). These must be vendored
  or replaced with crates.io versions before CI can run on other machines.
- **`cargo make` is required for the full CI pipeline**: plain `cargo test` works
  but won't run the audit step. Install with `cargo install cargo-make`.
- **Keystore placeholders**: `.keystore.placeholder` files in the monorepo root are
  not real keystores. Run `rocket-cmd crypto` to generate real ones; store them
  outside the repo.
- **`un-test-utils` must stay a dev-dependency**: never add it as a non-dev dependency.
