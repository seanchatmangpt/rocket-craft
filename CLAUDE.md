# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rocket Craft is a multi-game Unreal Engine 4.24 monorepo containing six UE4 game projects, a Rust SDK and CLI (`rocket-cmd`), a TypeScript PWA (`pwa-staff`), and several independent Rust workspaces (nexus-engine, blueprint-rs, unify-rs, infinity-blade-4/mud, asset-pipeline, chicago-tdd-tools). All project orchestration flows through the `./rocket` shell wrapper, which auto-builds and delegates to `tools/target/release/rocket-cmd`.

## Essential Commands

### Unified CLI (always prefer over ad-hoc scripts)

```bash
./rocket setup          # Bootstrap environment (delegates to rocket-sdk::setup)
./rocket build          # Build UE4 projects (reads project-manifest.json)
./rocket build -p ShooterGame -t ShooterGame -l Win64
./rocket sync           # Sync project manifest with filesystem
./rocket audit          # Semantic law compliance checks (knhk WASM plugins)
./rocket test           # Run all Rust tests + asset validation
./rocket doctor         # Diagnose environment (UE4_ROOT, Blender, Node, etc.)
./rocket info           # Print project manifest summary
./rocket pwa lint       # ESLint + Prettier on pwa-staff
./rocket crypto generate  # Generate Android keystores
./rocket wasm --file path/to/plugin.wasm  # Execute a WASM compliance plugin

# HTML5 pipeline (Brm project — proven working, Stage 6 PASS)
# All verbs accept --project <Name>; archive path derived as /tmp/<name>-html5-archive/HTML5
./rocket html5 preflight --project Brm  # 7-check gate: engine, emsdk, python3, disk≥50GB, uproject, Rosetta, emsdk-python
./rocket html5 cook --project Brm        # UAT BuildCookRun → Brm.wasm (175 MB) in /tmp/brm-html5-archive/HTML5/; auto-verifies + writes receipt
./rocket html5 verify --project Brm      # Verify WASM magic bytes + size + companion files; writes cook-receipt.json
./rocket html5 serve --project Brm       # Serve with COOP/COEP headers (required for SharedArrayBuffer/wasm-threads)
./rocket html5 open --project Brm        # Open served game in browser (finds .html automatically)
./rocket html5 log --lines 50            # Tail latest ue4-cook*.log (monitor running cook)
./rocket html5 status --project Brm      # Pipeline summary: engine, emsdk, package, receipt, port, manifest, cook log
./rocket html5 pipeline --project Brm   # One-shot: preflight → cook → verify in sequence (exits on first failure)
./verify_html5_pipeline.sh               # Full Stage 6 proof: cook → serve → Playwright → receipt PASS
# Playwright config: pwa-staff/playwright.html5.config.ts (headless:false, Metal GPU WebGL2, timeout:240s)
# Receipt: pwa-staff/test-results/tps-dflss-receipt.json (verdict=PASS, 362762 non-black pixels proven)
# Stage 6 proven 2026-06-19: real UE4 WebGL2 on Apple M3 Metal, 175.4 MB Brm.wasm
```

### Per-Workspace Rust

Each workspace is independent; run commands from its directory or with `--manifest-path`.

```bash
# tools workspace (rocket-sdk, rocket-cmd, knhk, unrdf)
cd tools && cargo build --release
cd tools && cargo test --all
cd tools && cargo clippy -- -D warnings
cd tools && cargo fmt

# nexus-engine (Gundam Nexus game engine, 10 crates)
cd nexus-engine && cargo test --all
cd nexus-engine && cargo test -p nexus-combat
cd nexus-engine && cargo test <test_name>   # run a single test by name

# blueprint-rs (UE4 Blueprint AST / T3D generation)
cd blueprint-rs && cargo build
cd blueprint-rs && cargo test --all

# unify-rs (17-crate semantic/RDF/MCP ecosystem)
cd unify-rs && cargo build
cd unify-rs && cargo test --all

# infinity-blade-4 MUD backend
cd infinity-blade-4/mud && cargo test --all

# chicago-tdd-tools (BDD framework)
cd chicago-tdd-tools && cargo test --all-features

# asset-pipeline (in git worktree at .claude/worktrees/agent-a63d171fb05007da1/)
cd .claude/worktrees/agent-a63d171fb05007da1/asset-pipeline && cargo build
```

### PWA (pwa-staff)

