# Technical Architecture & Roadmap 2026-2030

**Rocket Craft** is a multi-discipline game development platform integrating Unreal Engine 4 projects, formal type-safe Rust backends, semantic web infrastructure (RDF/SPARQL), and browser-native game delivery via HTML5/WASM. This document synthesizes architectural decisions, critical constraints, and the manufacturing pipeline that drives verification.

---

## 1. Type System Foundation

### 1.1 Zero-Cost Typestate Kernel

The bedrock of Rocket Craft is **compile-time enforced state machines via zero-sized phantom types**. No runtime overhead; illegal state transitions are compiler errors.

#### Core Pattern: `Machine<L: Law, P>`

```rust
pub struct Machine<L, P> {
    _law: std::marker::PhantomData<L>,
    pub phase: P,
}
```

- **`L` (Law)**: A marker trait defining domain rules and validation semantics
- **`P` (Phase)**: Represents the typestate (e.g., `Input → Validated → Admitted`)

Transitions consume self and return a new `Machine<L, NextPhase>`, preventing stale references and enforcing ordering:

```rust
impl<L: MyLaw> Machine<L, Input> {
    pub fn validate(self) -> Result<Machine<L, Validated>, Error> { ... }
}
impl<L: MyLaw> Machine<L, Validated> {
    pub fn admit(self) -> Result<Machine<L, Admitted>, Error> { ... }
}
// Compile error: no direct path from Input to Admitted
```

#### Instantiations Across the Monorepo

| Workspace            | State Machine           | Phases                                    |
|----------------------|-------------------------|-------------------------------------------|
| `rocket-sdk`         | `Machine<Law, Phase>`   | `Input → Validated → Admitted`            |
| `nexus-engine`       | `CombatMachine<S>`      | `Idle → Attacking → Resolving → Completed`|
|                      | `Connection<S>`         | `Disconnected → Handshaking → Connected → Authenticated → InLobby → InMatch` |
| `unify-rs/unify-rdf` | `ProjectManifest<S>`    | `Pending → Ingested → Validated`          |
| `asset-pipeline`     | `AssetPipeline`         | `Discovered → Validated → Converted → Staged` |

**Risk**: The pattern scales linearly with the number of domain state machines. As new game subsystems are added, maintain consistent naming (`*Machine<S>` convention) and document phase diagrams in crate-level rustdocs.

### 1.2 Phantom-Typed Units

All numeric quantities are encoded with phantom-type markers, preventing dimensional errors at compile time.

**Example**: `nexus-types/src/units.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hp(i32);
impl Hp { pub const MAX: i32 = 1000; }

#[derive(Debug, Clone, Copy)]
pub struct Damage(i32);

#[derive(Debug, Clone, Copy)]
pub struct Armor(i32);

// Type mismatch: cannot accidentally add Hp + Armor
let health: Hp = Hp(100);
let armor: Armor = Armor(10);
// let result = health + armor; // ← Compile error
```

**Scope**: `nexus-types` defines 8 phantom units (`Hp`, `Damage`, `Gold`, `Mana`, `Armor`, `Xp`, `ComboMultiplier`, `TimeDilation`). All game math flows through these types.

**Constraints**: 
- New units must be added to `nexus-types` (the zero-dependency root crate).
- All arithmetic must preserve units. Use helper methods like `Damage::to_hp_reduction(armor)` for dimension conversion.
- Do not create orphan numeric types in leaf crates; centralize in `nexus-types`.

### 1.3 Strongly-Typed Entity IDs

All entity references are wrapped in distinct types (`PlayerId`, `SessionId`, `ItemId`, `EnemyId`), preventing ID-swap bugs.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnemyId(u64);

// Type mismatch: cannot pass EnemyId where PlayerId is expected
let player_id: PlayerId = PlayerId(1);
let enemy_id: EnemyId = EnemyId(100);
// let session = lookup_player(enemy_id); // ← Compile error
```

**Roadmap**: As IB4 (Infinity Blade 4) and other projects grow, the ID taxonomy may expand. Consider a proc-macro `#[derive(EntityId)]` to generate boilerplate.

---

## 2. Multi-Workspace Orchestration

### 2.1 Resolver Strategy

The monorepo uses **two resolver generations** across six independent workspaces:

| Workspace            | Resolver | Rust Edition | Dep Strategy                    |
|----------------------|----------|---------------|---------------------------------|
| `tools/`             | 3        | 2024          | `[workspace.dependencies]` + inheritance |
| `nexus-engine/`      | 2        | 2021          | `[workspace.dependencies]` + inheritance |
| `blueprint-rs/`      | 2        | 2021          | Direct deps per crate           |
| `unify-rs/`          | 2        | 2021          | `[workspace.dependencies]` + patch.crates-io |
| `chicago-tdd-tools/` | —        | 2021          | Single crate                    |
| `infinity-blade-4/mud/` | 2     | 2021          | `[workspace.dependencies]` + inheritance |
| `asset-pipeline/`    | 2        | 2021          | `[workspace.dependencies]` + inheritance |
| `rocket-simulator/`  | 2        | 2021          | Single/minimal members          |

**Critical Rule**: Do not mix resolver versions in a single workspace. Resolver 3 (Rocket 2024 edition) only appears in `tools/` because it requires careful coordination of edition/dep unification across all members.

**Open Question**: Should the monorepo migrate fully to resolver 3 by 2028? Resolver 3 improves feature unification but risks breaking downstream consumers. For now, maintain resolver 2 as the standard and treat resolver 3 as an opt-in advanced pattern in `tools/`.

### 2.2 Dependency Graph

All workspaces share a common root in `nexus-types` (zero external deps):

