# unify-rs — CLAUDE.md

## Purpose

17-crate Rust workspace providing the semantic web layer, developer tooling
integrations, and cross-cutting infrastructure for the entire Rocket Craft monorepo.
Bridges RDF/semantic-web representations of UE4 projects with: an MCP server (AI
tool use), an LSP server (editor integration), WASM compilation targets, OpenTelemetry
observability, Node.js FFI, object-centric event logs (OCEL), and Blueprint codegen.
It is the integration hub — most other workspaces are consumed here.

## Directory Structure

```
unify-rs/
├── Cargo.toml                    # Workspace root (resolver = "2")
├── unify/                        # Main CLI binary
│   └── src/{app,commands,lib,main,output,version}.rs
├── unify-core/                   # Shared primitives (no external deps)
│   └── src/lib.rs
├── unify-sem/                    # Semantic type algebra
│   └── src/lib.rs
├── unify-config/                 # Config loading, manifest merging, validation
│   └── src/{lib,loader,manifest,merge,sections,tests,validate}.rs
├── unify-codegen/                # Code generation from semantic descriptions
│   └── src/lib.rs
├── unify-rdf/                    # RDF triple store, SPARQL, SHACL validation
│   └── src/{lib,triple,store,manifest,pipeline,sparql,shacl,project_bridge}.rs
├── unify-sem/                    # Semantic type system
│   └── src/lib.rs
├── unify-lsp/                    # Language Server Protocol implementation
│   └── src/{lib,capability,compositor,conformance,diagnostic,gate,snapshot,tests}.rs
├── unify-mcp/                    # Model Context Protocol MCP server (binary)
│   └── src/{lib,main,protocol,resource,server,tool,tools,rocket_tools}.rs
├── unify-ffi/                    # Node.js FFI via napi-rs shim
│   └── src/{lib,convert,napi_shim,registry,types}.rs
├── unify-admission/              # Admission control / policy enforcement
│   └── src/lib.rs
├── unify-bp/                     # Blueprint semantic bridge
│   └── src/{lib,classify,codegen,gate,ocel,pwa_export,receipt}.rs
├── unify-ocel/                   # Object-centric event logs (IEEE XES / OCEL spec)
│   └── src/lib.rs
├── unify-otel/                   # OpenTelemetry tracing/metrics export
│   └── src/lib.rs
├── unify-pm/                     # Process mining integration
│   └── src/lib.rs
├── unify-receipts/               # Immutable operation receipts
│   └── src/{lib,receipt}.rs
├── unify-rocket/                 # Rocket Craft SDK integration adapter
│   └── src/lib.rs
├── unify-macros/                 # Proc macros for unify DSL
│   └── src/lib.rs
├── unify-test/                   # Test framework: chicago-tdd, golden, scenario
│   └── src/{lib,assertions,chicago,coverage,fixture,golden,scenario,tracker}.rs
└── unify-integration-tests/      # End-to-end integration tests
    └── src/{lib,assert,chains,fixtures}.rs
```

## Key Commands

```bash
# Build entire workspace
cargo build --workspace

# Build and run the unify CLI
cargo run -p unify -- --help

# Build and run the MCP server
cargo run -p unify-mcp

# Build in release
cargo build --release --workspace

# Run all tests (unit + integration)
cargo test --workspace

# Run integration tests only
cargo test -p unify-integration-tests

# Run tests with output
cargo test --workspace -- --nocapture

# Check formatting
cargo fmt --workspace -- --check

# Format
cargo fmt --workspace

# Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build docs
cargo doc --workspace --no-deps --open

# Build WASM target (requires wasm32 toolchain)
rustup target add wasm32-unknown-unknown
cargo build -p unify-wasm --target wasm32-unknown-unknown
```

## Crate Responsibilities

| Crate                    | Role                                                                         |
|--------------------------|------------------------------------------------------------------------------|
| `unify-core`             | Zero-dep primitives: `UnifyId`, `Timestamp`, `Version`, shared error base   |
| `unify-sem`              | Semantic type algebra; `SemanticType`, `Constraint`, `Unification` logic     |
| `unify-config`           | `Config` loading from TOML/JSON; deep merge; section validation              |
| `unify-codegen`          | Generates Rust/TypeScript/T3D code from semantic descriptions                |
| `unify-rdf`              | RDF triple store (`Store`); SPARQL query engine; SHACL shape validation; project bridge |
| `unify-lsp`              | LSP server: diagnostics, completions, hover, document snapshots              |
| `unify-mcp`              | MCP server binary; exposes Rocket Craft tools as MCP resources+tools         |
| `unify-ffi`              | Node.js FFI bridge via napi-rs shim; type conversions; registry              |
| `unify-admission`        | Policy admission controller; gates operations against declared policies       |
| `unify-bp`               | Integrates `blueprint-core` (from `blueprint-rs/`) with semantic layer       |
| `unify-ocel`             | Object-centric event log (OCEL 2.0) recorder and exporter                   |
| `unify-otel`             | OpenTelemetry span/metric export; wraps `tracing` + exporters                |
| `unify-pm`               | Process mining: discover process models from OCEL logs                       |
| `unify-receipts`         | Append-only immutable receipts for every operation (audit trail)             |
| `unify-rocket`           | Adapts `rocket-sdk` (from `tools/`) types for use within unify-rs            |
| `unify-macros`           | Proc macros: `#[unify_type]`, `#[semantic]`, `#[rdf_resource]`              |
| `unify-test`             | Chicago-school TDD harness; golden files; scenario runner; coverage tracker  |
| `unify-integration-tests`| End-to-end chain tests across crates; fixture database; assertion library    |
| `unify` (binary)         | CLI: `unify check`, `unify validate`, `unify generate`, `unify serve`        |

