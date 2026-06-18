# Original User Request

## Initial Request — 2026-06-17T08:51:12-07:00

Build the Genie 26 World Manufacturing Platform based on the Version Vision 2030 PRD and ARD. The platform must parse user intent, manufacture Unreal 4 world artifacts, deploy a playable world, and support operating and evolving the world over time.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Reference Material

### Product Requirements Document (PRD)

```markdown
# Genie 26
## Product Requirements Document (PRD)

### Version
Vision 2030

### Product Category
World Manufacturing Platform

---

# Executive Summary
Genie 26 enables a user to describe a world using natural language and receive a playable Unreal 4 world that can be deployed, operated, modified, and shared.
The objective is not to generate images, videos, or temporary simulations.
The objective is to manufacture durable operational worlds.
Every generated world becomes a persistent asset capable of supporting exploration, operations, training, commerce, collaboration, and simulation.

# Problem Statement
Modern world creation requires specialized expertise across multiple disciplines.
Creating a playable environment requires environment design, asset selection, gameplay implementation, navigation setup, interaction design, deployment preparation, and operational maintenance.
This process can take weeks, months, or years.
Users want to describe a world and begin using it immediately.

# Vision
Any individual should be able to create a deployable Unreal 4 world using intent alone.
A world should be generated, launched, and made operational within minutes.
The generated world should remain editable and continuously improvable.

# Success Criteria
A user can:
1. Describe a world.
2. Generate a world.
3. Enter the world.
4. Modify the world.
5. Share the world.
6. Operate the world.
7. Improve the world over time.
Without requiring traditional world-development workflows.

# Product Principles
- Principle 1: Worlds Over Content (manufacture worlds, not media)
- Principle 2: Persistent Ownership (worlds remain durable assets)
- Principle 3: Continuous Evolution (worlds improve through iteration)
- Principle 4: Immediate Utility (worlds provide operational value immediately)
- Principle 5: Human Intent First (intent remains the primary authoring mechanism)

# Functional Requirements
- World Generation: Describe world, purpose, scale, environmental characteristics, and interaction goals to generate a playable Unreal 4 world.
- World Deployment: Launch, access, re-enter, and share worlds as a standard operation.
- World Modification: Add/remove locations, add/remove actors, modify rules, appearance, and interactions without rebuilding from scratch.
- World Persistence: Preserve state, structure, relationships, and modifications across sessions.
- Multi-User Collaboration: Enter, operate, and modify worlds together.
- World Operations: Monitor activity, manage participants, observe interactions, and review world history.
- World Lifecycle: Support creation, operation, expansion, optimization, archival, and restoration.
```

### Architecture Requirements Document (ARD)

```markdown
# Genie 26
## Architecture Requirements Document (ARD)

### Version
Vision 2030

# Architectural Goal
Transform user intent into operational Unreal 4 worlds.
The architecture exists to manufacture, operate, and evolve worlds.

# Core Architectural Model
Intent → World Specification → World Manufacturing → World Deployment → World Operation → World Evolution

# Architectural Layers
1. Intent Layer: Receive intent, clarify intent, preserve intent, and translate intent into world requirements. Output: World specification.
2. World Specification Layer: Represent the intended world, defining locations, actors, interactions, rules, objectives. Output: Manufacturable world definition.
3. World Manufacturing Layer: Construct Unreal 4 worlds (build environments, assemble world structures, create navigable spaces, configure interactions, produce deployable worlds). Output: Operational world.
4. Deployment Layer: Make worlds accessible (launch worlds, distribute worlds, manage access, support re-entry). Output: Accessible world.
5. Operations Layer: Support ongoing use (monitor activity, maintain state, coordinate participants, track world changes). Output: Operational continuity.
6. Evolution Layer: Improve worlds (accept modifications, expand environments, update rules, preserve continuity). Output: Improved world.

# Architectural Constraints
- Constraint 1: Every world must remain editable.
- Constraint 2: Every world must remain deployable.
- Constraint 3: Every world must remain operational.
- Constraint 4: World modification must preserve continuity whenever possible.
- Constraint 5: World ownership remains with the creator.

# World Model
A world consists of: Places, Actors, Objects, Relationships, Events, Rules, History.
```