```
nexus-types (no deps)
    ↓
nexus-{combat, session, economy, net, ecs, gfx, shop}
    ↓
nexus-integration (integration tests)

unify-core (no deps)
    ↓
unify-{config, rdf, lsp, mcp, ffi, test, bp, ...}
    ↓
unify (CLI binary)

rocket-sdk (no direct game deps)
    ↓
rocket-cmd (CLI binary)
    ↓
knhk (WASM law enforcer, uses rocket-sdk)

blueprint-core (no game deps)
    ↓
blueprint-{cli, macros, testing}

asset-pipeline (no game deps)

ib4-core (game domain types)
    ↓
ib4-{combat, progression, ai}
    ↓
ib4-mud (MUD server, consumes ib4-*)
    ↓
ib4-integration-tests
```

**Constraint**: No circular dependencies allowed. Use `cargo deny --all` in CI to detect cycles.

**Scaling**: As new game projects are added (e.g., GunnamNexusClient in Rust), follow the pattern: define a `*-types` crate with zero external deps, then build domain logic crates on top of it.

### 2.3 Workspace Boundaries

Each workspace is an **independent build and test unit**. They coordinate via:

1. **Published crates** (`crates.io` or local path deps)
2. **Semantic versioning** (breaking changes require version bump)
3. **Cargo feature flags** (for optional interop)

**Path Dependencies (Breaking CI)**

Two unresolved path deps remain a blocker for CI on unfamiliar machines:

- `tools/rocket-cmd/Cargo.toml` → `path = "/Users/sac/clap-noun-verb"`
- `tools/knhk/Cargo.toml` → `path = "/Users/sac/wasm4pm-compat"`

**2026-Q2 Milestone**: Vendor these crates or publish to crates.io. For now, maintain a machine-specific `Makefile.local` that symlinks or patches these paths.

---

## 3. Game Engine Integration

### 3.1 UE4.27 Target and HTML5 Constraints

Rocket Craft targets **Unreal Engine 4.27** (not 5.x) with mandatory **HTML5/WASM support** for browser delivery.

**Critical HTML5 Limitations**:

- **No Apex Destruction**: Procedural mesh destruction, soft-body physics, and cloth simulation do not work in HTML5.
- **No Procedural Meshes**: `ProceduralMeshComponent` is not available in HTML5 builds.
- **WebSocket Only**: Browser restrictions prevent raw UDP/TCP. All multiplayer uses `WebSocketNetworking` plugin on port **8889**.
- **Memory Budget**: HTML5 builds are constrained by browser memory limits (~500 MB on mobile, ~2 GB on desktop).
- **ES3 Shader Subset**: Shader complexity must stay within ES3 (no compute shaders, limited texture array support).

**Consequence**: Game design must account for these limits up-front. Any feature requiring destruction or procedural geometry requires a native (Win64/Mobile) code path.

### 3.2 Gundam Nexus — Formal Rust Model

**nexus-engine** (10-crate workspace) is the **authoritative formal model** of Gundam Nexus game logic. UE4 C++ implementation mirrors this Rust code.

**Architecture**:

```
nexus-types        ← Zero-dep root; units, IDs, typestates, errors
    ↓
nexus-{combat, session, economy, net, ecs, gfx, shop}  ← Domain layers
    ↓
nexus-integration  ← Game loop orchestration + 22 E2E tests
```

**Key Subsystems**:

| Crate           | Responsibility                                                 |
|-----------------|----------------------------------------------------------------|
| `nexus-combat`  | Typestate combat FSM, combo chains (up to 4 hits), parry/dodge resolution, damage calc |
| `nexus-session` | Player sessions, inventory (`Inventory<const N>`), NPC dialogue FSM |
| `nexus-economy` | Double-entry ledger (debit == credit), marketplace, auction typestates |
| `nexus-net`     | WebSocket protocol, matchmaking queue, game room state         |
| `nexus-ecs`     | `hecs::World` wrapper, 20 component types, 5 core systems      |
| `nexus-gfx`     | 3D math (`nalgebra`), camera, frustum culling, GPU vertex types |
| `nexus-shop`    | ChaCha8 gacha RNG with pity counter, battle pass season, AR barcode registry |

**Testing Strategy**: 

- **Unit tests** in each crate using `#[test]`
- **Property-based tests** via `proptest` with centralized strategies in `nexus-tests/src/strategies.rs`
- **Integration tests** in `nexus-integration/tests/` that drive the full game loop
- **Regression file**: `crates/nexus-integration/tests/integration_tests.proptest-regressions` — **must be committed**; it pins discovered edge cases

**Open Question**: Should `nexus-engine` be extracted as a standalone published crate (separate repo or `nexus-engine-pub/`)? For now, it remains internal. If third parties want to embed the Nexus logic, maintain a stable API contract via doc examples.

### 3.3 Infinity Blade 4 — AAA Mobile Action RPG

