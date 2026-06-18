# Original User Request

## Initial Request — 2026-06-15T14:34:16-07:00

Implement a working progressive web app (PWA) integrated with a local Supabase instance, including fully functioning user authentication, profiles, player management admin dashboard, leaderboard, and end-to-end testing with Playwright.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Supabase Client and PWA Authentication
Update `pwa-staff/src/lib/supabaseClient.ts` with the local Supabase URL (`http://127.0.0.1:54321`) and anon key (`sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`). Implement actual Supabase authentication in `pwa-staff/src/auth.ts`, `pwa-staff/src/login.ts`, `pwa-staff/src/signup.ts`, and `pwa-staff/src/profile.ts` using the Supabase JS client. Ensure that after sign-up or log-in, users are redirected to `profile.html` where their email is correctly displayed, and log-out redirects them back to `login.html`.

### R2. HTML Asset Paths
Fix relative asset paths in `login.html`, `signup.html`, and `profile.html` so they reference the generated dist directory as `dist/` or `./dist/` instead of `../dist/` since these files are served from the root.

### R3. DB Sync Trigger and Schema Alignment
Create a new migration or update migrations in `supabase/migrations` to sync `auth.users` to the `public.players` table upon user registration. The `public.players` table must support storing the player's email and name/username. Update `pwa-staff/src/admin.ts` to query and display player details from the `players` table. Update `pwa-staff/src/leaderboard.ts` to fetch leaderboard entries joined with the player's username so the leaderboard displays actual player names.