---

## Requirements

### R1. Intent & Specification Layer
Implement parser and specification handlers to ingest natural language intent and translate it into a structured World Specification modeling:
1. Places (bounding volumes, coordinate systems)
2. Actors and Objects (positions, attributes)
3. Relationships (parent-child place containment, associations)
4. Rules (interaction behaviors, constraints)
5. Events and History (log of edits and state transitions)

### R2. World Manufacturing Layer
Construct a layout compiler that translates the structured World Specification into playable Unreal 4 level layout artifacts. The output layout must represent all places, actors, and objects in a standard Unreal 4 level format (such as T3D level layouts).

### R3. Deployment & Operations Layer
Build a deployment manager and an interactive simulator/dashboard to launch the world as a playable environment.
- The environment must render places, actors, and objects in 3D based on the generated layout.
- The environment must support real-time user-controlled actor/object interactions (e.g. movement, selection) and monitor participant/system activity.
- The environment must expose operational endpoints or controls to query active world state.

### R4. Evolution Layer
Implement an evolver that accepts new modification prompts (add/remove places, actors, objects, or change rules) and updates the World Specification and manufactured Unreal 4 level layout. The evolution must be incremental, preserving unmodified structures, relationships, and history logs without rebuilding from scratch.

### R5. E2E Verification Suite
Provide a programmatic verification script that runs an end-to-end integration scenario:
1. Generates a world specification and Unreal 4 layout file from an initial prompt.
2. Deploys the interactive simulation.
3. Evolve/modifies the active world with a modification prompt.
4. Validates that the updated layout and simulation state preserve the original structures while applying the changes.

---

## Acceptance Criteria

### Intent & Specification
- [ ] Natural language commands can be parsed into a structured World Specification modeling Places, Actors, Objects, Relationships, Rules, and History.
- [ ] A cryptographic receipt chain secures the world's history logs and state changes against tampering.

### World Manufacturing
- [ ] Compilation produces syntactically valid Unreal 4 compatible level layout files representing places as bounded environments and actors/objects as placed entities.

### Deployment & Operations
- [ ] The simulation environment can be started and renders all entities in 3D according to their specified coordinates.
- [ ] Users can interactively control or manipulate entities within the active deployment.

### Evolution & Continuity
- [ ] Modifying the world updates the active layouts incrementally, maintaining unchanged places and actors and updating the event/receipt history.

### Verification
- [ ] The verify_genie.sh script runs the generation and evolution process on a sample case and exits with code 0 on success.

## Follow-up — 2026-06-17T18:00:30Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Execute the Teamwork Multi-Agent System on the problem constraints.

Refactor Rocket-Craft into a browser-native Unreal 4 world manufacturing pipeline using the SpeculativeCoder UE4.27 HTML5 ES3 fork.

Working directory: ~/rocket-craft
Integrity mode: benchmark