**infinity-blade-4/** is both a full UE4 project and a companion **6-crate Rust MUD backend**.

**UE4 Project Structure**:
- **Platforms**: iOS, Android, Win64
- **Key systems**: combat, progression (bloodlines + perks), equipment, AI (Titan + GodKing phases), HUD
- **Assets**: data-driven via CSV (`weapons.csv`, `armor.csv`, `enemies.csv`, `bloodline_perks.csv`, `gems.csv`) imported as UE4 DataTables

**MUD Companion**:
- Standalone TCP server for text-based multiplayer sessions
- Shares domain types via `ib4-core` crate
- Full combat loop, progression, and GodKing scripted encounters testable without UE4 Editor
- **6-crate structure**: `ib4-{core, combat, progression, ai, mud, integration-tests}`

**Open Questions**:
1. Should MUD be integrated into a unified in-game chat/session system, or remain a teaching tool?
2. Should IB4 C++ combat code be auto-generated from `ib4-combat` Rust via code-gen pipeline?

### 3.4 Asset Pipeline (Autonomous Conversion)

**asset-pipeline/** provides **autonomous 3D-to-UE4 conversion** via headless Blender.

**Pipeline**:
```
Discovered (walkdir scan, MIME check)
    ↓ validate()
Validated (extension + magic-byte check)
    ↓ hash()
Hashed (blake3 content address)
    ↓ convert() [spawns blender --background --python]
Converted (FBX output)
    ↓ stage()
Staged (copy to Content/Assets/, write manifest.json)
```

**Supported Input**: `.obj`, `.fbx`, `.stl`, `.dae`, `.gltf`, `.glb`

**Constraint**: Blender version sensitivity. Conversion scripts target Blender 3.x. Blender 4.x changed `bpy.ops.export_scene.fbx` signatures — update scripts if upgrading.

**2027 Roadmap**: Extend pipeline to support:
- Material slot mapping (artist-friendly metadata → UE4 material slots)
- LOD auto-generation (using Blender's built-in decimation)
- Collision mesh extraction (separate convex/complex collision from visual geometry)
- FBX texture embedding (embed referenced textures in the FBX to avoid dangling refs)

---

## 4. Semantic Web Layer — unify-rs

### 4.1 RDF Triple Store & SPARQL Engine

**unify-rs/unify-rdf** provides an in-memory **RDF triple store** with **SPARQL query support** and **SHACL shape validation**.

**Core Types**:
```rust
pub struct Triple {
    pub subject: IRI,        // Resource IRI (e.g., http://rocket-craft/game/nexus)
    pub predicate: IRI,      // Property IRI
    pub object: RDFTerm,     // Value (literal, IRI, or blank node)
}

pub struct Store { /* internal triple index */ }

impl Store {
    pub fn insert(&mut self, triple: Triple) -> Result<(), Error>;
    pub fn query_sparql(&self, query: &str) -> Result<QueryResult, Error>;
}
```

**Use Cases**:
1. **Project Manifest Bridge** (`project_bridge.rs`): Maps `versions/` UE4 projects into RDF triples.
2. **Asset Provenance**: blake3 hashes from asset-pipeline stored as RDF triples for audit trail.
3. **Game State Snapshots**: `nexus-integration` can export game state (entities, inventory, ledger) as RDF for process mining.

**SHACL Validation** (`shacl.rs`):
```rust
// Define a shape requiring every Project to have a name
let shape = ShaclShape::new("project")
    .requires_property("dcterms:title");

// Validate the store
store.validate_shapes(vec![shape])?;
```

**2027 Roadmap**: Integrate with **Knowledge Graphs**. Use the RDF store to:
- Track cross-crate dependencies and build times
- Detect circular dependency chains at runtime
- Export provenance metadata (who generated what Blueprint, when, with which inputs)

### 4.2 MCP Server — Tool Exposure

**unify-rs/unify-mcp** is an **MCP (Model Context Protocol) server** that exposes Rocket Craft tools and resources to AI clients (Claude Desktop, Claude Code, etc.).

**Architecture**:
```
unify-mcp (binary) ← runs as MCP stdio server
    ├── ToolRegistry
    │   ├── rocket/manifest/list    (list UE4 projects)
    │   ├── rocket/manifest/describe (describe a project)
    │   ├── rocket/build (invoke build)
    │   ├── rocket/audit (run compliance)
    │   └── anti-llm-cheat-lsp tools
    ├── ResourceRegistry
    │   ├── project://{name}   (access project metadata)
    │   ├── asset://{hash}     (access asset provenance)
    │   └── game-state://{session-id} (access live game state)
    └── McpServer (JSON-RPC over stdio)
