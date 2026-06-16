# asset-pipeline — CLAUDE.md

## Purpose

Autonomous 3D asset conversion pipeline for Rocket Craft. Watches directories for new
3D source files, validates and hashes them, converts to UE4-compatible FBX format via
headless Blender, then stages the results. The workspace exists to bridge raw artist
deliverables (`.obj`, `.fbx`, `.stl`, `.dae`, `.gltf`, `.glb`) into the UE4-ready
formats expected by the `versions/` UE4 projects.

## Directory Structure

```
asset-pipeline/
├── Cargo.toml               # Workspace root (members: pipeline-core, pipeline-cli)
├── pipeline-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Public API surface
│       ├── discover.rs      # File-system walk + MIME detection
│       ├── validate.rs      # Format validation, extension allow-list
│       ├── hash.rs          # blake3 content-addressing
│       ├── convert.rs       # Blender subprocess invocation
│       └── stage.rs         # Output staging, manifest write
└── pipeline-cli/
    ├── Cargo.toml
    └── src/
        └── main.rs          # clap CLI, notify file-watcher loop
```

> Note: This workspace is being created on branch `claude/inspiring-hamilton-jfww9e`.
> If the directories do not yet exist, run `cargo new --lib pipeline-core` and
> `cargo new pipeline-cli` inside `asset-pipeline/` to scaffold them.

## Key Commands

```bash
# Build everything
cargo build --workspace

# Build release binary
cargo build --release -p pipeline-cli

# Run all tests
cargo test --workspace

# Run the CLI watcher (requires BLENDER_PATH or blender in PATH)
cargo run -p pipeline-cli -- watch --input ./raw-assets --output ./staged

# Convert a single file
cargo run -p pipeline-cli -- convert path/to/model.obj --out ./staged

# Check formatting
cargo fmt --all -- --check

# Clippy (deny warnings)
cargo clippy --all-targets --all-features -- -D warnings
```

## Environment Variables

| Variable       | Required | Description                                     |
|----------------|----------|-------------------------------------------------|
| `BLENDER_PATH` | No       | Absolute path to the Blender executable. Falls back to `blender` on `$PATH`. |

Set it locally: `export BLENDER_PATH=/opt/blender/blender`

## Crate Responsibilities

### `pipeline-core` (library)
All domain logic lives here. Zero I/O in the typestate transitions themselves — I/O
is pushed to the call sites in the CLI.

- `discover.rs` — `walkdir` recursive scan; produces `DiscoveredAsset`
- `validate.rs` — extension + magic-byte checks; produces `ValidatedAsset`
- `hash.rs` — `blake3` digest of file content for change detection / dedup
- `convert.rs` — spawns `blender --background --python convert.py` as a subprocess;
  captures stdout/stderr; produces `ConvertedAsset`
- `stage.rs` — copies output FBX to staging dir; writes `manifest.json`; produces
  `StagedAsset`

### `pipeline-cli` (binary)
- `main.rs` — `clap` subcommands: `watch`, `convert`, `status`
- Uses `notify` crate for cross-platform file-system event watching
- Spawns `pipeline-core` pipeline stages in a `tokio` async runtime

## Typestate Pipeline

The core pattern is a linear typestate chain enforced at compile time:

```rust
DiscoveredAsset -> ValidatedAsset -> ConvertedAsset -> StagedAsset
```

Each transition is a fallible function returning `Result<NextState, PipelineError>`.
You cannot call `stage()` on a `DiscoveredAsset` — the type system prevents it.
Add new stages by defining a new struct and a conversion function; never skip states.

## Key Dependencies

| Crate      | Purpose                              |
|------------|--------------------------------------|
| `notify`   | Cross-platform file-system watching  |
| `walkdir`  | Recursive directory traversal        |
| `blake3`   | Fast cryptographic content hashing   |
| `tokio`    | Async runtime for the CLI            |
| `clap`     | CLI argument parsing                 |

## Supported Input Formats

`.obj` `.fbx` `.stl` `.dae` `.gltf` `.glb` — all converted to UE4 FBX via Blender's
Python scripting API (`bpy`).

## Relation to the Monorepo

- **Feeds `versions/`** — staged FBX files are consumed by UE4 projects in `versions/`
  via the UE4 Editor's content import pipeline.
- **Invoked by `tools/rocket-cmd`** — the `rocket` CLI's `sync` subcommand may trigger
  the pipeline to ensure assets are up-to-date before a build.
- **Hashes tracked by `unify-rs/unify-rdf`** — blake3 digests can be stored in the RDF
  triple store as asset provenance triples.

## Caveats and Gotchas

- **Blender must be headless**: always pass `--background` flag. GUI mode will hang.
- **Blender version sensitivity**: conversion scripts target Blender 3.x Python API.
  Blender 4.x changed `bpy.ops.export_scene.fbx` signatures — check the script if
  upgrading Blender.
- **Large files**: `.fbx` inputs can be 100 MB+. Do not commit binary assets to git;
  use Git LFS or the `versions/` content pipeline instead.
- **Windows paths**: `BLENDER_PATH` on Windows needs forward slashes or escaped
  backslashes in shell config.
- **Staging is not idempotent by default** — re-converting the same hash will
  overwrite the staged output. This is intentional; use the hash manifest to skip
  unchanged files.