## Key Types and Patterns

### RDF Triple Store (`unify-rdf/src/store.rs`, `triple.rs`)
```rust
store.insert(Triple { subject, predicate, object });
store.query_sparql("SELECT ?s WHERE { ?s rdf:type ue4:Project }");
```
SHACL shapes in `shacl.rs` validate store contents against declared constraints.

### MCP Server (`unify-mcp/src/server.rs`)
Implements the Model Context Protocol. Resources expose project state; tools wrap
`rocket-sdk` operations. The binary `unify-mcp` starts an MCP-over-stdio server
suitable for use with Claude Desktop or any MCP client.

### Blueprint Bridge (`unify-bp/`)
`unify-bp` imports `blueprint-core` (path dep: `../../blueprint-rs/blueprint-core`).
- `classify.rs` — classifies semantic descriptions into Blueprint node types
- `codegen.rs` — calls `BlueprintBuilder` to produce ASTs
- `gate.rs` — admission gate: validates a blueprint against policies before emit
- `ocel.rs` — records Blueprint generation events to the OCEL log
- `pwa_export.rs` — exports blueprint metadata for the PWA HUD
- `receipt.rs` — writes an immutable receipt for each generated Blueprint

### Config Merge (`unify-config/src/merge.rs`)
Deep merge of multiple config sources (file, env, CLI flags). Later sources win on
scalar values; arrays are appended. Use `Config::load_and_merge(sources)` — do not
merge configs manually.

### LSP Snapshots (`unify-lsp/src/snapshot.rs`)
The LSP maintains versioned document snapshots. Do not hold onto a `Snapshot` across
async yield points — it may be superseded. Always re-fetch from `Compositor`.

## Workspace Dependencies

Shared in `[workspace.dependencies]`:

| Dep          | Version | Notes                       |
|--------------|---------|-----------------------------|
| `serde`      | 1       | `features = ["derive"]`     |
| `serde_json` | 1       |                             |
| `thiserror`  | 1       |                             |
| `anyhow`     | 1       |                             |
| `blake3`     | 1       | Content hashing             |
| `tracing`    | 0.1     |                             |
| `tokio`      | 1       | `features = ["full"]`       |
| `clap`       | 4       | `features = ["derive"]`     |

## Relation to the Monorepo

- **`blueprint-rs/blueprint-core`** — `unify-bp` depends on it via relative path.
  Keep both workspaces in sync when changing Blueprint AST types.
- **`tools/rocket-sdk`** — `unify-rocket` wraps it. When `rocket-sdk` API changes,
  update `unify-rocket/src/lib.rs` first.
- **`pwa-staff/`** — `unify-bp/src/pwa_export.rs` generates JSON consumed by the
  PWA HUD. The output path must match what `pwa-staff` expects.
- **`nexus-engine/`** — `unify-mcp` can expose nexus game state as MCP resources.
- **`versions/`** — `unify-rdf/src/project_bridge.rs` maps `versions/` UE4 projects
  into RDF triples.

## Caveats and Gotchas

- **`unify-mcp` is a binary crate**: running `cargo test --workspace` will also build
  it. If the MCP server fails to compile, all workspace tests fail.
- **FFI crate (`unify-ffi`) requires napi-rs toolchain for Node.js builds**: standard
  `cargo build` skips the napi output. To build the `.node` file for Node.js:
  `npm run build` inside a wrapper package (not yet scaffolded in this workspace).
- **WASM target**: `unify-wasm` must be built with `--target wasm32-unknown-unknown`.
  It will not compile for native targets due to `no_std` constraints.
- **16-crate build is slow**: use `cargo build -p <crate>` during development rather
  than `--workspace` unless you need everything.
- **`unify-macros` is a proc-macro crate**: changes to it force a full rebuild of all
  dependent crates. Keep it minimal; derive macros only.
- **OCEL log is append-only**: never mutate or delete entries in the OCEL store.
  Query via `unify-pm` for process mining analysis.
- **Receipts are immutable**: `unify-receipts` uses blake3 chaining. Do not implement
  a `delete` or `update` operation on `Receipt`.