```

**Key Tools** (`rocket_tools.rs`):
- `rocket/manifest/list` — enumerate all UE4 projects
- `rocket/manifest/describe` — return target list, platforms, build flags
- `rocket/build` — trigger a build of a specific project + target + platform
- `rocket/audit` — run `knhk` law enforcement plugins
- `anti-llm-cheat-lsp/scan-file` — scan Rust/C/JS/TS for LLM-generated code patterns

**2027 Milestone**: Standardize the resource schema. Define an OpenAPI spec for all resource and tool endpoints so Claude and other clients can discover capabilities.

### 4.3 Anti-LLM-Cheat Detection

**unify-rs/anti-llm-cheat-lsp** scans source code for patterns indicating LLM-generated content:
- Generic variable names (`x`, `data`, `result`)
- Overly verbose comments
- Suspicious error handling (catch-all `_ => panic!()`)
- Encoder/decoder patterns (base64, hex encoding in unexpected contexts)
- Placeholder code (`TODO`, `FIXME`, `unimplemented!()`)

**Current Scope**: Rust, C, C++, JavaScript, TypeScript

**2026 Expansion**: Add detection for:
- GLSL/HLSL shader anomalies
- UE4 Blueprint bloat (unusually large node graphs)
- Python script patterns (relevant for asset-pipeline Blender scripts)

**Caveat**: This is a heuristic scanner, not a proof system. Use outputs as indicators, not final verdicts. High false-positive rate on verbose/auto-formatted code.

### 4.4 LSP Server — Editor Integration

**unify-rs/unify-lsp** provides a **Language Server Protocol** implementation for editor integration (VS Code, Neovim, etc.).

**Capabilities**:
- **Diagnostics**: Emit errors/warnings for policy violations
- **Completions**: Suggest valid state transitions for typestate machines
- **Hover**: Show type information and state machine phase docs
- **Document Snapshots**: Track versioned document state across edits

**Conformance Compositor** (`compositor.rs`): Manages concurrent edits and snapshots. **Critical**: Do not hold a `Snapshot` across async yield points; it may become stale. Always re-fetch from the compositor.

**2027 Roadmap**: Extend LSP to support:
- **Code Lens** for nexus-engine: show test coverage % on each combat function
- **Semantic Tokens**: color-code different typestate markers
- **Workspace Symbols**: search for all `Machine<L, _>` definitions across the workspace

---

## 5. Manufacturing Pipeline — rocket-cmd

### 5.1 Unified CLI Spine

**tools/rocket-cmd** is the primary developer-facing entry point. It wraps all infrastructure via `rocket-sdk` library calls.

**Subcommands** (clap derive):

| Subcommand   | Delegates to              | Purpose                               |
|--------------|---------------------------|---------------------------------------|
| `setup`      | `rocket-sdk::setup`       | Validate UE4 + SDK environment        |
| `sync`       | asset-pipeline + manifest | Pull latest assets, update manifest   |
| `build`      | RunUAT.sh (UE4)           | Build a UE4 project for a platform    |
| `run`        | UE4 launcher              | Launch editor or packaged game        |
| `audit`      | `knhk` plugins            | Run semantic law compliance checks    |
| `crypto`     | `rocket-sdk::crypto`      | Generate Android keystores            |
| `clean`      | rm -rf Binaries/          | Wipe build artifacts                  |
| `pwa`        | npm build (pwa-staff)     | Bundle TypeScript PWA                 |
| `info`       | project-manifest.json     | Print project summary                 |
| `test`       | cargo test (all)          | Run full test suite                   |
| `doctor`     | environment checks        | Diagnose environment health           |
| `logs`       | tail -f UE4 logs          | Stream UE4 log output                 |
| `wasm`       | knhk loader               | Execute WASM compliance plugins       |

### 5.2 Semantic Law Enforcement via WASM Plugins

**tools/knhk** is a **WASM plugin loader** using `wasmer`. Each law is a `.wasm` binary that exports:

```wasm
(func (export "check") (param $manifest i32 $len i32) (result i32))
```

Laws enforce architectural constraints:
1. **No circular crate dependencies**: Validate the workspace graph.
2. **All UE4 targets have keystores**: Fail audit if signing certs missing.
3. **Asset hashes recorded**: Verify all staged assets have RDF triples.
4. **No orphan test files**: All tests must be in proper test dirs.

**Loading** (`knhk/src/plugin.rs`):
```rust
let module = wasmer::Module::new(&engine, &bytes)?;
let instance = wasmer::Instance::new(&mut store, &module, &imports)?;
let check: wasmer::TypedFunction<(i32, i32), i32> = instance.exports.get_function("check")?;
let result = check.call(&mut store, (manifest_ptr, manifest_len))?;
```

**2027 Roadmap**:
1. Standardize on a plugin interface (versioned manifest format).
2. Add hotloading: `knhk watch ./laws/` to reload `.wasm` files on edit.
3. Formalize the "law language": a DSL that compiles to WASM for easier law authoring.

### 5.3 Build Orchestration

The `rocket build` command follows this flow:

1. **Validate** project exists in manifest.
2. **Resolve** target (editor vs. packaged), platform (Win64, Android, iOS, HTML5).
3. **Prepare** — run pre-build hooks (asset sync, law audit).
4. **Build** — invoke `RunUAT.sh BuildCookRun` with platform-specific flags.
5. **Package** — if `--package` flag, run cook + stage + archive.
6. **Sign** — if `--sign` flag, apply Android/iOS certificates.
7. **Emit Receipt** — write blake3-signed receipt with build fingerprint, log output, and artifact paths.

**Artifact Signing**: Uses blake3 for content-addressable build outputs. Receipts chain together: later receipts include blake3 hash of previous receipt, forming an audit trail.

---

## 6. Testing & Verification

### 6.1 Chicago School TDD

**chicago-tdd-tools** provides a **behavior-driven testing harness** with golden file support and scenario runners.

**Pattern**:
```rust
#[test]
fn given_player_with_100hp_when_takes_50_damage_then_hp_becomes_50() {
    let mut player = Player::new(Hp(100));
    player.take_damage(Damage(50));
    assert_eq!(player.hp, Hp(50));
}
```

**Golden Files**: Test outputs are captured and compared against baseline files:
```rust
let result = my_computation();
assert_golden!("tests/golden/my_test.json", result);
```

On first run, creates `my_test.json`. On subsequent runs, compares output. Helps catch unintended behavioral changes.

**2026-Q4 Milestone**: Expand Chicago TDD to support **scenario chains** — multi-step test sequences modeling user journeys.

### 6.2 Property-Based Testing with proptest

All domain types have **centralized `proptest` strategies** in `nexus-tests/src/strategies.rs`:

```rust
pub fn player_strategy() -> impl Strategy<Value = Player> {
    (0u64..u64::MAX, 0..1000u32).prop_flat_map(|(id, level)| {
        (Just(PlayerId(id)), Just(level))
    })
}

pub fn combat_machine_strategy() -> impl Strategy<Value = CombatMachine<Idle>> { ... }
```

**Invariants** (`nexus-tests/src/invariants.rs`):
```rust
#[test]
fn prop_combat_never_negative_hp(machine in combat_machine_strategy()) {
    // ... run combat loop, assert hp always >= 0
}