```bash
cd pwa-staff && npm ci
cd pwa-staff && npm test          # vitest unit tests
cd pwa-staff && npm run lint      # eslint
cd pwa-staff && npx tsc --noEmit  # type-check
cd pwa-staff && npm run build     # esbuild TS + postcss CSS → dist/
cd pwa-staff && npm start         # local-web-server on :3000
```

### Validation & Registry Tools

```bash
./rocket test                     # Run all test suites + native asset validation
./rocket doctor                   # Run programmatic environment & dependency verification
./copy_catalogue                  # Consolidate ontology files (delegates to `rocket registry copy`)
./index_o_crates                  # Index O-crates (delegates to `rocket registry index`)
```

### Asset Pipeline (in worktree)

```bash
./target/debug/asset-pipeline --config pipeline.toml watch
./target/debug/asset-pipeline --config pipeline.toml once --dir /path/to/models
```

## Architecture Overview

### The `Machine<Law, Phase>` Typestate Pattern

The dominant architectural pattern across all Rust workspaces. States are zero-sized `PhantomData<S>` type parameters; illegal transitions are compile errors because the required `impl` block simply does not exist.

Canonical example — `nexus-net/src/connection.rs`:
```
Disconnected → Handshaking → Connected → Authenticated → InLobby → InMatch
```
Each state has only the transition methods that are legal from that state. The same pattern appears in:
- `nexus-engine/crates/nexus-combat/src/machine.rs` (`CombatMachine<Idle/Attacking/Parrying/…>`)
- `unify-rs/unify-rdf/src/project_bridge.rs` (`ProjectManifest<Pending/Ingested/Validated>`)
- `tools/rocket-sdk` (`Machine<Law, Phase>` with Law traits for domain rules)

To simplify constructing these machines safely, builders are provided (e.g. `CombatMachineBuilder`, `PlayerSessionBuilder`, `ConnectionBuilder`, `AuctionBuilder`, `MechBuilder`, `CivilizationBuilder`, `MechAssemblySpecBuilder`). For dynamic, network-driven state transitions, runtime state enums (`CombatState`, `SessionState`, etc.) and transition errors (e.g., `CombatTransitionError`) describe failures cleanly.

The `rocket-sdk` formalises this as `Machine<L: Law, P>` where `L` is a trait that defines `validate()` and `P` is a phase struct (Input → Validated → Admitted). `ggen` code-generates these skeletons from Ostar ontology definitions.

### `rocket-cmd` / `rocket-sdk` — The CLI Spine

`tools/rocket-cmd/src/main.rs` is the binary entry point (clap 4 derive). It delegates to `rocket-sdk` modules for all business logic (`setup`, `manifest`, `crypto`, `doctor`). The `knhk` crate provides semantic law enforcement via Wasmer 4: laws compile to WASM and are loaded at audit time. `unrdf` provides RDF triple-store logic for `project-manifest.json` ingestion and SPARQL-style queries.

### Blueprint Generation Pipeline

`blueprint-rs` provides two layers:
1. **Low-level AST** (`blueprint-core/src/ast.rs`, `types.rs`) — full UE4 K2 pin/node graph, round-trippable to T3D text format.
2. **High-level builder** (`BlueprintBuilder`) — consumed by `blueprint-macros` proc-macro crate.

`BlueprintBuilder` → `T3dSerializer` / `JsonSerializer` → `.uasset`-compatible T3D text output.

### Gundam Nexus Engine (`nexus-engine`)

10-crate workspace; `nexus-types` has zero internal dependencies and is the root of the dependency graph. Load order: `nexus-types` → `nexus-combat`, `nexus-session`, `nexus-economy`, `nexus-ecs`, `nexus-gfx`, `nexus-shop`, `nexus-net` → `nexus-integration` → `nexus-tests`.

- **nexus-types**: Phantom-typed units (`Hp`, `Gold`, `Damage`), strongly-typed IDs (`PlayerId`, `SessionId`), all typestate markers, `GameError`/`TypeError`.
- **nexus-combat**: `CombatMachine<S>` typestate, combo system, parry/dodge resolution, damage calculation.
- **nexus-net**: Typestate `Connection<S>` (Disconnected→InMatch), duel matchmaking, room state, serde_json message codec.
- **nexus-tests**: `proptest` strategies for all domain types + property-based invariant tests.

### unify-rs — Semantic Web / MCP Layer

