# System Architecture Overview

This document explains the high-level design decisions, structural patterns, and architectural layout of the Rocket-Craft ecosystem. It connects the Rust SDK/CLI spines, the Gundam Nexus game engine, the semantic RDF registry, the Model Context Protocol (MCP) server, and the TPS/DfLSS World Manufacturing Pipeline.

---

## 1. Architectural Layout

Rocket-Craft is structured as an integrated multi-workspace monorepo, where independent Rust workspaces handle specialized domain logic while communicating via standard CLI interfaces and JSON-RPC protocols.

```text
               +-------------------------------------------+
               |                rocket CLI                 |
               +-------------------------------------------+
                                     |
                                     v
               +-------------------------------------------+
               |        unify CLI / unify-automl           |
               +-------------------------------------------+
                /         |                      \        \
               v          v                       v        v
        +----------+ +----------+           +----------+ +-----------+
        |  genie-  | |  unify-  |           | chicago- | |  nexus-   |
        |   core   | |   rdf    |           |   tdd    | |  engine   |
        +----------+ +----------+           +----------+ +-----------+
             |            |                       |            |
             v            v                       v            v
        UE4 T3D      SHACL / SPARQL          BFS Traversal  Typestate
        Layouts      Triple Store            State Space    Generics
```

---

## 2. Compile-Time Safety: The Typestate Pattern

To prevent runtime errors, the ecosystem relies heavily on the **Typestate Pattern** (most notably formalized in `rocket-sdk` as `Machine<L: Law, P>` and across the `nexus-engine` and `unify-rs` crates).

### Why Typestate?
Traditional systems use runtime enum checks (e.g., `if state == State::Connected`). If a developer calls a function that is invalid in the current state, the system must throw a runtime exception or handle a failure path. 

The typestate pattern moves these rules to the **compiler**. States are represented as zero-sized structures (phantom types). Transition methods are only implemented on the specific state structures where they are legal. If you attempt an invalid transition, the code fails to compile.

### Key Examples in the Codebase
1. **Multiplayer Networking (`nexus-net/src/connection.rs`)**:
   Enforces socket phases:
   `Disconnected` -> `Handshaking` -> `Connected` -> `Authenticated` -> `InLobby` -> `InMatch`.
   You cannot join matchmaking from the `Disconnected` state because the `join_matchmaking` function is only implemented for `Connection<InLobby>`.
2. **Combat Machine (`nexus-combat/src/machine.rs`)**:
   Tracks entity phases: `CombatMachine<Idle>` -> `CombatMachine<Attacking>` -> `CombatMachine<Stunned>`.
3. **Project Manifest Ingestion (`unify-rdf/src/project_bridge.rs`)**:
   Tracks project state: `ProjectManifest<Pending>` -> `ProjectManifest<Ingested>` -> `ProjectManifest<Validated>`.
4. **Law Gates (`rocket-sdk`)**:
   Combines a `Law` validation trait with a generic Phase marker (`Input` -> `Validated` -> `Admitted`).

---

## 3. Semantic Metadata Registry: RDF & SHACL

The `unify-rdf` and `unrdf` crates model project metadata using **Semantic Web (RDF)** standards rather than rigid SQL schemas.

### Why RDF?
In a multi-game monorepo, project configurations, asset pipelines, and network configurations differ wildly. RDF models metadata as a directed graph of **Triples** (`Subject`, `Predicate`, `Object`), allowing new properties to be attached without database migrations.

### How it operates
- **Manifest Bridge**: Parses `project-manifest.json` and imports properties into the `TripleStore` database.
- **SPARQL Queries**: The `query` command matches triple patterns (e.g., `?game hasTarget ?target`) to extract relationships.
- **SHACL Constraints**: Shape definitions (written in `shacl.rs`) act as schemas, asserting invariants (e.g., "Every active game must declare an entry point and target platform"). If a constraint is violated, validation fails and prevents build tasks.

---

## 4. AI Tooling Integration: Model Context Protocol (MCP)

To allow LLMs and automated developers to interact safely and structured with the repository, `unify-mcp` implements a Rust-native **Model Context Protocol (MCP) Server**.

### Architecture
The server is based on a JSON-RPC 2.0 transport channel (typically running over standard input/output). It registers:
1. **Tool Registry**:
   Exposes commands like `rocket/manifest/list` or `rocket/audit/run`. The AI can execute these tools to read configurations or trigger validations without direct shell command execution.
2. **Resource Registry**:
   Exposes system files, logs (like `pipeline_run.log`), and active specifications as read-only URIs. The AI can subscribe to updates or pull these resources into context dynamically.

---

## 5. The Playwright Manufacturing Strategy (TPS/DfLSS)

The absolute source of truth in Rocket-Craft is the **TPS/DfLSS (Toyota Production System / Design for Lean Six Sigma)** verification standard.

### Core Principle
Passing unit tests or green compiler checks are **false positives**. They only prove that the parts fit together statically, not that the software works in its execution environment.

### The Accepted Crown Path
To verify a change:
1. Compile the level specification down to a **WorldSpec JSON**.
2. Compile the layout into a **UE4 T3D file** and package it into an **HTML5/WASM bundle** using the SpeculativeCoder UE4.27 fork.
3. Spin up a local web server serving the packaged WASM.
4. Launch **Playwright (Chromium)** and wait for the engine to signal readiness (`window.UE4_EngineReady = true`).
5. Capture a **baseline screenshot**.
6. Inject keyboard/movement input (actuation).
7. Capture an **after-screenshot**.
8. Compute the **visual delta** (pixel difference).
9. Pass only if the visual delta is above the movement threshold, proving the game loaded, rendered WebGL, and successfully responded to input.
10. Generate a cryptographically signed **audit receipt** containing the BLAKE3 hashes.