#[test]
fn prop_ledger_balance_preserved(txns in vec(transaction_strategy(), 1..100)) {
    // ... apply all txns, assert debit == credit
}
```

**Regression File**: `crates/nexus-integration/tests/integration_tests.proptest-regressions` — **must be committed**. It records failing cases found during testing.

**2027 Roadmap**:
1. Increase `PROPTEST_CASES` default from 256 to 1024 in CI (requires faster hardware or parallel test runners).
2. Add **shrinking replay** — given a failing input, automatically minimize it to the smallest reproducing case.
3. Integrate with **Miri** (Rust interpreter) to catch undefined behavior in tests.

### 6.3 E2E Verification — TPS/DfLSS Manufacturing Strategy

The **TPS (Toyota Production System) / DfLSS (Design for Lean Six Sigma)** manufacturing doctrine mandates:

> The final authority is **Playwright visual verification of a browser-native Unreal 4 HTML5/WASM world**.

**7-Gate Acceptance Matrix**:

| Gate | Criterion                                          | Failure Routing                  |
|------|----------------------------------------------------|---------------------------------|
| 0    | Rocket-Craft has a declared world contract        | Contract cell                   |
| 1    | Rocket-Craft emits UE4-consumable artifacts       | Artifact generation cell         |
| 2    | SpeculativeCoder UE4.27 HTML5 ES3 produces pkg   | UE4 fork/build cell             |
| 3    | Playwright opens package, detects engine ready    | Browser-load cell               |
| 4    | Screenshot shows non-error WebGL/Unreal scene     | WebGL/runtime cell              |
| 5    | Keyboard input injected                           | Input-binding cell              |
| 6    | After-screenshot differs from before (delta > ε) | Visual-delta cell               |
| 7    | Prompt, contract hash, logs, screenshots recorded | Receipt/audit cell              |

**Crown Path**:
```
Prompt → Contract → UE4 Artifact → HTML5 Package → Browser → Screenshot
  → Input → Visual Delta → Browser Console → Receipt