**Reference Material:**
- UE4 HTML5 ES3 Fork: [SpeculativeCoder/UnrealEngine-HTML5-ES3](https://github.com/SpeculativeCoder/UnrealEngine-HTML5-ES3)

## Requirements

### R1. Target Output Architecture
The pipeline must output a static HTML5/WebGL2/WASM Unreal 4 package. Pixel Streaming is forbidden. Success is defined exclusively as a packaged, playable browser world—not just a passing simulator or unit test suite.

### R2. End-to-End Playwright Verification
Use Playwright as the mandatory end-to-end verifier. The verification script must open the locally served HTML5 output, send movement input into the canvas, and capture screenshots before and after the input.

### R3. Combinatorial Maximalism (Parallel Uncertainties)
Launch parallel agents against every independent uncertainty. No agent may claim project verified; agents may only claim local receipts. The orchestrator admits only end-to-end receipts.
The independent uncertainties to attack in parallel are:
- UE4 fork setup
- HTML5 packaging
- minimal project creation
- generated world artifact import
- Rocket-Craft contract refactor
- headless build automation
- local web serving
- Playwright browser validation
- visual screenshot comparison
- repair-loop automation

## Acceptance Criteria

### End-to-End Pipeline
- [ ] A Prompt is converted to a Rocket-Craft world contract.
- [ ] Unreal 4 world artifacts are successfully generated.
- [ ] The world is packaged via the SpeculativeCoder UE4.27 HTML5 build/package.
- [ ] The static package is served locally.

### Verifiable Browser Receipt
- [ ] Playwright opens the locally served world without errors.
- [ ] Playwright successfully sends movement input to the running WebGL instance.
- [ ] The screenshot delta (before and after movement) proves motion occurred.
- [ ] A receipt records the build log, package path, browser URL, screenshots, input trace, and verdict.

## Follow-up — 2026-06-17T18:02:12Z

Implement the TPS/DfLSS Playwright Manufacturing Strategy as the governing acceptance system for Rocket-Craft.

Do not treat this as documentation. Treat it as law.

Refactor Rocket-Craft so the final authority is Playwright visual verification of a browser-native Unreal 4 HTML5/WASM world built with the SpeculativeCoder UE4.27 HTML5 ES3 fork.

The pipeline must not accept:
- Rust-only simulation
- CLI emulation
- mocked worlds
- unit-test-only success
- package-only success
- compile-only success
- screenshots without input actuation
- input actuation without visual delta
- visual delta without receipt

The accepted crown path is:

Prompt
→ Rocket-Craft Contract
→ Unreal 4 world artifact
→ HTML5/WASM package
→ local browser launch
→ Playwright waits for engine readiness
→ Playwright captures baseline screenshot
→ Playwright sends movement input
→ Playwright captures after screenshot
→ visual delta is computed
→ browser console logs are captured
→ cryptographic receipt is produced

If visual motion delta is below threshold, mark DEFECT and route repair by failure taxonomy.

verified requires a replayable receipt proving the generated world was visible, loaded, responsive, and moved under input.

## Add this acceptance matrix

GATE 0 — Source Admission
PASS only if Rocket-Craft has a declared world contract for the prompt.

GATE 1 — Unreal Artifact Admission
PASS only if Rocket-Craft emits Unreal 4-consumable world artifacts.

GATE 2 — HTML5/WASM Package Admission
PASS only if the SpeculativeCoder UE4.27 HTML5 ES3 build produces browser-deployable output.

GATE 3 — Browser Load Admission
PASS only if Playwright opens the packaged world and detects engine readiness.

GATE 4 — Visual World Admission
PASS only if screenshot shows a non-error WebGL/Unreal scene.

GATE 5 — Actuation Admission
PASS only if keyboard input is injected.

GATE 6 — Motion Admission
PASS only if after-screenshot differs from before-screenshot above threshold.

GATE 7 — Receipt Admission
PASS only if prompt, contract hash, build log, package path, screenshots, console logs, input trace, visual delta, and final verdict are recorded.

## The repair routing law

No generic repair loop.

Every failure must route to a cell:
- UE4 fork/build cell
- HTML5 packaging cell
- Rocket-Craft contract cell
- Unreal artifact generation cell
- local serving cell
- Playwright browser-load cell
- WebGL/runtime cell
- input-binding cell
- visual-delta cell
- receipt/audit cell

## The command to agents

Stop proving that code exists.

Prove that the world drives.


## Follow-up — 2026-06-17T23:24:31Z

Resolve all implementation gaps, stubs, placeholders, single-line functions, assertion shortcuts, debug macros, and overclaiming terms in the Rocket-Craft project. Ensure the entire codebase is production-ready, fully compliant with Anti-LLM guidelines, and passes the complete test suite.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Complete Stubs and Placeholders
Fully implement all code paths and files marked with `STUB` or acting as placeholders with complete, production-ready business logic:
- `unify-rs/unify-bp/src/pwa_export.rs`
- `unify-rs/unify-integration-tests/src/fixtures.rs`
- `unify-rs/unify-rdf/src/pipeline.rs`
- Any other files containing `STUB` comments or placeholder blocks across the workspace.

### R2. Replace Single-Line and Catch-All Stubs
Replace all hardcoded/empty single-line functions, placeholder method bodies, and empty catch-all match arms with robust, functional code:
- Replace the single-line `description` method in `classify.rs` with descriptive logic.
- Fully implement `namespace`, `noun`, and `verb` functions in `unify-core/src/lib.rs` and `unify-rocket/src/lib.rs`.
- Replace empty catch-all `_ => {}` matches in `manifest.rs` with proper error handling/logging.
- Fully implement the lifecycle stubs `on_start` and `on_stop` in `wasm-patterns/src/actor.rs`.
- Replace dummy helper functions in `wasm-tests/tests/pattern_integration.rs`.

### R3. Harden Assertions & Eliminate Test Shortcuts
Refactor tests relying on substring matching or text-based shortcut assertions to use precise, schema-compliant structural validations:
- Refactor string assertions in `unify-integration-tests/src/lib.rs`.
- Refactor HUD display/substring searches in `wasm-ui/tests/hud.rs` and `wasm-ui/tests/message_bridge.rs`.

### R4. Remove Debug Macro Leakage
Clean up and replace all print statements (`println!`) in compiler scripts, FFI modules, and MCP/CLI binary entrypoints with structured logging or tracing:
- `unify-ffi/build.rs`
- `unify-mcp/src/main.rs`
- `unify/src/commands.rs` and `unify/src/main.rs`

### R5. Eliminate Overclaim/verified Language
Remove any unverified overclaiming status tags (like `zero violations`, `readdressed`, `done`) in code comments or logs.

## Acceptance Criteria

### Compilation & Workspace Testing
- [ ] The entire `unify-rs` and `wasm-threads` workspaces compile without any errors or warnings.
- [ ] Running `cargo test --workspace` passes 100% of all unit and integration tests.
- [ ] No stubs, placeholders, or `TODO` comments remain anywhere in the code.

### Compliance & Quality
- [ ] The `anti-llm-cheat-lsp` compliance scanner returns 0 errors/violations across the entire codebase.
- [ ] The E2E Playwright test suite passes successfully.

## Follow-up — 2026-06-18T00:30:26Z

# Teamwork Project Prompt

Build a combinatorial testing engine using `chicago-tdd-tools` in Rust that discovers all game servers and Infinity Blade code in the project. The engine must systematically simulate all possible game logic states and permutations for all the actual games in this project, functioning like an "aimbot" that automatically explores the entire game state space to ensure nothing is left untested.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Combinatorial Test Engine Construction
Use Rust and `chicago-tdd-tools` to build an engine capable of discovering and hooking into all game servers and Infinity Blade simulation code within the `rocket-craft` codebase. 

### R2. Systematic State Exploration
The engine must simulate and traverse all possible game states, moves, and logic branches for the specific games found in this project (achieving combinatorial maximalism).

## Acceptance Criteria

### Combinatorial Verification
- [ ] A Rust binary exists that, when run, automatically loads the game logic and systematically executes a combinatorial simulation across all discovered games.
- [ ] The engine produces a verifiable log, matrix, or report demonstrating that the entire permutation space of game rules, entities, and moves was explored without encountering unhandled panics or invalid states.
- [ ] The test coverage and state traversal run completely autonomously from start to finish.

## Follow-up — 2026-06-18T00:31:39Z

Additional context regarding the Combinatorial Maximalism engine:

"it is like the chess coor system" (referring to the chess coordinate system).

## Follow-up — 2026-06-18T00:39:32Z

Critical directive from user:

"make sure to distrust all first responses"

Adopt a stance of absolute skepticism. Do not accept the first successful test run or the first implementation at face value. Actively verify, cross-check, and assume initial results may be flawed or a false positive until rigorously proven otherwise.

