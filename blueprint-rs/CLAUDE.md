# blueprint-rs — CLAUDE.md

## Purpose

Rust workspace for generating Unreal Engine 4 Blueprint assets programmatically.
Provides an AST for Blueprint node graphs, a builder DSL, T3D and JSON serializers,
a validator, a diff engine, and a CLI with file-watching. The output is `.t3d` text
files (UE4's text-based asset format) that can be imported directly into the UE4
Editor. Used by `unify-rs/unify-bp` to generate Blueprints from semantic descriptions.

## Directory Structure

```
blueprint-rs/
├── Cargo.toml              # Workspace root (resolver = "2")
├── Cargo.lock
├── blueprint-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Re-exports: ast, builder, serializer, validator, diff
│       ├── ast.rs           # Blueprint AST: BlueprintClass, Node, Pin, Graph
│       ├── builder.rs       # Builder pattern: BlueprintBuilder::new()...build()
│       ├── types.rs         # Primitive UE4 types, PinType, PropertyFlags
│       ├── nodes/
│       │   ├── mod.rs
│       │   ├── events.rs    # EventBeginPlay, EventTick, CustomEvent nodes
│       │   ├── flow.rs      # Branch, Sequence, ForLoop, WhileLoop nodes
│       │   ├── math.rs      # Add, Subtract, Multiply, Clamp, etc.
│       │   └── variables.rs # Get/Set variable nodes
│       ├── serializer/
│       │   ├── mod.rs
│       │   ├── t3d.rs       # T3D text format serializer (primary UE4 import format)
│       │   └── json.rs      # JSON serializer for tooling interchange
│       ├── parser.rs        # Parse existing .t3d files back to AST
│       ├── validator.rs     # Pin type compatibility, dangling connection checks
│       ├── diff.rs          # Structural diff between two Blueprint ASTs
│       ├── layout.rs        # Auto-layout for node positions
│       ├── patterns.rs      # Reusable node-graph pattern templates
│       ├── registry.rs      # Node type registry (name → NodeFactory)
│       ├── render.rs        # Blueprint visualization helpers
│       └── t3d.rs           # Low-level T3D primitives (Object blocks)
│   └── examples/
│       ├── hello_blueprint.rs   # Minimal working Blueprint
│       ├── fps_controller.rs    # FPS character controller Blueprint
│       └── ui_widget.rs         # UMG widget Blueprint
├── blueprint-cli/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs          # clap CLI + notify file-watcher
├── blueprint-macros/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       └── lib.rs           # Proc macros: #[blueprint_node], #[pin]
└── blueprint-testing/
    ├── Cargo.toml
    └── src/
        └── lib.rs           # Test fixtures, golden .t3d files, assertion helpers
```

## Key Commands

```bash
# Build entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Run examples
cargo run --example hello_blueprint -p blueprint-core
cargo run --example fps_controller -p blueprint-core
cargo run --example ui_widget -p blueprint-core

# Run the CLI (watch mode: regenerate .t3d on source change)
cargo run -p blueprint-cli -- watch --input ./blueprints --output ./out

# Run the CLI (one-shot generation)
cargo run -p blueprint-cli -- generate --input blueprint.json --output out.t3d

# Check formatting
cargo fmt --workspace -- --check

# Format in place
cargo fmt --workspace

# Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build proc macro crate alone
cargo build -p blueprint-macros
```

## Core Abstractions

### Blueprint AST (`src/ast.rs`)

The central type hierarchy:

```
BlueprintClass
  └── FunctionGraph[]
        └── Node[]
              ├── input_pins: Pin[]
              └── output_pins: Pin[]
```

`Node` has a `NodeKind` enum discriminating between Event, Function Call, Variable Get/Set, Flow Control, etc. `Pin` carries a `PinType` (bool, int, float, string, object, struct, exec) and an optional default value.

### Builder Pattern (`src/builder.rs`)

```rust
let bp = BlueprintBuilder::new("MyActor")
    .parent_class("AActor")
    .add_variable("Health", PinType::Float, 100.0)
    .begin_function("BeginPlay")
        .add_node(NodeKind::EventBeginPlay)
        .connect("EventBeginPlay.exec_out", "PrintString.exec_in")
    .end_function()
    .build()?;
```

### T3D Serializer (`src/serializer/t3d.rs`)

Writes UE4's `Begin Object`/`End Object` format. The output is importable via UE4's
"Import" dialog or placed in `Content/` alongside `.uasset` files. Key method:
`T3dSerializer::serialize(&blueprint) -> String`.

### Validator (`src/validator.rs`)

Checks: pin type compatibility, no dangling exec pins, no orphaned nodes, no cycles
in the data-flow graph (cycles in exec flow are allowed for loops). Call
`validator::validate(&blueprint) -> Vec<ValidationError>` before serializing.

### Diff Engine (`src/diff.rs`)

Structural diff between two `BlueprintClass` values. Used by the CLI watch mode to
emit only changed nodes. Output is a `BlueprintDiff` with `added`, `removed`, and
`modified` node lists.

## Node Categories

| Module               | Node Types                                              |
|----------------------|---------------------------------------------------------|
| `nodes/events.rs`    | `EventBeginPlay`, `EventTick`, `EventOnDamage`, `CustomEvent` |
| `nodes/flow.rs`      | `Branch`, `Sequence`, `ForLoop`, `WhileLoop`, `Gate`   |
| `nodes/math.rs`      | `Add`, `Subtract`, `Multiply`, `Divide`, `Clamp`, `Lerp` |
| `nodes/variables.rs` | `GetVariable`, `SetVariable`, `GetArrayElement`        |

## Key Dependencies

| Crate        | Purpose                                 |
|--------------|-----------------------------------------|
| `serde`      | JSON serialization (`derive`)           |
| `serde_json` | JSON output for tooling interchange     |
| `uuid`       | Unique node/pin GUIDs (UE4 requires these) |
| `notify`     | File-system watching in CLI             |

## Relation to the Monorepo

- **`unify-rs/unify-bp`** depends on `blueprint-core` via path:
  `{ path = "../../blueprint-rs/blueprint-core" }`. When you change a public type in
  `blueprint-core`, rebuild `unify-bp` immediately.
- **`infinity-blade-4/Blueprints/`** — the `.t3d` files there were generated by
  `blueprint-cli`. Re-generate via:
  `cargo run -p blueprint-cli -- generate --input infinity-blade-4/...`
- **`blueprint-macros`** is a proc-macro crate and must be built before any crate
  that uses `#[blueprint_node]`. It has its own compilation step.

## Caveats and Gotchas

- **T3D format is UE4-specific**: the serializer targets UE4.24–4.27. UE5 uses a
  different format. Do not use `blueprint-core` for UE5 targets without updating
  `t3d.rs`.
- **GUIDs are required**: every `Node` and `Pin` needs a UUID. The builder auto-assigns
  them; if you construct AST nodes manually, call `Node::new_with_id(uuid::Uuid::new_v4())`.
- **Proc macros need `blueprint-macros` in scope**: the `#[blueprint_node]` macro
  requires `extern crate blueprint_macros;` on stable Rust. It's a separate compilation
  unit — changes to macros require a full rebuild of dependent crates.
- **Layout is optional**: `layout.rs` computes (x, y) positions for the UE4 node graph
  display. You can skip it, but the Blueprint will open with all nodes stacked at (0,0).
- **Circular data-flow is a validator error**: the validator rejects cycles in the data
  (value) graph. Exec-flow cycles (loops) are permitted and expected.
- **`blueprint-testing` is dev-only**: add it only as a `[dev-dependencies]` entry.