```

**Receipt Contents**:
- Prompt text
- Contract hash (blake3 of world spec)
- Build log (stdout/stderr)
- Package path
- Before/after screenshots (PNG)
- Console logs (JSON)
- Input trace (keystrokes)
- Visual delta (pixel difference %)
- Pass/fail verdict
- Timestamp + signature

**2026 Roadmap**: Implement Playwright harness in `unify-rs/unify-test`:
```rust
pub async fn verify_world(contract: &WorldContract) -> Result<Receipt> {
    let pkg = build_html5_package(&contract)?;
    let browser = launch_browser(&pkg).await?;
    let before = capture_screenshot(&browser).await?;
    send_input(&browser, "W").await?;  // Move forward
    let after = capture_screenshot(&browser).await?;
    let delta = compute_visual_delta(&before, &after)?;
    if delta > MOTION_THRESHOLD { 
        emit_receipt(Pass, &contract, &before, &after)?
    } else {
        route_repair(delta)?
    }
}
```

### 6.4 Test Coverage Gaps

**Known Testing Voids** (as of 2026-Q2):

1. **nexus-net**: WebSocket protocol parsing tested in isolation, but not under packet loss/latency. Add chaos testing.
2. **blueprint-rs**: JSON round-trip tests exist, but not T3D serialization + UE4 import cycle.
3. **asset-pipeline**: Conversion tested with mock Blender, but not with real Blender subprocess.
4. **pwa-staff**: Unit tests cover auth, but not full Supabase real-time subscription flow.
5. **unify-mcp**: Tool signatures validated, but not integration with Claude Desktop.
6. **infinite-blade-4/mud**: ASCII combat tested, but not full GodKing phase scripting.

**2026-Q3 Plan**: Execute parallel focused test pushes:
- **nexus-net**: Add chaos monkey fuzzer for packet corruption.
- **blueprint-rs**: Build UE4-native test that imports generated `.t3d` files.
- **pwa-staff**: Mock Supabase real-time channels; test subscription lifecycle.

---

## 7. Security Posture

### 7.1 Parser Validation & Hardening

All parsers (project manifest, .ini files, T3D, CSV) are **fuzz-tested** to reject malformed inputs:

```bash
cargo fuzz -p rocket-sdk -- fuzzing_corpus/
```

**Fixed (2025-Q4)**: 6 critical UTF-8 handling bugs in `project-manifest.json` parser and state machine transitions.

**Scope**: 
- `rocket-sdk/src/manifest.rs` — JSON parser (serde with strict validation)
- `blueprint-rs/blueprint-core/src/parser.rs` — T3D tokenizer
- `asset-pipeline/pipeline-core/src/validate.rs` — MIME type checker

### 7.2 Anti-Cheat Scanning

**unify-rs/anti-llm-cheat-lsp** detects LLM-generated source code patterns:
- Suspicious comments
- Placeholder identifiers
- Blanket error handling
- Encoder/decoder anomalies

**Current Coverage**: Rust, C, C++, JavaScript, TypeScript

**Limitations**: 
- High false-positive rate on auto-formatted code.
- Cannot detect intentional obfuscation by careful prompt engineering.
- Use as a signal (for code review), not a hard gate.

**2027 Enhancement**: Add **semantic hashing** — compute blake3 hash of the AST (not source text), enabling detection of structurally identical code regardless of formatting.

### 7.3 Audit Gates

**Three audit gates** must pass before deployment:

1. **Law Audit** (`rocket audit`): All WASM plugins pass.
2. **Dependency Audit** (`cargo audit`): No known vulnerabilities in `Cargo.lock`.
3. **Receipt Audit** (`unify-rdf` query): All artifacts have provenance triples.

**Gate Bypass**: Allowed only by explicit exemption in `audit-exemptions.toml`, signed with a PGP key. Exemptions are logged and reviewable.

### 7.4 WASM Plugin Stability

**Risk**: A malicious or buggy WASM plugin can crash the entire `rocket audit` command.

**Mitigations**:
- Run plugins in **isolated wasmer `Store`** (memory sandbox).
- Set execution **time limits** (e.g., 5-second timeout per plugin).
- Validate plugin **interface contract** (expected exports, argument types).

**2027 Roadmap**: Implement **plugin signature verification** — sign plugins with a private key, verify signature before loading.

---

## 8. Performance Targets

### 8.1 Frame Time Budget (HTML5)

**Target**: 60 FPS on mid-range mobile (e.g., iPhone 12, Snapdragon 855).

**Budget Breakdown**:
- **Game logic** (nexus-engine): 5 ms
- **Physics** (UE4 Chaos): 3 ms
- **Rendering** (WebGL): 8 ms
- **Network** (WebSocket receive/tick): 1 ms
- **Margin**: 3 ms

**Total**: 20 ms / frame

**Constraints**:
- **No Apex Destruction**: Would exceed physics budget.
- **Triangle Budget**: 100k triangles per frame (culled).
- **Texture Memory**: 256 MB on mobile (with streaming).

### 8.2 Memory Budget

| Platform   | Budget    | Allocation Strategy                         |
|------------|-----------|---------------------------------------------|
| Mobile    | 500 MB    | Aggressive pooling, no allocations in hot loop |
| Desktop   | 2 GB      | Standard allocator, streaming LODs           |
| HTML5     | Browser   | Shared (canvas + game), limit to 256 MB       |

**Measurement**: Add `tracing` spans around hot paths (combat loop, ECS system dispatch). Use `cargo flamegraph` to identify allocation hotspots.

### 8.3 Asset Streaming

**asset-pipeline** stages models with **automatic LOD (Level of Detail)** generation planned for 2027:
- LOD0: Full detail, 100% triangles
- LOD1: 50% triangles (decimated via Blender)
- LOD2: 25% triangles
- LOD3: 10% triangles (bounding box)

**Streaming Protocol**:
1. Request asset by blake3 hash.
2. Serve LOD3 immediately (fast).
3. In background, stream LOD1, LOD2, LOD0 as bandwidth allows.
4. UE4 swaps LOD based on camera distance.

### 8.4 Network Protocols

**WebSocket Framing** (`nexus-net`): 
- Frame size: max 64 KB (UE4 replication packet limit).
- Compression: optional zstd (UE4 native).
- Latency target: < 100 ms P99 (LAN), < 250 ms P99 (WAN).

**Matchmaking**: 
- Queue latency: < 10 seconds median (optimistic)
- Room creation: < 2 seconds after match found

**Measurement**: Add latency histograms to nexus-net connection logging. Export via OpenTelemetry.

---

## 9. Critical Path Items — Equilibrium Milestones 2026-2027

### Q2 2026
- [ ] **Resolve path deps**: Vendor `clap-noun-verb` and `wasm4pm-compat` or publish to crates.io
- [ ] **CI coverage**: Extend `.github/workflows/ci.yml` to run `nexus-engine`, `blueprint-rs`, and `unify-rs` tests
- [ ] **Asset pipeline launch**: Build and test autonomous conversion pipeline with Blender integration
- [ ] **MCP server v1**: Expose `rocket/manifest` and `rocket/build` tools to Claude Desktop

### Q3 2026
- [ ] **Playwright harness v1**: Implement 7-gate E2E verification for one world contract
- [ ] **Nexus combat v1**: Full combat loop (attack, parry, combo, damage) passing E2E
- [ ] **Blueprint round-trip**: Generate `.t3d` files, import into UE4, verify in-editor
- [ ] **PWA offline mode**: Service worker caching, Supabase sync on reconnect

### Q4 2026
- [ ] **IB4 mobile packaging**: Android + iOS builds with keystore signing working end-to-end
- [ ] **Chicago TDD scenario runner**: Multi-step test chains (setup → action → assert → teardown)
- [ ] **Anti-LLM-cheat expansion**: Add C/C++/Python detection
- [ ] **Documentation**: Write architectural decision records (ADRs) for all critical patterns

### Q1 2027
- [ ] **Nexus duel matchmaking v1**: Queue, room creation, initial player state sync
- [ ] **Asset streaming v1**: LOD generation, background streaming protocol
- [ ] **Semantic versioning policy**: Define breaking-change decision tree for workspace upgrades
- [ ] **GunnamNexusClient Rust**: Start client-side prediction + authoritative server model

### Q2 2027
- [ ] **Unified test CI**: All 6 workspaces tested in CI; 80%+ code coverage target
- [ ] **Performance baseline**: Flamegraph profiles for combat loop, ECS dispatch, rendering
- [ ] **Plugin hotloading**: `knhk watch ./laws/` auto-reloads `.wasm` plugins on edit
- [ ] **RDF triple store query optimization**: Index by predicate; benchmark SPARQL latency

### Q3 2027
- [ ] **Multi-player E2E**: Two HTML5 clients, one server, full duel flow verified
- [ ] **Blueprint DSL v1**: High-level syntax for defining Blueprint node graphs (compiles to T3D)
- [ ] **IB4 GodKing AI**: Scripted 5-phase encounter, tested in MUD + UE4 C++
- [ ] **Supabase edge functions**: Deploy first 3 functions (score submission, matchmaking, chat)

### Q4 2027
- [ ] **Production hardening**: Security audit, performance tuning, documentation finalized
- [ ] **Launch Initial Equilibrium**: Gundam Nexus public beta (HTML5 browser, one map, 2v2 duels)
- [ ] **Monitoring & observability**: OpenTelemetry metrics exported to production dashboard

---

## 10. Tech Debt & Risks

### 10.1 Path Dependencies (Blocker)

**Issue**: Two external path deps break CI on unfamiliar machines:
```toml
# tools/rocket-cmd/Cargo.toml
clap-noun-verb = { path = "/Users/sac/clap-noun-verb" }