17-crate workspace. Key crates:
- **unify-rdf**: RDF triple store, SPARQL pipeline, SHACL validation, project manifest bridge (also uses `Pending/Ingested/Validated` typestate).
- **unify-mcp**: JSON-RPC MCP server (`McpServer` builder pattern), tool and resource registries, `rocket_tools.rs` exposes `rocket/manifest/list` and related tools that read `project-manifest.json` from cwd.
- **unify-lsp**: LSP capability/diagnostic gate and conformance compositor.
- **unify-codegen**, **unify-macros**: Code generation layer.

### PWA (`pwa-staff`)

TypeScript PWA with esbuild bundler. Service worker at `worker.ts`. Auth managed via `src/auth.ts` — wraps `@supabase/supabase-js`, fires `auth-change` CustomEvents. Other modules (`leaderboard.ts`, `hud.ts`, `admin.ts`) listen for those events. Compiled artifacts go to `dist/`. No framework; vanilla TS DOM manipulation + Supabase client.

### Infinity Blade 4 MUD Backend

`infinity-blade-4/mud/` — 6-crate Rust workspace (`ib4-core`, `ib4-combat`, `ib4-progression`, `ib4-ai`, `ib4-mud`, `ib4-integration-tests`). Standalone text-based MUD server for IB4's backend; no dependency on nexus-engine.

## Rust Workspaces

| Workspace | Root | Crates | Resolver |
|---|---|---|---|
| tools | `tools/` | rocket-sdk, rocket-cmd, knhk, unrdf, un-test-utils | 3 |
| nexus-engine | `nexus-engine/` | nexus-types, nexus-combat, nexus-session, nexus-economy, nexus-net, nexus-ecs, nexus-gfx, nexus-shop, nexus-integration, nexus-tests | 2 |
| blueprint-rs | `blueprint-rs/` | blueprint-core, blueprint-cli, blueprint-macros, blueprint-testing | 2 |
| unify-rs | `unify-rs/` | unify-core, unify-sem, unify-admission, unify-receipts, unify-rdf, unify-lsp, unify-test, unify-ffi, unify-config, unify-mcp, unify-bp, unify-integration-tests, unify-ocel, unify-pm, unify-rocket, unify, unify-codegen/macros | 2 |
| ib4-mud | `infinity-blade-4/mud/` | ib4-core, ib4-combat, ib4-progression, ib4-ai, ib4-mud, ib4-integration-tests | 2 |
| chicago-tdd-tools | `chicago-tdd-tools/` | chicago-tdd-tools | — |
| asset-pipeline | `.claude/worktrees/…/asset-pipeline/` | pipeline-core, pipeline-cli | — |

## Unreal Engine Projects

| Name | Path | Targets | Platforms |
|---|---|---|---|
| ShooterGame | `versions/4.24-Shooter/ShooterGame/` | ShooterGameEditor, ShooterClient, ShooterGame, ShooterServer | Win64, HTML5 |
| SurvivalGame | `versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/` | SurvivalGameEditor, SurvivalGameServer, SurvivalGame | Win64, Android, HTML5 |
| Brm | `versions/4.24.0/` | BrmEditor, BrmServer, Brm | Win64, Android |
| RealisticRendering | `versions/Realistic/RealisticRendering/` | (none) | Win64 |
| FullSpectrum | `versions/Template/FullSpectrum/` | (none) | Win64 |
| InfinityBlade4 | `infinity-blade-4/` | InfinityBlade4, InfinityBlade4Editor | iOS, Android, Win64 |

All projects and their `uproject_path`/`targets` are the authoritative source in `project-manifest.json`.

## Environment Setup

| Tool | Required | Notes |
|---|---|---|
| Rust stable | Yes | `rustup`; `tools/` uses edition 2024 |
| Node.js 20.x | Yes | `pwa-staff` only |
| Python 3.x | Yes | Only used inside Blender for asset-pipeline (.py scripts) |
| Unreal Engine 4.24 | Yes (UE builds) | Set `UE4_ROOT`; uses UE4.24.3-HTML5 build with WebSocketNetworking + VaRest plugins |
| Blender | Yes (asset-pipeline) | Set `BLENDER_PATH` or ensure `blender` in PATH; macOS: `/Applications/Blender.app/Contents/MacOS/blender` |
| Android SDK | Optional | Only for Android platform builds |
| Docker | Optional | Local Supabase (`supabase start`) at `127.0.0.1:54321` |

Rocket Doctor checks the configured `UE4_ROOT` path and verifies local plugin presence.

HTML5 UE4 networking standardises on port **8889** via `WebSocketNetworking` plugin. In production, Nginx terminates TLS on 443 and proxies to `ws://localhost:8889`.

## Development Workflow

### Making Changes

