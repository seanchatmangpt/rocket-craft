# CLI Command Reference Manual

This document provides a complete syntax reference for all CLI tools and options in the Rocket-Craft unified ecosystem. This includes the `unify` CLI subcommands (`automl`, `dev`, `genie`, etc.) and the options for the `combinatorial-engine` state exploration runner.

---

## Global Options

These flags apply globally to all `unify` command dispatches.

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--json` | None | Boolean | `false` | Formats all standard output as machine-readable JSON. |
| `--quiet` | None | Boolean | `false` | Suppresses progress logging, warnings, and informational terminal messages. |
| `--help` | `-h` | Boolean | `false` | Prints help information and syntax usage overview. |

---

## Unify Subcommands

### Receipt Command
Compute a BLAKE3 cryptographic receipt for the given data payload.
- **Syntax**: `cargo run -p unify -- receipt --label <label> <data>`
- **Arguments**:
  - `<data>` (Positional, String): The raw content to hash.
- **Flags**:
  - `-l, --label <label>` (String, Required): The namespace or classification label of the receipt.

### Verify Command
Verify a BLAKE3 receipt chain stored as a JSON string or file.
- **Syntax**: `cargo run -p unify -- verify <chain_json>`
- **Arguments**:
  - `<chain_json>` (Positional, String): A JSON serialization containing the receipt key and hash to check.

### Gate Command
Evaluate a semantic compliance law gate on a data file.
- **Syntax**: `cargo run -p unify -- gate --law <law> --data <data>`
- **Flags**:
  - `-l, --law <law>` (String, Required): The name of the compliance law validator (e.g. standard rules).
  - `-d, --data <data>` (String, Required): The path to the data file under evaluation.

### Info Command
Display version metadata for all crates in the `unify-rs` workspace.
- **Syntax**: `cargo run -p unify -- info`

### Dispatch Command
Send a structured JSON noun-verb command payload.
- **Syntax**: `cargo run -p unify -- dispatch [--namespace <namespace>] <noun> <verb> [--input <input>]`
- **Arguments**:
  - `<noun>` (Positional, String): The resource type (e.g., `manifest`).
  - `<verb>` (Positional, String): The action to execute (e.g., `list`).
- **Flags**:
  - `-n, --namespace <namespace>` (String, Optional): Grouping namespace context.
  - `-i, --input <input>` (String, Optional): Supplementary input parameter JSON.

### Query Command
Execute a triple-pattern matching query over Turtle input data.
- **Syntax**: `cargo run -p unify -- query [--ttl <ttl>] <pattern>`
- **Arguments**:
  - `<pattern>` (Positional, String): SPARQL-style triple pattern string (e.g., `?s ?p ?o`).
- **Flags**:
  - `-t, --ttl <ttl>` (String, Optional): The path to the Turtle (.ttl) format metadata database.

### Witnesses Command
Retrieve the semantic witness registry logs for the workspace.
- **Syntax**: `cargo run -p unify -- witnesses [--domain <domain>]`
- **Flags**:
  - `-d, --domain <domain>` (String, Optional): Filter witnesses by a specific domain namespace.

### World-Parse Command
Translate natural language intent into a structured WorldSpec JSON.
- **Syntax**: `cargo run -p unify -- world-parse --intent <intent> --output <output>`
- **Flags**:
  - `-i, --intent <intent>` (String, Required): Natural language description of the level map.
  - `-o, --output <output>` (String, Required): Output path to write the WorldSpec JSON.

### World-Validate Command
Validate a WorldSpec JSON against semantic coherence rules.
- **Syntax**: `cargo run -p unify -- world-validate --spec <spec>`
- **Flags**:
  - `-s, --spec <spec>` (String, Required): Path to the WorldSpec JSON file.

### World-Generate Command
Compile a validated WorldSpec JSON into an Unreal Engine T3D level layout.
- **Syntax**: `cargo run -p unify -- world-generate --spec <spec> --output <output>`
- **Flags**:
  - `-s, --spec <spec>` (String, Required): Path to the WorldSpec JSON.
  - `-o, --output <output>` (String, Required): Output path for the copy-pasteable `.t3d` level file.

### World-Deploy Command
Deploy a manufactured world specification and launch the visualizer dashboard.
- **Syntax**: `cargo run -p unify -- world-deploy --spec <spec> --log <log>`
- **Flags**:
  - `-s, --spec <spec>` (String, Required): Path to the WorldSpec JSON file.
  - `-l, --log <log>` (String, Required): Output path for the deployment telemetry log.

### World-Evolve Command
Mutate an existing WorldSpec JSON using modification intent instructions.
- **Syntax**: `cargo run -p unify -- world-evolve --spec <spec> --intent <intent> --output <output>`
- **Flags**:
  - `-s, --spec <spec>` (String, Required): Path to the base WorldSpec JSON file.
  - `-i, --intent <intent>` (String, Required): Natural language evolutionary instructions.
  - `-o, --output <output>` (String, Required): Output path to write the evolved WorldSpec JSON.

---

## Genie Manufacturing Subcommands

Genie subcommands are accessed under the `genie` root namespace: `cargo run -p unify -- genie <subcommand>`.

### Manufacture Subcommand
Compile a level directly from intent down to spec and T3D clipboard layout in one call.
- **Syntax**: `cargo run -p unify -- genie manufacture --intent <intent> --out-spec <out_spec> --out-t3d <out_t3d>`
- **Flags**:
  - `-i, --intent <intent>` (String, Required): Natural language description.
  - `--out-spec <out_spec>` (String, Required): Destination path for the WorldSpec JSON.
  - `--out-t3d <out_t3d>` (String, Required): Destination path for the copy-pasteable T3D level mesh output.

### Evolve Subcommand
Mutate an existing world specification and regenerate its T3D level.
- **Syntax**: `cargo run -p unify -- genie evolve --spec <spec> --intent <intent> --out-spec <out_spec> --out-t3d <out_t3d>`
- **Flags**:
  - `-s, --spec <spec>` (String, Required): Path to the baseline WorldSpec JSON.
  - `-i, --intent <intent>` (String, Required): Mutation details.
  - `--out-spec <out_spec>` (String, Required): Evolved spec JSON destination.
  - `--out-t3d <out_t3d>` (String, Required): Evolved T3D level destination.

### Deploy Subcommand
Deploy a manufactured world to the server platform, outputting transaction logs.
- **Syntax**: `cargo run -p unify -- genie deploy --spec <spec> --log <log>`
- **Flags**:
  - `-s, --spec <spec>` (String, Required): Path to the WorldSpec JSON.
  - `-l, --log <log>` (String, Required): Path to write the deployment transaction log.

---

## AutoML Subcommands

AutoML subcommands are accessed under the `automl` root: `cargo run -p unify -- automl <subcommand>`.

### Discover Subcommand
Scan directories recursively to extract binding tag metrics from source files.
- **Syntax**: `cargo run -p unify -- automl discover [path]`
- **Arguments**:
  - `[path]` (Positional, Optional, Default: `.`): The root folder to start scanning. Traverses `.rs`, `.h`, `.cpp`, and `.hpp` files, skipping directories starting with a dot or named `target`.

### Optimize Subcommand
Run Monte Carlo simulations of combat encounters to balance point allocation.
- **Syntax**: `cargo run -p unify -- automl optimize [--points <points>] [--target <target>] [--sims <sims>]`
- **Flags**:
  - `-p, --points <points>` (Unsigned 32-bit Integer, Default: `8`): Total stat points to allocate.
  - `-t, --target <target>` (Float 64-bit, Default: `0.6`): Target player win rate decimal between `0.0` and `1.0`.
  - `-s, --sims <sims>` (Unsigned Size, Default: `20`): Simulation runs per stat configuration.

---

## Dev Lifecycle Subcommands

Developer commands are accessed under the `dev` root: `cargo run -p unify -- dev <subcommand>`.

### Init Subcommand
Bootstrap a developer workspace with configurations and test source files.
- **Syntax**: `cargo run -p unify -- dev init [path]`
- **Arguments**:
  - `[path]` (Positional, Optional, Default: `./dev_env`): The folder where scaffolding is written. Writes `dev_config.json` and `test_component.rs`.

### Start Subcommand
Launch the background developer Node.js server.
- **Syntax**: `cargo run -p unify -- dev start [path]`
- **Arguments**:
  - `[path]` (Positional, Optional, Default: `./dev_env`): The developer folder. Spawns `node genie_server.js` and writes the PID to `[path]/server.pid`.

---

## Combinatorial Engine

The combinatorial state explorer binary is run directly from the `chicago-tdd-tools` crate.

- **Syntax**: `cargo run -p chicago-tdd-tools --bin combinatorial-engine [options]`
- **Flags**:
  - `-o, --output <output>` (PathBuf, Default: `combinatorial_report.json`): The destination file path to write the full state traversal JSON report.