# tools/knhk/Cargo.toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat" }
```

**Impact**: CI fails on GitHub runners, developer machines, Docker containers.

**Solutions**:
1. **Publish to crates.io** (preferred): `crates.io/crates/clap-noun-verb`, `crates.io/crates/wasm4pm-compat`
2. **Vendor inline**: Copy source into `tools/vendor/` and use path deps.
3. **Conditional Cargo features**: Allow builds without these deps for CI (feature = "noun-verb" defaults to true).

**Target**: Resolve by 2026-Q2 or block all CI.

### 10.2 HTML5 Limitations

**Apex Destruction** and **Procedural Meshes** do not work in HTML5. Game design consequences:
- No destructible buildings/terrain.
- No generated geometry at runtime.
- No soft-body cloth/hair.

**Workaround**: Implement native (Win64) code paths for these features, render fallback geometry in HTML5.

**Open Question**: Should we officially **drop HTML5 support** and target WebGPU instead? WebGPU removes many ES3 constraints but requires browser adoption (only ~60% of users as of 2026).

### 10.3 WASM Plugin Fragility

**Risk**: A buggy WASM plugin can crash the entire `rocket audit` workflow.

**Current Mitigations**:
- Isolated wasmer `Store` (memory sandbox)
- 5-second execution timeout
- Interface contract validation

**Missing**: Signature verification, rollback strategy, health checks.

**2027 Plan**: 
- Add plugin version pinning: `knhk/plugins/laws.manifest.json` declares versioned plugin hashes.
- Implement canary deployment: run new plugin version on a subset of machines first.

### 10.4 Resolver Version Fragmentation

Two resolver versions in use (`tools/` uses resolver 3, others use resolver 2) creates coordination overhead.

**Risk**: Merging workspaces later becomes difficult; feature unification rules differ.

**Mitigation**: 
- Document resolver strategy clearly (this roadmap does).
- Plan for resolver 3 migration by 2028-Q1; make a deliberate choice (adopt or abandon).
- Use `[patch.crates-io]` and `[workspace.dependencies]` consistently.

### 10.5 Circular Dependency Detection

**Current**: No CI gate prevents circular dependencies in the workspace graph.

**Solution**: Add to CI:
```bash
cargo deny --all check bans
```

Fails if any crate has a cycle. Run in GitHub Actions on every push.

### 10.6 Test Flakiness in CI

**Known Issue**: Nexus property-based tests occasionally timeout on resource-constrained CI runners.

**Mitigation**:
- Lower default `PROPTEST_CASES` in CI (256 vs. 1024 locally).
- Increase test timeout to 120 seconds per test binary.
- Use `proptest-regressions` file to replay known failures deterministically.

### 10.7 Asset Pipeline Subprocess Risk

**asset-pipeline** spawns `blender --background` as a subprocess. If Blender crashes or hangs, the pipeline stalls.

**2026 Hardening**:
- Implement 30-second subprocess timeout.
- Capture stderr and log any Blender errors.
- Add fallback: if conversion fails, emit warning and continue (skip asset).
- Consider replacing subprocess with embedded Blender API (bpy via ctypes) for robustness.

### 10.8 Blueprint T3D Serialization Drift

**Risk**: `blueprint-rs` and UE4 Editor may diverge on T3D format interpretation, causing round-trip failures.

**Mitigation**: 
- Test generated `.t3d` files by importing into UE4 and examining the resulting Blueprint in-editor.
- Maintain a "canonical" reference Blueprint for each generated type (created in-editor, never auto-generated).
- Use `git diff` on `.t3d` files to catch serialization changes.

**2027 Plan**: Build UE4-native unit test that imports a generated `.t3d`, inspects node graph, and asserts structure matches the original Rust `BlueprintBuilder` intent.

### 10.9 MCP Client Compatibility

**Risk**: MCP server (unify-mcp) may not be compatible with all clients (Claude Desktop, LM Studio, etc.).

**Mitigation**:
- Test against official MCP reference implementation (`stdinout_mcp`).
- Validate tool and resource signatures against MCP spec v1.0.
- Maintain compatibility matrix in docs.

### 10.10 Rust Edition 2024 Adoption

**tools/** uses Rust edition 2024 (resolver 3). Other workspaces use edition 2021.

**Risk**: Edition 2024 feature stabilization may lag; dependencies may not support it yet.

**Mitigation**: 
- Pin `tools/` to known-compatible set of dependencies (vendored or crates.io with pinned versions).
- Consider reverting to edition 2021 if adoption proves risky.

**Decision Point**: 2026-Q3 — evaluate whether to standardize on edition 2024 across all workspaces or revert to 2021.

---

## 11. Strategic Decisions & Rationale

### 11.1 Phantom-Typed Units Over Newtype

**Decision**: All game quantities are phantom-typed (`Hp(i32)`, `Damage(i32)`), not newtypes.

**Rationale**: 
- Zero runtime cost (phantom types are erased).
- Prevents dimensional errors at compile time (e.g., adding Hp + Armor fails).
- Scales to large type hierarchies without bloat.

**Trade-off**: Less fine-grained control over arithmetic operations. For example, `Damage / Armor` is not well-defined; must explicitly convert via `damage.to_hp_reduction(armor)`.

### 11.2 Typestate Over Runtime Guards

**Decision**: State machines use zero-sized phantom types, not runtime enums.

**Rationale**:
- Illegal transitions caught at compile time, not runtime.
- Zero overhead; phantom types are erased.
- Forced sequential state transitions (cannot jump directly from `Input` to `Admitted`).

**Trade-off**: More boilerplate; each state transition requires a new impl block. Mitigated by macro code-gen (planned 2027).

### 11.3 Resolver 3 in tools/ Only

**Decision**: Only `tools/` uses resolver 3; other workspaces use resolver 2.

**Rationale**:
- Resolver 3 requires careful coordination of feature unification.
- `tools/` is self-contained (no internal game deps), so safer to experiment.
- Other workspaces (nexus-engine, etc.) are more mature; less risk from changing resolver.

**Trade-off**: Coordination overhead. If `tools/` feature changes ripple to other workspaces, must audit carefully. Mitigated by strict path dependency isolation.

### 11.4 In-Memory RDF Store (Not a Full Graph DB)

**Decision**: `unify-rdf` is an in-memory RDF store, not a full graph database (e.g., not Neo4j).

**Rationale**:
- Embedded in Rust (no separate process to manage).
- Fast for build-time queries (project manifest, asset provenance).
- Lower operational overhead.

**Trade-off**: Does not scale to massive graphs (> 1M triples). If provenance tracking becomes critical (e.g., full game state snapshots), may need external graph DB. Decision point: 2027-Q4.

### 11.5 Playwright for E2E Verification (Not Unreal Engine PIE)

**Decision**: E2E tests use Playwright browser automation, not UE4 Play-in-Editor.

**Rationale**:
- Tests the actual HTML5/WASM build (the delivered artifact), not an editor-only path.
- Cross-platform (Windows, macOS, Linux, mobile).
- Aligns with manufacturing doctrine: visual delta proves the world drives.

**Trade-off**: Slower iteration (build HTML5 package each test run). Mitigated by caching, parallel builds, and hot-reload.

### 11.6 Chicago TDD Over BDD Frameworks

**Decision**: Use Chicago-school TDD (context-expectation pairs) instead of Cucumber/Gherkin.

**Rationale**:
- Lower ceremony; tests are Rust code, not external feature files.
- Easier to refactor alongside implementation.
- Integrates naturally with proptest and golden files.

**Trade-off**: Less readable to non-technical stakeholders. Mitigated by comment-heavy tests and scenario documentation.

---

## 12. Roadmap Summary — 2026 to 2030

### 2026: Foundation & Equilibrium Hardening
- **Q1-Q2**: Resolve path deps, launch asset pipeline, extend CI coverage.
- **Q3-Q4**: Playwright E2E v1, Nexus combat v1, Blueprint round-trip, PWA offline.
- **Target**: Prototype multiplayer duel on HTML5 browser.

### 2027: Scaling & Optimization
- **Q1-Q2**: Matchmaking, asset streaming LODs, unified test CI, performance baseline.
- **Q3-Q4**: Multi-player E2E, Blueprint DSL, IB4 GodKing, Supabase edge functions.
- **Target**: Public Equilibrium launch (Gundam Nexus, 2v2 duels, one map).

### 2028: Consolidation & Polish
- **Q1-Q2**: Production hardening, security audit, documentation finalization, monitoring.
- **Q3-Q4**: Feature parity with design doc, platforming (mobile release pipelines), community feedback loop.
- **Target**: 1.0 release, 10k+ concurrent players support.

### 2029-2030: Expansion & Ecosystem
- **Scaling**: Multi-region deployment, geo-replication, edge computing (Deno Edge Functions).
- **Features**: Avatar customization, seasonal battle passes, esports ranking system.
- **Ecosystem**: Modding SDK, third-party Blueprint tools, open-source publishing.
- **Target**: Establish Rocket Craft as a viable indie game engine / platform.

---

## 13. Open Research Questions

1. **Should nexus-engine be extracted as a standalone published crate?** Currently internal; if third parties want the formal model, creating `nexus-engine-pub/` on crates.io makes sense.

2. **Can we replace WASM plugins with a simpler DSL?** The current WASM plugin system is powerful but has operational overhead. A higher-level law language (compiling to WASM) could improve ergonomics.

3. **Should HTML5 support be deprecated in favor of WebGPU?** HTML5 has hard limits (no destruction, no procedural meshes). WebGPU removes these but requires browser adoption. Decision point: 2027-Q1.

4. **What is the long-term storage strategy for massive OCEL logs?** Append-only OCEL recording enables process mining, but a 6-month duel history is terabytes. Archive to cold storage? Implement sampling?

5. **Can we auto-generate C++ UE4 code from Rust nexus-engine?** Currently Rust and C++ are maintained in parallel. Code generation could improve sync but adds complexity.

6. **How should multiplayer cheating be detected?** Current anti-cheat is heuristic (code scanning). Needs forensics (replay validation, anomaly detection in game metrics).

7. **Should the MCP server support real-time subscriptions?** Current MCP is request-response; real-time build logs or game state streams require async push.

---

## Appendix: Glossary

| Term                | Definition                                                       |
|---------------------|------------------------------------------------------------------|
| **typestate**       | A zero-sized phantom type representing a state in a state machine |
| **phantom type**    | A `PhantomData<T>` that has no runtime representation             |
| **resolver 2/3**    | Cargo feature unification algorithm; resolver 3 is stricter       |
| **Machine<L, P>`**  | Generic struct with a Law trait `L` and Phase type `P`           |
| **blueprint**       | A UE4 visual script (node graph), generated by blueprint-rs      |
| **T3D**             | Unreal's text asset format (human-readable UE4 object serialization) |
| **MCP**             | Model Context Protocol; standardized tool/resource interface for AI |
| **RDF**             | Resource Description Framework; semantic triples (subject, predicate, object) |
| **SPARQL**          | Query language for RDF stores                                     |
| **SHACL**           | Shape Constraint Language; validates RDF store against shapes    |
| **WASM**            | WebAssembly; binary format for portable code (used for law plugins) |
| **HTML5 build**     | UE4 project compiled to WebGL + JavaScript (browser executable)   |
| **receipt**         | Immutable audit record of a build, signed with blake3 chain      |
| **E2E**             | End-to-end test; exercises full system from UI to backend        |
| **golden file**     | Baseline output used in regression tests (diff mode)              |
| **proptest**        | Property-based testing framework (generates random inputs)       |
| **OCEL**            | Object-Centric Event Log; trace format for process mining         |
| **Chicago TDD**     | Test-driven development style emphasizing context-expectation pairs |
| **DfLSS**           | Design for Lean Six Sigma; manufacturing quality methodology    |
| **PIE**             | Play-In-Editor; UE4 mode for testing gameplay in the editor      |

---

## Document Metadata

- **Created**: 2026-06-18
- **Last Updated**: 2026-06-18
- **Scope**: Internal architecture documentation for engineers and architects
- **Audience**: Rocket Craft core team, contributors, technical decision-makers
- **Distribution**: Checked into `/home/user/rocket-craft/TECHNICAL_ROADMAP.md`

---

**End of Document**

All critical decisions documented above reflect the current state as of Q2 2026. This roadmap should be revisited quarterly and updated with resolved risks, achieved milestones, and newly discovered constraints.