1. Work on branch `claude/inspiring-hamilton-jfww9e`; never force-push main/master.
2. Commit message format: `feat(scope): description` or `fix(scope): description`.
3. Before committing: run `./rocket test` (blocks refs to `Highrise` or `Brm-HTML5-Shipping` using native asset validation).
4. For UE-plugin changes: run `./rocket doctor` to diagnose environment setup.
5. `./rocket audit` runs semantic law compliance via WASM-loaded knhk plugins.

### CI (`.github/workflows/ci.yml`)

Runs on every push. Two jobs:
- **pwa-staff**: `npm ci` → lint → `tsc --noEmit` → `vitest run` → `npm run build`
- **chicago-tdd-tools**: `cargo build --all-features` → `cargo test --all-features`

Other workspaces are not yet in CI; test them manually before pushing.

## Code Conventions

### Rust

- **Error handling**: `thiserror` for typed domain errors (all workspaces define their own `Error` enum); `anyhow` + `color-eyre` for top-level CLI context chaining in `rocket-cmd`.
- **Async**: `tokio` with `features = ["full"]` throughout; `tokio::sync::broadcast` for multi-subscriber event buses.
- **Testing**: `proptest` for property-based tests in `nexus-tests/src/strategies.rs` + `invariants.rs`; integration tests live in `tests/` subdirs or dedicated `*-integration-tests` / `*-tests` crates.
- **CLI**: All binaries use `clap` 4.x with derive macros.
- **Typestate**: New state machines must use zero-sized `PhantomData<S>` markers. Never add runtime state guards where a typestate transition suffices.
- **Workspace deps**: Declare shared deps once in `[workspace.dependencies]` and reference with `dep = { workspace = true }` in member `Cargo.toml` files (nexus-engine pattern).

### TypeScript (pwa-staff)

- Bundled with `esbuild`; no framework.
- Auth events flow as `CustomEvent('auth-change')` on `window`; modules must not poll Supabase directly.
- `prettier` + `eslint-config-prettier` enforced; run `npm run lint` before committing.

## Gotchas and Caveats

- **`versions/` is binary**: `.uasset` and `.umap` files are Unreal binary assets. Never `git add` them to diffs or try to merge them. They must be managed with Unreal's built-in source control or copied wholesale.
- **UE4 HTML5 limitations**: Apex destruction and procedural meshes (`ProceduralMeshComponent`) are not supported in HTML5 builds. Avoid adding them to projects targeting HTML5.
- **`wasm4pm-compat` path dep**: `tools/knhk/Cargo.toml` and `tools/rocket-cmd/Cargo.toml` contain `path = "/Users/sac/wasm4pm-compat"` — an absolute local path. This breaks CI and other machines. If you need to build `tools/` on a different machine, either mock the dep or adjust the path.
- **`clap-noun-verb` external path**: `rocket-cmd/Cargo.toml` also has `path = "../../../clap-noun-verb"`. Same issue as above.
- **PMX model conversion** (asset-pipeline): Requires the `mmd_tools` Blender addon installed. Without it, `.pmx` files will fail conversion silently.
- **`supabase start`** requires Docker Desktop running. The local anon key (`sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`) is safe to commit; it only works against the local instance.
- **`rocket` binary rebuild**: The `./rocket` wrapper only rebuilds `rocket-cmd` if `tools/target/release/rocket-cmd` does not exist. After source changes in `tools/`, run `cd tools && cargo build --release` manually or delete the binary to force a rebuild.
- **Resolver 3 in `tools/`**: The workspace uses `resolver = "3"` (Rust 2024 edition). Other workspaces use `resolver = "2"`. Do not mix these within a single workspace.

## Asset Pipeline Quick Start

The `asset-pipeline` workspace (in `.claude/worktrees/agent-a63d171fb05007da1/asset-pipeline/`) provides autonomous 3D-model-to-UE4-FBX conversion.

```bash
cd .claude/worktrees/agent-a63d171fb05007da1/asset-pipeline
cargo build

# Continuous watch mode
./target/debug/asset-pipeline --config pipeline.toml watch

# One-shot batch over a directory
./target/debug/asset-pipeline --config pipeline.toml once --dir /path/to/models
```

Supported input formats: `.obj`, `.fbx`, `.stl`, `.dae`, `.gltf`, `.glb`. Default size limit: 500 MB. Converted FBX files are staged to the UE4 `Content/Assets/` directory. A `pipeline-manifest.json` run log is written to the working directory after each run. Configuration lives entirely in `pipeline.toml`; no flags are needed at runtime once it is configured.
