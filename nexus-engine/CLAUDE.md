# nexus-engine — CLAUDE.md

## Purpose

10-crate Rust workspace implementing the backend engine for **Gundam Nexus**, a
multiplayer mobile game. Covers combat simulation, ECS world, economy/marketplace,
WebSocket networking, 3D math utilities, gacha shop, and full integration tests.
All crates use `workspace.dependencies` — add new deps there first, then inherit
with `dep = { workspace = true }`.

## Directory Structure

```
nexus-engine/
├── Cargo.toml                        # Workspace root; [workspace.dependencies] here
└── crates/
    ├── nexus-types/                  # Foundational types shared by all crates
    │   └── src/{lib,errors,ids,math}.rs
    ├── nexus-combat/                 # Typestate combat machine
    │   └── src/{lib,machine,combo,damage,events,parry}.rs
    ├── nexus-session/                # Player sessions, inventory, NPC dialogue
    │   └── src/{lib,session,player,inventory,npc}.rs
    ├── nexus-economy/                # Ledger, marketplace, auction, shop
    │   └── src/{lib,ledger,marketplace,auction,shop}.rs
    ├── nexus-net/                    # WebSocket protocol, matchmaking, rooms
    │   └── src/{lib,protocol,connection,matchmaking,message_codec,room}.rs
    ├── nexus-ecs/                    # hecs ECS, 20 components, 5 systems
    │   └── src/{lib,world,components,systems,scheduler}.rs
    ├── nexus-gfx/                    # 3D math, camera, frustum, bytemuck vertices
    │   └── src/{lib,math,camera,pipeline,vertex,color}.rs
    ├── nexus-shop/                   # ChaCha8 gacha, pity system, battle pass, AR
    │   └── src/{lib,gacha,battle_pass,ar_bridge}.rs
    ├── nexus-tests/                  # proptest harness, strategies, fuzz corpus
    │   └── src/{lib,strategies,invariants,model,fuzz}.rs
    └── nexus-integration/            # Full game loop, 22 end-to-end tests
        └── src/{lib,game_loop}.rs
```

## Key Commands

```bash
# Build entire workspace
cargo build --workspace

# Run all tests (unit + integration + proptest)
cargo test --workspace

# Run tests for a single crate
cargo test -p nexus-combat
cargo test -p nexus-integration

# Run proptest with more cases
PROPTEST_CASES=1000 cargo test -p nexus-tests

# Check formatting
cargo fmt --workspace -- --check

# Format in place
cargo fmt --workspace

# Clippy — all targets, deny warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build docs
cargo doc --workspace --no-deps --open
```

## Crate Responsibilities

| Crate              | Role                                                                 |
|--------------------|----------------------------------------------------------------------|
| `nexus-types`      | Phantom-typed IDs, math primitives (`Vec3`, `Quat`), error base types, typestates |
| `nexus-combat`     | Typestate combat FSM: `Idle→Attacking→Resolving→completed`; combo chains; parry timing |
| `nexus-session`    | Player session typestates; const-generic inventory `Inventory<N>`; NPC dialogue FSM |
| `nexus-economy`    | Double-entry ledger (debit == credit always); marketplace listings; auction typestates |
| `nexus-net`        | WebSocket framing with typestate protocol; duel matchmaking queue; `GameRoom` |
| `nexus-ecs`        | `hecs::World` wrapper; 20 component types; 5 systems; `SystemScheduler` |
| `nexus-gfx`        | `nalgebra`-backed typed 3D math; `Camera`, `Frustum`; `bytemuck` GPU vertex types |
| `nexus-shop`       | `rand_chacha::ChaCha8Rng` gacha with pity counter; battle pass season; AR item bridge |
| `nexus-tests`      | Centralized `proptest` strategies, model-based testing, fuzz corpus management |
| `nexus-integration`| Orchestrates all crates in a single game-loop; 22 end-to-end scenario tests |

## Key Types and Patterns

### Typestate Pattern (used in combat, session, economy, net)
States are zero-sized phantom types on a generic struct. Illegal transitions don't
compile. Example combat machine:
```rust
CombatMachine<Idle> -> CombatMachine<Attacking> -> CombatMachine<Resolving>
```

### Const-Generic Inventory
```rust
Inventory<const N: usize>  // capacity is a compile-time constant
```

### Double-Entry Ledger (`nexus-economy/src/ledger.rs`)
Every transaction has a debit and credit entry; the ledger enforces balance at the
type level. Never mutate ledger entries after insertion.

### ECS Component Types (`nexus-ecs/src/components.rs`)
20 components including `Position`, `Velocity`, `Health`, `CombatState`, `Faction`,
`Equipment`, `Inventory`, `NetworkId`. Use `hecs::World::spawn` with component tuples.

### proptest Strategies (`nexus-tests/src/strategies.rs`)
All domain strategies are centralized here. Reuse them in new test crates rather than
reimplementing — import `nexus-tests` as a dev-dependency.

## Workspace Dependencies

Shared versions declared in `Cargo.toml` `[workspace.dependencies]`:

| Dep              | Version | Notes                           |
|------------------|---------|---------------------------------|
| `serde`          | 1       | `features = ["derive"]`         |
| `tokio`          | 1       | `features = ["full"]`           |
| `nalgebra`       | 0.33    | `features = ["serde-serialize"]`|
| `glam`           | 0.27    | `features = ["serde"]`          |
| `hecs`           | 0.10    | `features = ["serde"]`          |
| `rand_chacha`    | 0.3     | Used in gacha RNG               |
| `proptest`       | 1       | Test-only                       |

## Relation to the Monorepo

- **`gundam-nexus/`** (sibling) — game design documents; the GDD and mobile suit
  specs that define what nexus-engine must implement.
- **`unify-rs/unify-mcp`** — the MCP server may expose nexus game state as resources.
- **`tools/rocket-sdk`** — build/audit tooling can be invoked against this workspace
  via `cargo run -p rocket-cmd -- build`.
- **`nexus-integration`** test regression file at
  `crates/nexus-integration/tests/integration_tests.proptest-regressions` — commit
  this file; it pins discovered edge cases.

## Caveats and Gotchas

- **proptest regressions**: The file
  `crates/nexus-integration/tests/integration_tests.proptest-regressions` must be
  committed. Never delete it — it records previously failing cases.
- **ChaCha8 seed**: The gacha engine in `nexus-shop` uses a seeded `ChaCha8Rng` for
  reproducibility in tests. Production code must seed from a secure source, not a
  hardcoded value.
- **hecs archetype churn**: Adding or removing components from an entity triggers
  archetype moves. Batch component additions at spawn time to avoid churn in hot paths.
- **nalgebra vs glam**: `nexus-gfx` uses `nalgebra` for typed math (matrices,
  quaternions). `glam` appears in workspace deps for potential use by other crates —
  do not mix them in the same calculation.
- **Workspace resolver 2**: Required for feature unification across the workspace.
  Do not downgrade to resolver "1".
- **nexus-tests is dev-only**: It exists only as a dev-dependency. Do not add
  `nexus-tests` as a regular dependency in any production crate.