### R4. Edge Function Implementation
Implement the Supabase edge function `supabase/functions/submit-score/index.ts` to parse the request body, validate the score (must be a number between 0 and 1000), and save it into the database (`game_sessions` and update the player's high score in `leaderboard`).

### R5. Local Server & E2E Testing
Configure `local-web-server` or `npm run start` to serve the `pwa-staff` frontend on port 3000. Run the Playwright end-to-end test suite (`tests-e2e/auth.spec.ts`) against the running local server and local Supabase instance to verify the full registration, profile display, login, logout flow passes successfully.

## Acceptance Criteria

### Authentication & UI
- [ ] Sign-up, login, profile view, and logout flows are fully implemented using Supabase client in `pwa-staff/src`.
- [ ] HTML pages properly load CSS and JS bundles without 404 path errors.
- [ ] Profiles display the registered user's actual email from Supabase session.

### Database Sync & Edge Functions
- [ ] Registering a user triggers a Postgres trigger that inserts the user's ID, email, and username/email prefix into the `public.players` table.
- [ ] Admin dashboard successfully fetches and displays registered players list without Postgres column missing errors.
- [ ] Leaderboard page successfully displays player usernames and their high scores.
- [ ] Edge function `submit-score` writes records to `game_sessions` and updates `leaderboard` table.

### E2E Verification
- [ ] Playwright E2E test `user authentication flow` runs successfully and all steps pass.

## Follow-up — 2026-06-15T16:55:03-07:00

Resolve all remaining gaps for production release of the Progressive Web App (PWA) with local Supabase integration and ensure 100% successful end-to-end testing with Playwright.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Resolve Playwright E2E Test Failures and Browser Constraints
- Modify the Playwright configuration in `pwa-staff/playwright.config.ts` to run E2E tests exclusively on the `chromium` browser project (removing firefox and webkit to avoid missing browser binary issues on the host system).
- Fix the test in `pwa-staff/tests-e2e/example.spec.ts` by updating the expected title regex match from `/PWA Staff/` to `/Rocket Craft/` to match the actual application title in `index.html`.

### R2. Verify Application and Test Suite Health
- Verify that Vitest unit tests in the `pwa-staff` workspace run and pass.
- Verify that Playwright E2E tests run and pass without throwing browser configuration errors.

## Acceptance Criteria

### E2E Testing
- [ ] Playwright E2E tests execute and pass successfully on the Chromium browser.
- [ ] No browser launch or executable errors are present in the test logs.
- [ ] The webServer command correctly boots the PWA on port 3000 during test execution.

### Unit Testing
- [ ] Vitest unit tests in `pwa-staff/` pass successfully.

## Follow-up — 2026-06-15T17:31:19-07:00

Upgrade the Rocket Craft PWA launcher with a premium cyberpunk gamer-centric UI/UX, implement a collapsible in-app developer debug HUD (DX/QoL), and add database indexes and telemetry logging to the Supabase schema.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Cyberpunk Gaming UI/UX
- Upgrade all PWA pages (`index.html`, `login.html`, `signup.html`, `profile.html`, `admin.html`, `leaderboard.html`) using vanilla CSS to implement a premium cyberpunk neon dark mode.
- Use glassmorphic card layouts, responsive layouts, glowing neon button hover effects, custom gaming-oriented typography, and subtle micro-animations.

### R2. Collapsible In-App Developer Console HUD
- Implement a collapsible developer debugging HUD available on all frontend pages when toggled (e.g., via a floating debug button in the corner).
- The HUD must display:
  - Active session details (decoded JWT values like email, user ID, role, and expiration timestamp).
  - Quick triggers to test mock score submissions.
  - Database stats fetched from backend endpoints (e.g., number of registered players and total game sessions).

### R3. Database Optimization & Telemetry Schema
- Create a new migration in `supabase/migrations/` that adds database indexes to:
  - `public.players(high_score DESC)`
  - `public.game_sessions(player_id, score)`
- Create a `public.telemetry_logs` table in the migration:
  - Fields: `id` (uuid primary key), `player_id` (foreign key to `players.id`, nullable for unauthenticated events), `event_type` (text, e.g., 'login', 'registration', 'profile_view', 'score_submission'), `payload` (jsonb), and `created_at` (timestamp with timezone).
- Integrate backend client logic to log records into `public.telemetry_logs` whenever a player registers, logs in, views their profile, or submits a score.

### R4. Verification & Testing
- Update Vitest unit tests in `pwa-staff/` to cover new helper functions and console components.
- Update Playwright E2E tests to verify that the Developer Debug HUD is present, can be toggled open, and that new page layouts load without JavaScript console errors.

## Acceptance Criteria

### PWA UI/UX
- [ ] All pages render with the new cyberpunk neon dark theme, including layout grids, forms, tables, and buttons.
- [ ] Responsive UI fits mobile, tablet, and desktop screens with zero overlapping text.

### Developer HUD (DX/QoL)
- [ ] Collapsible debug panel is present on all pages and can be toggled.
- [ ] HUD displays decoded JWT state when user is logged in, and shows an unauthenticated state when logged out.
- [ ] Stats display correct count of registered players.

### Database & Telemetry
- [ ] Supabase schema contains the new indexes on `players` and `game_sessions`, and the `telemetry_logs` table.
- [ ] Performing auth operations (signup, login, logout), profile views, and score submissions creates corresponding rows in `public.telemetry_logs`.

### Test Suite Execution
- [ ] Vitest unit tests in `pwa-staff/` execute and pass successfully.
- [ ] Playwright E2E tests execute and pass successfully on Chromium.

## Follow-up — 2026-06-17T07:06:50Z

Research Google DeepMind's Genie (Generative Interactive Environments / World Model) on arXiv (e.g. arXiv:2402.15391). Implement a core interactive world-model simulator in Rust, and build a Python-based pipeline that integrates TPOT2 for model optimization and DSPy for optimizing interactive player/designer LLM agents.

Working directory: ~/rocket-craft
Integrity mode: development

## Requirements

### R1. DeepMind Genie World Model Core in Rust
Implement a high-performance simulation or reference model of the Genie architecture. The core must include:
1. A spatiotemporal tokenizer representation to discretize state inputs (e.g., grid maps, 2D/3D states, or simple image frames).
2. A Latent Action Model (LAM) that infers actions from transitions between consecutive states.
3. A Dynamics Model that predicts the next discretized state given the current state and a (latent) action.

### R2. Python Integration & TPOT2 AutoML Optimization
Build a Python wrapper or interface to interact with the Rust world model. Use TPOT2 to automate hyperparameter tuning or pipeline optimization for the dynamics predictor or latent action classifier (e.g., optimizing predictive accuracy or convergence rates on a dataset of state trajectories).

### R3. DSPy LLM Player & Designer Agent
Create a DSPy-based interactive agent. Use DSPy to define and compile an optimized LLM prompt/program that:
1. Acts as a player navigating the simulated world, selecting actions based on natural language commands or visual/textual feedback.
2. Acts as a world designer, generating prompt inputs for the world model to spawn new environments.

### R4. Unreal Engine 4 Export & Benchmark
Provide a utility to export simulated world maps or state sequences into a standard JSON-based scene/actor layout format compatible with Unreal Engine 4. Include a benchmark script comparing frame-by-frame generation latency and memory footprint between the Genie world model implementation and traditional engine-like asset loads.

### R5. End-to-End Validation
Provide an automated script (`verify_world_model.sh` or `.py`) that runs the Rust simulator, executes the TPOT2 optimization, tests the DSPy agent, exports a map to UE4 format, and verifies that the entire flow completes without errors.

## Acceptance Criteria

### Core Implementation
- [ ] The Rust core compiles and can load, tokenize, and predict state transitions.
- [ ] The Python bindings/subprocess wrapper can interact with the Rust simulator to get state predictions.

### Pipelines & Optimization
- [ ] The TPOT2 AutoML search runs and successfully finds optimized parameters for the dynamics model.
- [ ] The DSPy interactive agent successfully compiles and executes navigation instructions on the simulated environment using an LLM.

### Unreal Engine 4 Export
- [ ] The system outputs a valid JSON map file representing the generated world layout, compatible with Unreal Engine 4 import.

### Verification
- [ ] Running the verification script completes successfully, demonstrating the entire pipeline runs from end to end.

## Follow-up — 2026-06-17T07:14:14Z

Incorporate the "Genie 26 Vision 2030" philosophy and specification into the World Manufacturing Platform design for the Genie simulator and pipeline in `~/rocket-craft`.

### Genie 26 Vision 2030 Core Principles to Incorporate:
1. **World Manufacturing Philosophy:** Treat every generated system as a "world" that contains:
   * **Objects** (State variables/world elements)
   * **Actors** (Entities interacting within the world)
   * **Relationships** (Structural bounds/hierarchies)
   * **Events** (Transitions/Inputs)
   * **Rules** (Physics/Constraints/Semantic Laws)
   * **Processes** (Workflows/Execution loops)
   * **Receipts** (Lineage, provenance, BLAKE3 receipts/cryptographic lineage, replay records)

2. **Receipted Worlds:** Every world state transition and generation run must support verifiable receipts (cryptographic origin, specification alignment, operational/replay history).

Ensure that the Rust simulation/dynamics model and the Python pipeline reflect these core components, allowing a user to specify a world's objects/rules and manufacture it with verifiable execution and cryptographic receipts.


## Follow-up — 2026-06-17T07:16:57Z

Implement the Genie 26 World Manufacturing Platform based on the Version Vision 2030 PRD and ARD. The platform must manufacture, deploy, operate, and evolve playable Unreal 4 worlds directly from user intent.

Working directory: ~/rocket-craft
Integrity mode: development

## Requirements

### R1. Intent & Specification Layer
Implement a mechanism to ingest user intent (natural language prompts) and output a structured World Specification. The specification must explicitly model the world components:
1. Places (locations and environments)
2. Actors (entities within the world)
3. Objects (items and assets)
4. Relationships (structural hierarchy and bounds)
5. Rules (physics, constraints, and interactions)
6. History/Events (logs of modifications and state transitions)

### R2. World Manufacturing (Unreal 4 Artifacts)
Implement a manufacturing engine that takes the structured World Specification and constructs a playable, navigable Unreal 4 world artifact (e.g., scene layouts, maps, or project setups compatible with Unreal 4).

### R3. World Evolution & State Continuity
Implement an evolution mechanism. When new user intent is received to modify an existing world, the system must update the World Specification and manufacture the updated Unreal 4 artifacts while preserving the existing state, structures, relationships, and history (no starting over from scratch).

### R4. World Deployment & Operation
Provide support to launch/deploy the manufactured world, access/re-enter the world, and log the world's operational and modification history.

### R5. Automated Verification
Provide an automated integration script (`verify_genie.sh`) that takes a test intent prompt, generates a world, applies a modification intent to evolve it, and verifies the generated and evolved Unreal 4 artifacts for structural validity.

## Acceptance Criteria

### Input & Output Validation
- [ ] Natural language intent can be parsed into a structured World Specification modeling Places, Actors, Objects, Relationships, Rules, and History.
- [ ] The manufacturing output is a valid Unreal 4 compatible world map/scene layout file.

### Evolution & Continuity
- [ ] World modifications update the existing map/scene artifacts incrementally, preserving unmodified actors and relationships.

### Verification
- [ ] The `verify_genie.sh` script runs the generation and evolution process on a sample case and exits with code 0 on success.


## Follow-up — 2026-06-17T07:25:43Z

The user has set a /goal to validate the entire Genie system. The goal details are:
"entire genie system is validated by using claude -p to create a world and then interacting with world in browser"

To achieve this goal:
1. **Web Runtime / Browser Deployment:** The manufactured Unreal 4 world artifact must have a browser runtime implementation (e.g., an HTML/JS/WebAssembly frontend, or a web dashboard representing the simulated Unreal 4 world layout) so that the user can open and interact with the manufactured world directly in a web browser.
2. **Interactive CLI / Script:** Ensure there is a CLI/script (like `claude -p` or a direct prompt generation interface) that creates a world, launches a local web server to host it, and prints the URL to open it.
3. **Validation:** The verification suite must demonstrate that the world can be built, hosted, and interacted with via a web interface.

Update the project specification and direct the orchestrator to build a web/browser-based interactive runtime for the manufactured worlds.

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

### R3. Combinatorial Maximalism (Parallel Uncertain uncertainties)
Launch parallel agents against every independent uncertainty. No agent may claim project victory; agents may only claim local receipts. The orchestrator admits only end-to-end receipts.
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

Victory requires a replayable receipt proving the generated world was visible, loaded, responsive, and moved under input.

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

## Stop proving that code exists.

Prove that the world drives.

## Follow-up — 2026-06-17T19:29:01Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Execute the Teamwork Multi-Agent System on the problem constraints.

Complete the full Rocket-Craft ecosystem by closing all remaining feature gaps across the PWA Frontend, Supabase State Persistence, Multiplatform Builds (Windows/Linux), and offline Service Worker caching.

Working directory: ~/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. PWA Frontend & Canvas Integration
The web dashboard (`pwa-staff/`) must seamlessly embed the manufactured HTML5 UE4 output. It must replace any mock visualization with the actual compiled WASM game canvas and render the cryptographic receipt upon completion.

### R2. State Persistence & Authentication
Integrate Supabase to authenticate users and persist generated `WorldSpec` contracts to the database, ensuring world history is tied to specific user accounts.

### R3. Multiplatform Packaging
Extend the headless manufacturing pipeline (currently HTML5-only) to also target Windows (`.exe`) and Linux (`.elf` or `.sh`) standalone builds for dedicated servers or native desktop clients.

### R4. Offline Asset Caching
Implement a Service Worker in the PWA to aggressively cache the heavy Unreal Engine WASM payloads and asset bundles, enabling the world to load offline after the initial play.

## Acceptance Criteria

### PWA & Canvas Integration
- [ ] Playwright E2E script confirms the UE4 `<canvas>` element mounts inside the React/PWA dashboard DOM.
- [ ] The cryptographic receipt data is visually rendered in the UI adjacent to the canvas.

### State Persistence
- [ ] Programmatic script successfully registers a mock user via Supabase Auth.
- [ ] A test script queries the Supabase database and verifies the generated `WorldSpec` JSON was saved under the correct user ID.

### Multiplatform Builds
- [ ] The pipeline successfully emits a Windows `.exe` standalone build for a generated world.
- [ ] The pipeline successfully emits a Linux standalone build for a generated world.

### Offline Caching
- [ ] Playwright disables network connectivity (offline mode) after initial load and confirms the UE4 application still boots successfully from the Service Worker cache.


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

### R5. Eliminate Overclaim/Victory Language
Remove any unverified overclaiming status tags (like `zero violations`, `solved`, `done`) in code comments or logs.

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


## Follow-up — 2026-06-18T02:18:00Z

# Teamwork Project Prompt

Research all the games within the `rocket-craft` repository and build a comprehensive "AutoML"-style abstraction layer. This layer must provide automatic discovery and configuration, game balance optimization, and a massive upgrade to Developer Experience (DX) and Quality of Life (QoL) via automated CLI tooling.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Dynamic Discovery & Auto-Binding
Implement an abstraction layer that automatically detects, configures, and registers new game components, servers, and logic across the codebase to completely eliminate boilerplate wiring.

### R2. Game Balance Auto-Optimizer
Build an "AutoML"-style tuning engine that autonomously runs combinatorial game simulations to dynamically optimize and balance in-game stats, logic, and economies.

### R3. Developer CLI & Environment Tooling
Create a unified suite of command-line tools that automatically scaffolds local development environments and seamlessly manages server lifecycles for all games.

## Acceptance Criteria

### Comprehensive Verification
- [ ] **Auto-Binding**: A programmatic test suite proves that the auto-config layer successfully detects and wires up an unconfigured game component completely without manual intervention.
- [ ] **Optimization**: The balance engine successfully runs a full simulation loop and outputs a mathematically tuned configuration matrix for game stats.
- [ ] **Tooling**: The new Developer CLI successfully and autonomously spins up the complete local dev environment and required servers.
- [ ] **Auditor Review**: An independent secondary agent reviews the implementation to judge that the DX/QoL improvements are satisfactory and that no verification steps were bypassed.

## 2026-06-18T03:49:24Z

# Teamwork Project Prompt

Update all high-level ecosystem documents with recent milestones (Combinatorial Testing, AutoML DX), execute a 10-phase code documentation sweep across Rust, JS, and C to inject exhaustive doctests and moduledocs, and author a complete documentation suite following the Diátaxis framework.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. High-Level Ecosystem Sync
Update `PRESS_RELEASE.md`, `VISION_2030.md`, `CHANGELOG.md`, and other high-level ecosystem documents to accurately reflect the recently integrated Combinatorial Testing and AutoML DX systems.

### R2. Exhaustive 10-Phase Code Documentation
Execute a 10-phase documentation plan across the entire Rust, JS, and C codebase. Document absolutely everything—including internal modules, private functions, and utility scripts—ensuring no code is left unexplained. Inject executable doctests and comprehensive moduledocs into all files.

### R3. Diátaxis Framework Implementation
Author a completely new, structured documentation suite that strictly adheres to the Diátaxis framework. It must clearly separate content into the four quadrants: Tutorials, How-to Guides, Reference, and Explanation.

## Acceptance Criteria

### Comprehensive Verification
- [ ] **Executable Doctests**: The CI pipeline must successfully run `cargo test --doc` (and equivalent JS/C doc-test runners) across the entire codebase with a 100% pass rate.
- [ ] **Diátaxis Compliance Audit**: An independent secondary agent must audit the newly authored documentation suite to verify that all four Diátaxis quadrants exist, are correctly categorized, and are structurally complete.
- [ ] **Ecosystem Sync Validation**: The secondary agent must confirm that the high-level ecosystem documents accurately and consistently reflect the latest system capabilities.

