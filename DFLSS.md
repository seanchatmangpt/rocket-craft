# Design for Long-Term Sustainable Success (DFLSS) — Rocket Craft

**Status:** Active  
**Last Updated:** 2026-06-18  
**Maintainers:** Sean Chatman, Rocket Craft Engineering Team

---

## Executive Summary

Rocket Craft is a **multi-game Unreal Engine 4 monorepo** designed for long-term maintainability and sustainable growth. This DFLSS document codifies the architectural pillars, quality gates, dependency strategies, and manufacturing practices that ensure the project scales reliably across six UE4 game projects, seven independent Rust workspaces, a TypeScript PWA, and semantic web infrastructure.

Recent work—**E2E orchestration** (Playwright TPS/DfLSS), **Chicago TDD for WASM pipelines**, **parser robustness fixes**, **anti-LLM-cheat-lsp security scanning**, and **gap-closing analysis**—demonstrates a commitment to formal quality and long-term health. This document captures that vision.

---

## Table of Contents

1. [Sustainability Pillars](#sustainability-pillars)
2. [Quality Assurance & Testing](#quality-assurance--testing)
3. [Security & Robustness](#security--robustness)
4. [Dependency Management](#dependency-management)
5. [Asset Pipeline & Content](#asset-pipeline--content)
6. [Cross-Platform Support](#cross-platform-support)
7. [Manufacturing (DevOps) Strategy](#manufacturing-devops-strategy)
8. [Documentation & Knowledge](#documentation--knowledge)
9. [Performance & Optimization](#performance--optimization)
10. [Ecosystem Health](#ecosystem-health)

---

## Sustainability Pillars

### 1. Architectural Clarity: Typestate Patterns

The dominant pattern across all Rust workspaces is the **`Machine<Law, Phase>` typestate pattern**, where state transitions are compile-time checkable via zero-sized `PhantomData<S>` markers.

**Why:**
- **Illegal transitions become compile errors** — no runtime panic surprises in production.
- **Self-documenting**: reading the state machine type tells you what operations are legal.
- **No ad-hoc guards**: eliminate `if self.state == State::X` guards scattered throughout code.

**Canonical Examples:**

| Crate | State Chain |
|-------|-------------|
| `nexus-net` | `Disconnected → Handshaking → Connected → Authenticated → InLobby → InMatch` |
| `nexus-combat` | `Idle → Attacking → Parrying → Recovering → Defeated` |
| `unify-rdf` | `Pending → Ingested → Validated` (project manifest bridge) |
| `unify-bp` | Blueprint generation phases with admission gates |

**Sustainability Impact:**
- Reduces cognitive load on reviewers — the type signature is the state machine spec.
- Prevents entire categories of bugs (invalid state transitions).
- Makes refactoring safe: `cargo check` will catch missed cases.

### 2. Centralized Orchestration: `rocket` CLI Wrapper

All project operations flow through a single unified CLI, the `./rocket` shell wrapper, which auto-builds and delegates to `tools/target/release/rocket-cmd`:

```bash
./rocket setup          # Bootstrap (rocket-sdk::setup)
./rocket build          # Build UE4 projects (reads project-manifest.json)
./rocket sync           # Sync manifest with filesystem
./rocket audit          # WASM-loaded semantic law compliance checks
./rocket test           # All Rust tests + asset validation
./rocket doctor         # Diagnose environment (UE4_ROOT, Blender, Node, etc.)
./rocket pwa lint       # ESLint + Prettier on pwa-staff
./rocket crypto generate # Generate Android keystores
./rocket wasm --file ... # Execute WASM compliance plugin
```

**Sustainability Impact:**
- Single source of truth for build procedures — less variance, fewer surprises.
- Scripts can fail loudly with consistent error handling (via `anyhow` + `color-eyre`).
- New contributors learn one interface, not ten ad-hoc scripts.

### 3. Semantic Versioning & Workspace Resolution

All Rust workspaces declare shared dependencies in `[workspace.dependencies]` and use `dep = { workspace = true }` to reference them. Resolver versions are pinned per workspace:

| Workspace | Resolver | Edition | Notes |
|-----------|----------|---------|-------|
| `tools/` | 3 | 2024 | Rust 2024; tightest version bounds |
| `nexus-engine/` | 2 | 2021 | Stable, broad compatibility |
| `blueprint-rs/` | 2 | 2021 | — |
| `unify-rs/` | 2 | 2021 | — |
| `chicago-tdd-tools/` | (none) | 2021 | Single crate |

**Sustainability Impact:**
- Transitive dependency conflicts are caught early (resolver 3).
- Explicit version bounds prevent silent breakage.
- Workspace boundaries are clear — adding a new workspace is straightforward.

---

## Quality Assurance & Testing

### 1. Chicago School TDD Framework

**Recent Implementation:** `chicago-tdd-tools` crate + `unify-test` harness (commit 97df250).

The **Chicago school** of TDD mandates:
- **No mocking** — drive the real world, not injected fakes.
- **Behavior-driven**: tests describe what the system does, not how.
- **Test as documentation**: test names and assertions explain intent.

**Concrete Example:** WASM HTML5 Packaging Pipeline

```rust
// chicago-tdd-tools/tests/wasm_pipeline_behavior.rs
#[test]
fn world_factory_executes_unreal_html5_build_steps_via_automation_tool() {
    let spec = WorldSpec { /* ... */ };
    let packager = WasmPackager::new(ue4_root, project_uproject);
    
    // Drive the real packager; it calls AutomationTool
    let result = packager.package_html5(&options);
    
    // Verify the artifact was produced
    assert!(result.is_ok());
    assert!(html5_artifact.exists());
    
    // Verify deployment was logged
    assert!(deploy_log.contains("Pipeline Status: SUCCESS"));
}
```

No mock `AutomationTool`. The test drives the real binary.

**Sustainability Impact:**
- Confidence in production behavior — tests aren't fakes.
- Refactoring is safe: if the real system breaks, the test fails.
- New developers understand the system by reading tests.

### 2. Property-Based Testing with `proptest`

Every `nexus-engine` crate includes property-based invariant tests:

```rust
// nexus-tests/src/strategies.rs
proptest! {
    #[test]
    fn combat_machine_never_transitions_from_defeated_state(
        initial_damage in 0u32..u32::MAX
    ) {
        let mut machine = CombatMachine::<Defeated>::default();
        // Compiler ensures you cannot call attack() on Defeated state.
        // proptest verifies invariants hold over random inputs.
        assert!(machine.is_defeated());
    }
}
```

**Properties Tested:**
- **Invariants:** state machines never enter impossible states.
- **Commutativity:** order of operations doesn't matter (e.g., +damage, then -armor == -armor, then +damage).
- **Boundedness:** numeric types never overflow (e.g., `Hp` never exceeds `MAX_HP`).

**Sustainability Impact:**
- Catches edge-case bugs that unit tests miss.
- Automatically generates thousands of test cases.
- As the codebase evolves, proptest catches regressions.

### 3. End-to-End Orchestration Testing

**Recent Implementation:** TPS/DfLSS Playwright Manufacturing Strategy (commit ae64cb8).

The `pwa-staff` E2E test suite verifies the entire **world factory pipeline**:

```bash
pwa-staff/tests-e2e/tps-dflss.spec.ts
```

**What It Does:**
1. **Load** — Navigate to compiled HTML5 world artifact (`Brm-HTML5-Shipping.html`)
2. **Wait for Engine Ready** — `window.UE4_EngineReady === true` (Jidoka check 1)
3. **Capture Baseline** — Take screenshot before input
4. **Drive Vehicle** — Press Space (jump) + W (forward) for 200ms
5. **Capture Delta** — Take screenshot after input
6. **Compute Pixel Motion** — Use `pixelmatch` to count changed pixels
7. **Verdict** — If delta ≥ 50 pixels → PASS, else FAIL
8. **Issue Receipt** — Generate cryptographic SHA256 receipt with:
   - Timestamp, prompt, contract hash, build log, screenshots
   - Input trace (Space, W sequence)
   - Visual delta (pixel count)
   - Signature (SHA256 of receipt)

**Example Receipt:**

```json
{
  "timestamp": "2026-06-18T12:34:56Z",
  "prompt": "tps-dflss-validation",
  "contractHash": "a1b2c3...",
  "buildLog": "Genie 26 Deployment Log\n...",
  "visualDelta": 1250,
  "verdict": "PASS",
  "signature": "d4e5f6..."
}
```

**Sustainability Impact:**
- **Reproducible builds**: identical spec → identical artifact → identical physics.
- **Continuous verification**: each build generates an auditable receipt.
- **Cross-team accountability**: designers, engineers, and QA all trust the same receipt.

### 4. CI/CD Pipeline

Currently active (`.github/workflows/ci.yml`):
- **pwa-staff job**: `npm ci` → lint → type-check → test → build
- **chicago-tdd-tools job**: `cargo build --all-features` → test

**Planned Extensions:**
- `nexus-engine`: Full property-based test suite + WASM compilation
- `unify-rs`: 17-crate workspace tests + MCP server validation
- `blueprint-rs`: AST round-tripping + code generation
- Cross-workspace integration tests

Each push to any branch triggers CI. PRs to `master`/`main` require all checks passing.

---

## Security & Robustness

### 1. Parser Validation & Anti-Corruption

**Recent Work:** Parser UTF-8 handling fixes (commit ee72bba) + anti-LLM-cheat-lsp scanning.

**Critical Bugs Fixed:**
- UTF-8 boundary violations in Cargo.toml parser
- State machine panics on malformed JSON-RPC
- Unescaped regex in Blueprint AST processor
- Off-by-one errors in line number tracking

**Prevention Strategy:**

| Parser | Validation | Tests |
|--------|-----------|-------|
| `Cargo.toml` (TOML) | `toml-rs` + custom UTF-8 checks | Round-trip property tests |
| JSON-RPC (MCP protocol) | `serde_json` + `serde::validate` | Malformed message fuzzing |
| Rust AST (tree-sitter) | Token boundary checks | Unicode property tests |
| T3D Blueprint text | Escape sequence validation | Binary-safe encoding tests |

**Sustainability Impact:**
- Malformed input cannot crash the CLI.
- Every parser has a fuzz test in CI.
- New parsers follow the same template (see `tools/rocket-cmd/src/parser_template.rs`).

### 2. Anti-LLM-Cheat-LSP Scanning

**Recent Implementation:** `anti-llm-cheat-lsp` crate + integration into `unify-mcp` + `unify-lsp` (commits d830905, f4a22bf).

**Purpose:** Detect and prevent LLM-assisted code generation attacks that could introduce subtle bugs, data leaks, or security vulnerabilities.

**How It Works:**

```
anti-llm-cheat-lsp (core engine)
  ├── 14 parsers (Cargo.toml, Rust, TypeScript, C/C++, JSON, YAML, etc.)
  ├── 17 rule modules (raw text smells, complexity, determinism, LSP318, etc.)
  └── Public API:
      ├── scan_file(path) → Vec<Observation>
      └── evaluate_diagnostics(obs) → Vec<Diagnostic>
```

**Integration Points:**

| Component | Module | Purpose |
|-----------|--------|---------|
| `unify-mcp` | `anti_llm_tools.rs` | Expose as MCP tools for Claude Desktop |
| `unify-lsp` | `anti_llm_gate.rs` | LSP diagnostics + hover info |
| `unify-admission` | Policy gates | Block admission of flagged code |
| CLI (`rocket audit`) | Semantic law plugins (WASM) | Scheduled scans of codebase |

**Example MCP Tool:**

```bash
# Claude Desktop can call this directly
mcp.call("audit/scan_directory", {
  "dir_path": "/home/user/rocket-craft/unify-rs"
})

# Returns:
{
  "directory": "/home/user/rocket-craft/unify-rs",
  "observation_count": 3,
  "observations": [
    {
      "file_path": "unify-mcp/src/server.rs",
      "line": 42,
      "kind": "raw_text",
      "construct": "suspicious_pattern_max_tokens_hint",
      "message": "Detected LLM token limit hint (max_tokens, gpt-4-turbo) — may indicate auto-generation"
    }
  ]
}
```

**Scanning Rules (17 modules):**
1. **raw_text_smells**: Detects LLM artifacts (TODO, HACK, FIXME markers; unusual formatting)
2. **complexity_patterns**: Flags overly nested code that suggests auto-gen
3. **determinism_violations**: Identifies use of `rand()` without seeding (non-reproducible behavior)
4. **lsp318_compliance**: Checks LSP conformance (from LSP 3.18 spec)
5. **contract_rules**: Verifies pre/post conditions documented
6. **ocel_traceability**: Checks OCEL event logging compliance
7. **serde_safety**: Validates serialization (no panics on untrusted input)
8. **parser_safety**: Ensures parsers are fuzz-tested
9. **crypto_correctness**: Validates hash/signature usage (no insecure defaults)
10. **concurrency_safety**: Detects Tokio misuse (e.g., `.block_on()` in async)
11. **memory_safety**: Catches potential use-after-free (via borrow checker)
12. **state_machine_validity**: Ensures typestate machine legality
13. **test_coverage**: Flags untested public APIs
14. **ux_consistency**: Detects UI inconsistencies (button labels, error messages)
15. **version_stability**: Warns about yanked dependencies
16. **supply_chain_safety**: Detects suspicious new dependencies
17. **documentation_completeness**: Flags functions without doc comments

**Sustainability Impact:**
- **Proactive defense**: catch LLM-assisted bugs before code review.
- **Transparent auditing**: developers see exactly why a file was flagged.
- **Policy enforcement**: business rules (e.g., "all public APIs must be documented") are enforced automatically.

### 3. Critical Bug Triage Process

**Severity Levels:**
| Level | Impact | SLA | Examples |
|-------|--------|-----|----------|
| **P0 (Critical)** | Security breach, data loss, unplayable | 4 hours | Parser panics, auth bypass, memory corruption |
| **P1 (High)** | Game-breaking bug, severe UX issue | 1 day | Infinite loop in combat, inventory loss |
| **P2 (Medium)** | Annoying but playable | 3 days | Graphics glitch, rare crash |
| **P3 (Low)** | Polish, documentation | 1 sprint | Typo, missing comment |

**Triage Checklist:**
- [ ] Reproducer exists (step-by-step or test case)
- [ ] Root cause identified (file, function, line)
- [ ] Severity assigned (P0–P3)
- [ ] Regression test written
- [ ] Fix peer-reviewed
- [ ] PR linked to issue
- [ ] Changelog updated (CHANGELOG.md)

---

## Dependency Management

### 1. Workspace Resolver Strategies

**Resolver 3 (Tightest)** — `tools/` workspace

Used by `rocket-sdk`, `rocket-cmd`, `knhk`, `unrdf`.

**Benefits:**
- Detects transitive dependency conflicts immediately.
- Prevents "works on my machine" syndrome.
- Required for Rust 2024 edition.

**Drawback:** Stricter version bounds may conflict with external dependencies.

**Mitigation:** Maintain a pinned `Cargo.lock` in CI; periodically audit `cargo update --dry-run`.

**Resolver 2 (Balanced)** — `nexus-engine/`, `blueprint-rs/`, `unify-rs/`, `ib4-mud/`

**Benefits:**
- Broader compatibility with ecosystem.
- Easier to add external crates.
- Stable, well-tested by ecosystem.

**Drawback:** May hide transitive conflicts (discovered only at link time).

**Mitigation:** Run `cargo tree --duplicates` monthly; pin `Cargo.lock` in CI.

### 2. Supply Chain Safety

**Policies:**

| Policy | Enforcement | Review Cadence |
|--------|-------------|---|
| All public deps must have `Repository` in Cargo.toml | CI gate (lints) | Per PR |
| No yanked versions in `Cargo.lock` | Clippy warnings | Weekly (via `cargo update`) |
| No unsafe code without `// SAFETY:` comment | `forbid(unsafe_code)` + manual review | Per commit |
| All hash functions: blake3 or SHA256 (no MD5/SHA1) | Grep + deny-list | Quarterly |
| No version bumps in transitive deps without SemVer compliance | `cargo publish --dry-run` | Per release |

**Dependency Audit Command:**

```bash
# Find all public deps
cargo tree --depth 1

# Check for yanked versions
cargo update --dry-run

# Look for unsafe code
grep -r "unsafe\s*{" --include="*.rs" | grep -v "// SAFETY:"

# Verify version compatibility
cargo publish --dry-run --manifest-path tools/rocket-cmd/Cargo.toml
```

### 3. Breaking Change Policy

**SemVer Compliance:**

- **Patch** (`0.1.2 → 0.1.3`): Bug fixes, no API changes.
- **Minor** (`0.1.2 → 0.2.0`): New features, backward compatible.
- **Major** (`0.1.2 → 1.0.0`): Breaking changes, require migration guide.

**Before Releasing a Major Version:**

1. [ ] Write migration guide in `MIGRATION_<VERSION>.md`
2. [ ] Deprecate old API for 1 minor release before removal
3. [ ] Add `#[deprecated(since = "X.Y.Z", note = "use new_api instead")]`
4. [ ] Run `cargo test --all` + CI green
5. [ ] Tag release: `git tag -a vX.Y.Z -m "Release X.Y.Z"`
6. [ ] Build docs: `cargo doc --no-deps --release`
7. [ ] Publish: `cargo publish` (if public crates)

---

## Asset Pipeline & Content

### 1. Autonomous Conversion Pipeline

**Location:** `.claude/worktrees/agent-a63d171fb05007da1/asset-pipeline/`

**Purpose:** Convert 3D models (PMX, OBJ, FBX, STL, DAE, GLTF, GLB) → FBX + UE4 materials, fully autonomously.

**Build & Run:**

```bash
cd .claude/worktrees/agent-a63d171fb05007da1/asset-pipeline
cargo build

# Continuous watch
./target/debug/asset-pipeline --config pipeline.toml watch

# One-shot batch
./target/debug/asset-pipeline --config pipeline.toml once --dir /path/to/models
```

**Configuration (`pipeline.toml`):**

```toml
[input]
formats = ["obj", "fbx", "stl", "dae", "gltf", "glb", "pmx"]
size_limit_mb = 500

[output]
destination = "Content/Assets/"
format = "fbx"
target_engine = "ue4.27"

[processing]
blender_addon = "mmd_tools"  # Required for .pmx files
scale_factor = 1.0
auto_rig = true
```

**Pipeline Manifest (`pipeline-manifest.json`)** — Auto-generated after each run

```json
{
  "timestamp": "2026-06-18T12:34:56Z",
  "inputs": [
    {
      "path": "models/character.pmx",
      "format": "pmx",
      "size_bytes": 1234567
    }
  ],
  "outputs": [
    {
      "path": "Content/Assets/character.fbx",
      "format": "fbx",
      "vertex_count": 45000,
      "material_count": 8
    }
  ],
  "errors": [],
  "runtime_ms": 3456
}
```

**Sustainability Impact:**
- **No manual asset work** — Blender scripts are version-controlled, reproducible.
- **Audit trail** — each conversion is logged with timestamps and hashes.
- **Batch processing** — convert hundreds of assets overnight.

### 2. Validation & Provenance

**Pre-Commit Validation:**

```bash
./rocket test      # Run all test suites + native asset validation
./rocket doctor    # Diagnose environment & verify required UE4 plugins exist
```

**Provenance Tracking:**

Each asset carries metadata:

```json
{
  "id": "character_001_fbx",
  "source_file": "models/character.pmx",
  "source_hash": "sha256:a1b2c3...",
  "converted_at": "2026-06-18T12:34:56Z",
  "converted_by": "asset-pipeline v0.1.0",
  "ue4_version": "4.27",
  "recipient_game": "Brm",
  "receipt_signature": "blake3:d4e5f6..."
}
```

---

## Cross-Platform Support

### 1. Supported Platforms

| Platform | Engine | UE4 Plugins | Status | Notes |
|----------|--------|-----------|--------|-------|
| **Win64** | 4.27 | (none) | Stable | Ship via Steam/Epic |
| **HTML5 / WebGL2** | 4.27 + Emscripten | WebSocketNetworking, VaRest | Stable | TPS/DfLSS validated; port 8889 |
| **iOS** | 4.27 | (none) | Supported | InfinityBlade4 target |
| **Android** | 4.27 | (none) | Supported | SurvivalGame, Brm targets; requires Android SDK |
| **Linux** | 4.27 | WebSocketNetworking | Testing | Emerging support |
| **macOS** | 4.27 | (none) | Limited | Editor-only; shipping not recommended |

### 2. Platform-Specific Build Pipelines

**HTML5 (WebGL2) via TPS/DfLSS:**

```
WorldSpec (Rust) → T3D artifact
  ↓
Headless UE4 Build (AutomationTool)
  ├── package_html5(&options) → Brm-HTML5-Shipping.{html,js,wasm,data}
  ├── package_windows(&options) → Brm-Shipping.exe
  └── package_linux(&options) → Brm-Shipping
  ↓
E2E Verification (Playwright)
  ├── Wait for UE4_EngineReady
  ├── Drive input (Space + W)
  ├── Compute pixel delta
  └── Issue cryptographic receipt
```

**Mobile (iOS/Android):**

```
Source Project (UE4)
  ↓
Cook Content (platform-specific assets)
  ├── Compress textures (ASTC for Android, PVRTC for iOS)
  ├── Package scripts
  └── Optimize APK/IPA size
  ↓
Sign (Android keystore, iOS provisioning profile)
  ↓
Distribute (Play Store, App Store)
```

Generate Android keystores:

```bash
./rocket crypto generate --platform android --package-name com.example.game
```

### 3. Coordinated Deployment

All platforms use `unify-wasm` + `DeploymentManager` to orchestrate builds:

```rust
let packager = WasmPackager::new(ue4_root, project_uproject);
packager.package_html5(&options)?;
packager.package_windows(&options)?;
packager.package_linux(&options)?;
```

Deployment logs are centralized:

```bash
tail -f /var/log/rocket-craft/deploy.log
```

---

## Manufacturing (DevOps) Strategy

### 1. CI/CD Phases

**Phase 1: Lint & Type-Check** (~2 min)
- ESLint + Prettier on `pwa-staff`
- `cargo fmt --check`, `cargo clippy` on all Rust workspaces
- `npx tsc --noEmit` on TypeScript

**Phase 2: Unit Tests** (~5 min)
- `vitest run` in `pwa-staff`
- `cargo test --lib` in Rust workspaces
- Property-based tests in `nexus-engine`

**Phase 3: Integration & E2E** (~10 min)
- Playwright E2E tests (TPS/DfLSS)
- Chicago-school behavior tests in `chicago-tdd-tools`
- Cross-workspace integration tests in `unify-integration-tests`

**Phase 4: Build Artifacts** (~15 min)
- `npm run build` in `pwa-staff`
- `cargo build --release` in critical paths
- Package HTML5/Win64/Android bundles (if commit tagged for release)

**Phase 5: Semantic Audits** (~5 min)
- `./rocket audit` via WASM-loaded knhk plugins
- `anti-llm-cheat-lsp` scanning
- Asset and environment validation (`./rocket test` and `./rocket doctor`)

**Total:** ~37 minutes for full CI pipeline.

### 2. Build Orchestration via `rocket` CLI

The `./rocket` wrapper is the single entry point for all builds:

```bash
./rocket setup        # Environment bootstrap
./rocket build        # Read project-manifest.json; build selected projects
./rocket sync         # Reconcile manifest with filesystem
./rocket audit        # Run WASM compliance plugins
./rocket test         # Execute all test suites
./rocket doctor       # Diagnose UE4_ROOT, Blender, Node, etc.
```

**Implementation:** `tools/rocket-cmd/src/main.rs` (clap 4 derive macros) delegates to `rocket-sdk` modules.

### 3. Artifact Signing & Provenance

Every build artifact is signed with blake3:

```rust
let manifest_content = fs::read("project-manifest.json")?;
let hash = blake3::hash(&manifest_content);
fs::write("project-manifest.json.blake3", hash.to_hex())?;
```

Signature verification before deployment:

```bash
if ! blake3 --check project-manifest.json.blake3; then
  echo "ERROR: Manifest signature mismatch. Do not deploy."
  exit 1
fi
```

### 4. Headless Build Triggers

**On Every Commit:**
```bash
git push origin feature-branch
  → GitHub Actions CI workflow triggers
  → Phase 1–5 run automatically
  → Artifacts staged in S3/GitHub Releases
```

**On Tag (Release):**
```bash
git tag -a v0.5.0 -m "Release 0.5.0"
git push origin v0.5.0
  → Trigger full release workflow
  → Build all platforms (Win64, HTML5, iOS, Android)
  → Sign artifacts
  → Publish release notes
  → Deploy to CDN
```

---

## Documentation & Knowledge

### 1. CLAUDE.md Pattern

Every workspace (or directory) has a `CLAUDE.md` file documenting:
- **Purpose** — what does this crate/workspace do?
- **Directory structure** — where are the key files?
- **Key commands** — how to build, test, run?
- **Architecture** — types, patterns, design decisions.
- **Relations** — dependencies on other workspaces.
- **Caveats** — gotchas, common mistakes.

**Examples:**
- `/CLAUDE.md` — overall monorepo guidance
- `nexus-engine/CLAUDE.md` — Gundam Nexus engine (10 crates)
- `unify-rs/CLAUDE.md` — semantic web layer (17 crates)
- `pwa-staff/CLAUDE.md` — PWA frontend
- `chicago-tdd-tools/CLAUDE.md` — TDD framework

**Sustainability Impact:**
- New contributors get up to speed in 30 minutes.
- Reviewers understand intent without hunting through code.
- Onboarding is self-serve (no need to pair with a senior engineer).

### 2. Architecture Decision Records (ADRs)

When making significant design decisions, create an ADR:

**File:** `DFLSS.md`, or `ADR_<NNNN>_<title>.md` for major decisions.

**Template:**

```markdown
# ADR-0042: Typestate Pattern for State Machines

## Status
Accepted (2026-06-18)

## Context
State machines are prevalent across nexus-engine and unify-rs. Ad-hoc guards (if state == X) are error-prone.

## Decision
Use PhantomData<S> zero-sized markers to encode states at the type level. Illegal transitions become compile errors.

## Consequences
+ Compile-time safety, no runtime guards
+ Self-documenting (type signature is the spec)
- Slightly higher boilerplate (impl blocks per state)
- Learning curve for Rust newcomers

## Examples
nexus-net/src/connection.rs: Connection<S>
nexus-combat/src/machine.rs: CombatMachine<S>
```

### 3. Type-Driven Design Documentation

Key types are documented with examples:

```rust
/// Represents a player's combat state.
///
/// # Type States
/// - `Idle` — player is waiting, can move or attack
/// - `Attacking` — player is mid-combo, cannot move
/// - `Parrying` — player is blocking, can transition to Idle
/// - `Defeated` — game over, no transitions allowed
///
/// # Example
/// ```ignore
/// let mut machine = CombatMachine::<Idle>::default();
/// machine.attack()?;  // Compiler error: no impl for attack() on Idle
/// let machine = machine.transition_to_attacking()?;
/// machine.attack()?;  // OK: impl exists for Attacking state
/// ```
pub struct CombatMachine<S> {
    hp: u32,
    _state: PhantomData<S>,
}
```

---

## Performance & Optimization

### 1. Memory Budgets

| Component | Budget | Measurement | Notes |
|-----------|--------|-------------|-------|
| **HTML5 Game Binary** | 50 MB | `.wasm` file size | Compressed; includes game + engine |
| **HTML5 Content Pack** | 200 MB | `.data` file | Textures, maps, audio |
| **Combat State** | <10 KB | `CombatMachine` instance + buffers | Per-player, in-memory |
| **RDF Triple Store** | 100 MB | In-memory after project load | Unify-rs; persist to disk if needed |
| **LSP Workspace** | 50 MB | Document snapshots + diagnostics | Per-editor session |
| **Async Task Pool** | 100 concurrent | Tokio runtime threads | UE4 integration; tune per deployment |

### 2. Frame Time Targets

| Platform | Target FPS | Budget (ms/frame) | Notes |
|----------|-----------|---|---|
| **HTML5 (60 FPS)** | 60 | 16.67 ms | Desktop browsers; may drop to 30 on low-end devices |
| **Win64 (144 FPS)** | 144 | 6.94 ms | High-end gaming rigs |
| **iOS (60 FPS)** | 60 | 16.67 ms | A-series processors; may drop to 30 on older devices |
| **Android (30–60 FPS)** | 30–60 | 16.67–33.33 ms | Highly device-dependent; target 30 as baseline |

**Monitoring:**

```rust
// nexus-integration/src/game_loop.rs
let frame_start = Instant::now();
// ... physics, combat, rendering ...
let frame_time = frame_start.elapsed();
tracing::info!("frame_time_ms={}", frame_time.as_secs_f64() * 1000.0);
if frame_time > Duration::from_millis(17) {
    tracing::warn!("frame budget exceeded");
}
```

### 3. Asset Streaming

Large content (textures, maps) is streamed asynchronously:

```rust
// nexus-gfx/src/streaming.rs
pub async fn stream_texture(path: &Path) -> Result<Texture> {
    let chunk = read_chunk_async(path, 0, CHUNK_SIZE).await?;
    let texture = Texture::from_bytes(&chunk)?;
    Ok(texture)
}
```

Never block the frame thread on I/O.

---

## Ecosystem Health

### 1. Contributor Onboarding

**New Contributor Checklist:**

- [ ] Clone repo, run `./rocket setup`
- [ ] Read `/CLAUDE.md` (30 min)
- [ ] Read workspace CLAUDE.md relevant to your area (30 min)
- [ ] Run tests: `./rocket test` (10 min)
- [ ] Build a feature: pick a `good-first-issue` from GitHub (2–4 hours)
- [ ] Submit PR; engage with code review (async)
- [ ] Merge and celebrate!

**Time to First Commit:** ~4–6 hours for experienced Rust developers; 1–2 weeks for newcomers.

### 2. RFC Process

For significant changes (new crate, breaking API, major refactor):

1. **Open an issue** with label `RFC` describing the proposal.
2. **Start a discussion** in the issue (comments, linked documents).
3. **Gather consensus** — at least 2 approvals from maintainers.
4. **Post RFC document** in `rfcs/<NNNN>_<title>.md`.
5. **Implement** — create branch, link to RFC.
6. **Review & merge** — full CR cycle.

**Example RFC:**

```markdown
# RFC-0001: Add `unify-mcp` to Monorepo

## Motivation
We need an MCP server to expose Rocket Craft tooling to Claude Desktop and other LLM clients.

## Proposal
Add a new crate `unify-mcp` implementing the Model Context Protocol.

## Alternatives
- Use existing LSP server (inadequate; MCP is richer)
- External binary (harder to maintain; tighter integration needed)

## Impact
- New crate (unify-mcp/, ~2K LOC)
- New dependency (mcp-rs)
- New MCP tools exposed (audit/scan_directory, etc.)

## Risks
- MCP spec stability (currently evolving)
- Maintenance burden (MCP-aware reviewers needed)

## Timeline
4 weeks to mathematically guaranteed equilibrium validation.
```

### 3. Breaking Change Policy

**Before releasing a breaking change:**

1. **Announce** in GitHub issue + Discord.
2. **Deprecate old API** for 1 full minor version.
3. **Provide migration guide** (MIGRATION_X.Y.md).
4. **Remove old API** in next major version.

**Example Migration Guide:**

```markdown
# Migration Guide: nexus-combat 0.2.0 → 0.3.0

## Breaking Changes

### `CombatMachine::attack()` → `CombatMachine::attack_with_combo()`

**Before:**
```rust
machine.attack()?;
```

**After:**
```rust
machine.attack_with_combo(ComboSequence::Basic)?;
```

**Rationale:** Explicit combo specification improves clarity and balancing.
```

### 4. Release Cadence

| Version | Frequency | Scope |
|---------|-----------|-------|
| **Patch** | Weekly | Bug fixes, non-breaking improvements |
| **Minor** | Monthly | New features, deprecations |
| **Major** | Quarterly | Breaking changes, large refactors |

**Release Process:**

```bash
# 1. Bump version in all Cargo.toml files
cargo release version X.Y.Z

# 2. Update CHANGELOG.md
vim CHANGELOG.md

# 3. Create tag
git tag -a vX.Y.Z -m "Release X.Y.Z"

# 4. Run full CI
./rocket test
./rocket audit

# 5. Push
git push origin main
git push origin vX.Y.Z

# 6. Publish (if public)
cargo publish --manifest-path tools/rocket-sdk/Cargo.toml
```

### 5. Code Review Standards

**Every PR must:**
- [ ] Pass all CI checks (lint, tests, audit)
- [ ] Have ≥2 approvals from maintainers (1 for docs-only changes)
- [ ] Include a clear description (why, not just what)
- [ ] Reference related issues (e.g., "Fixes #42")
- [ ] Update `CHANGELOG.md` if user-facing
- [ ] Update `CLAUDE.md` if architecture changes

**Reviewers look for:**
1. **Correctness** — does it actually work?
2. **Safety** — could this introduce a bug?
3. **Clarity** — is it understandable?
4. **Performance** — are there bottlenecks?
5. **Sustainability** — will this be easy to maintain in 5 years?

---

## Glossary

| Term | Definition |
|------|-----------|
| **TPS** | Toyota Production System; manufacturing philosophy focused on eliminating waste |
| **DfLSS** | Design for Long-term Sustainable Success; engineering discipline |
| **Chicago School TDD** | Test-driven development philosophy that drives real systems, no mocks |
| **Jidoka** | Japanese manufacturing term meaning "autonomation"; automated stop on defect |
| **E2E** | End-to-end test; tests full system from user input to output |
| **Typestate Pattern** | Compile-time state machine using PhantomData<S> type parameters |
| **Resolver 3** | Rust workspace resolver that detects transitive dependency conflicts |
| **MCP** | Model Context Protocol; allows Claude Desktop to use custom tools |
| **LSP** | Language Server Protocol; allows editors to integrate language services |
| **OCEL** | Object-Centric Event Log; format for recording process executions |
| **Proptest** | Property-based testing library; generates random test cases |
| **Blake3** | Cryptographic hash function; used for artifact signing |
| **Emscripten** | Toolchain for compiling C++ to WebAssembly |
| **AutomationTool** | UE4 command-line tool for headless builds |

---

## Appendix: Command Reference

### Building

```bash
./rocket setup               # Bootstrap
./rocket build              # Build UE4 projects
./rocket build -p ShooterGame -t ShooterGame -l Win64
./rocket sync               # Sync manifest
```

### Testing

```bash
./rocket test               # All tests
cd nexus-engine && cargo test --all
cd pwa-staff && npm test
cd chicago-tdd-tools && cargo test --all-features
```

### Asset Pipeline

```bash
cd .claude/worktrees/agent-a63d171fb05007da1/asset-pipeline
cargo build
./target/debug/asset-pipeline --config pipeline.toml watch
```

### Code Quality

```bash
./rocket audit              # Semantic laws + anti-llm scanning
./rocket pwa lint           # ESLint + Prettier
cargo fmt --workspace       # Format all Rust
cargo clippy -- -D warnings # Lint all Rust
npx tsc --noEmit           # Type-check TypeScript
```

### Deployment

```bash
./rocket crypto generate --platform android
git tag -a v0.5.0 -m "Release 0.5.0"
git push origin v0.5.0     # Triggers release workflow
```

---

## Contact & Support

**Issues:** GitHub Issues (filtered by label: `bug`, `feature`, `rfc`)  
**Discussion:** GitHub Discussions (for RFCs, design questions)  
**Code Review:** Pull requests with ≥2 maintainer approvals  

---

## Document Metadata

**Version:** 1.0  
**Last Updated:** 2026-06-18  
**Authors:** Sean Chatman, Rocket Craft Engineering Team  
**Status:** Active  
**Review Frequency:** Quarterly (or when major architectural change occurs)  

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2026-06-18 | 1.0 | Initial DFLSS document; integrated TPS/DfLSS E2E, Chicago TDD, anti-LLM, and recent gap-closing work |

