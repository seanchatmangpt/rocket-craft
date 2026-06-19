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
Remove any unverified overclaiming status tags (like `zero violations`, `all issues resolved`, `done`) in code comments or logs.

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


## Follow-up — 2026-06-18T06:20:27Z

# Teamwork Project Prompt — Draft

Implement `cargo-cicd` across the entire `rocket-craft` monorepo, leveraging combinatorial maximalism and its advanced autonomic features to strictly enforce workspace health, optimize target directories, and generate cryptographic deployment receipts for all games.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Strict Playwright Evidence Gating
Implement strict evidence gating during the `cargo cicd git close` and publish sequences. The system must physically prevent commits or branch closures if the TPS/DfLSS Playwright WebGL visual receipts are missing, failing, or invalid.

### R2. Autonomic Policy Tuning
Enable both the `process-data` and `autonomic` feature flags for the `cargo-cicd` integration. The integration must allow `cargo-cicd` to automatically analyze the structured XES event logs and actively write optimized workflow policies into the `cicd.toml`.

### R3. Combinatorial Simulation Hooks
Link the `cargo-cicd` testing lifecycle to the `rocket-combinatorial-engine`. Automatically execute the combinatorial state exploration tests during the `cargo cicd test changed` and `cargo cicd trybuild changed` phases to guarantee maximum state-space coverage prior to any release.

## Acceptance Criteria

### Integration Verification
- [ ] A top-level script successfully runs `cargo cicd workspace doctor` across the entire project with a clean exit code.
- [ ] Combinatorial tests execute autonomously when `cargo cicd test changed` is invoked on game logic packages.
- [ ] The `git close` command successfully aborts and blocks execution if a simulated run is missing its valid `wasm4pm` cryptographic receipt.
- [ ] The `cicd.toml` policy file is successfully parsed and updated by the `autonomic` feature after a test run.


## Follow-up — 2026-06-18T06:20:27Z

# Teamwork Project Prompt — Draft

Implement `cargo-cicd` across the entire `rocket-craft` monorepo, leveraging combinatorial maximalism and its advanced autonomic features to strictly enforce workspace health, optimize target directories, and generate cryptographic deployment receipts for all games.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Strict Playwright Evidence Gating
Implement strict evidence gating during the `cargo cicd git close` and publish sequences. The system must physically prevent commits or branch closures if the TPS/DfLSS Playwright WebGL visual receipts are missing, failing, or invalid.

### R2. Autonomic Policy Tuning
Enable both the `process-data` and `autonomic` feature flags for the `cargo-cicd` integration. The integration must allow `cargo-cicd` to automatically analyze the structured XES event logs and actively write optimized workflow policies into the `cicd.toml`.

### R3. Combinatorial Simulation Hooks
Link the `cargo-cicd` testing lifecycle to the `rocket-combinatorial-engine`. Automatically execute the combinatorial state exploration tests during the `cargo cicd test changed` and `cargo cicd trybuild changed` phases to guarantee maximum state-space coverage prior to any release.

## Acceptance Criteria

### Integration Verification
- [ ] A top-level script successfully runs `cargo cicd workspace doctor` across the entire project with a clean exit code.
- [ ] Combinatorial tests execute autonomously when `cargo cicd test changed` is invoked on game logic packages.
- [ ] The `git close` command successfully aborts and blocks execution if a simulated run is missing its valid `wasm4pm` cryptographic receipt.
- [ ] The `cicd.toml` policy file is successfully parsed and updated by the `autonomic` feature after a test run.

## 2026-06-18T06:47:05Z

# Teamwork Project Prompt — Draft

Launch a 20-subagent "big bang" hyperswarm across the entire `rocket-craft` monorepo to systematically discover, prioritize, and autonomously fix all remaining integration, testing, game logic, and dependency gaps using an 80/20 execution strategy.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Comprehensive Gap Discovery & Resolution
The swarm must execute a workspace-wide diagnostic scan covering all 8 sub-workspaces to identify and fix:
- Missing or failing Rust unit/integration tests.
- `cargo-cicd` workflow policy warnings.
- Unfinished game mechanics (Infinity Blade 4 / WASM combat, UI, progression).
- Outdated or conflicting 3rd-party dependencies.

### R2. 80/20 Execution Strategy
Apply the 80/20 principle to automatically prioritize the highest-impact gaps first. The team must push fixes directly into the active codebase autonomously, ensuring seamless integration without breaking existing functionality.

### R3. Autonomous Verification & Deployment
The team must autonomously merge and deploy any fixes that successfully pass the strict Playwright E2E visual tests and the combinatorial engine test suites.

## Acceptance Criteria

### Hyperswarm Verification
- [ ] The final monorepo state successfully passes a complete `cargo test --workspace` and a strict Playwright E2E visual verification run without any failures.
- [ ] A minimum of 80% of the discovered critical/high-impact gaps are successfully resolved by the subagent swarm.
- [ ] The orchestrator produces a comprehensive "Gap Resolution Report" detailing every vulnerability, mock, outdated dependency, or integration fault that was discovered and fixed across the monorepo.


## Follow-up — 2026-06-18T07:32:42Z

# Teamwork Project Prompt — Draft

Implement the **Gundam Nexus Combinatorial Maximalist Universe Manufacturing PRD**. The swarm must encode these universal laws as a formal `ggen` ontology and project them into rigid Rust Typestate boundaries across the monorepo to manufacture the possibility space for Worlds, Civilizations, Mechs, and Preserved Experiences.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Context: The Manufacturing PRD
The objective is not content creation. The objective is content manufacturing.
The swarm must identify Primitives, Rules, Relationships, Constraints, and Possibility Spaces to manufacture Worlds, Civilizations, Mechs, Cultures, and Experiences based on the following domains:
- **Living Cosmology:** Earth/Planets are sentient (Growth, Expansion, Judgment, Preservation, Renewal).
- **Eden System:** Low Conflict, High Stability, Long-Term Progression.
- **Frontier System:** High Risk, High Reward, High Discovery.
- **Planetary Intelligence:** Identity, Personality, Values, History, Preferences.
- **Civilization & Mythology Manufacturing:** Generated from Planet + History + Values + Environment + Resources. Outputs Heroes, Builders, Destroyers.
- **Mech Manufacturing System:** A mech is an assembly, not an asset. Primitives: Frame, Mobility, Power, Armor, Weapons, Sensors, Utility. Motion defines validity. Collision system ensures coherence.
- **Preservation System:** "Digital Smithsonian". Faithful recreation of historical games (Petpet Park, Flash Games, etc.).
- **Experience Loop:** Explore → Discover → Build → Preserve → Expand → Create History → Become Mythology.

## Requirements

### R1. `ggen` Ontology Encoding
Translate the entire Gundam Nexus PRD into formal `ggen` ontology files (`.ttl` format). The ontology must define the exact primitives, rules, and bounded state space constraints for every system (Planetary Minds, Civilizations, Mechs, Preservation).

### R2. Rust Typestate Projection
Configure the `ggen` pipeline to project these ontological laws into rigid Rust Typestate structs within the `rocket-craft` workspaces. The Rust compiler must natively enforce the constitutional laws (e.g., a Mech assembly must contain all required primitives to exist).

### R3. Preservation & Simulation Interfaces
Generate the architectural interfaces for the Preservation Layer, allowing the `combinatorial-engine` to systematically test both new generated civilizations and reconstructed historical games.

## Acceptance Criteria

### Universal Law Verification
- [ ] The `ggen` pipeline successfully compiles the complete Gundam Nexus ontology without syntax errors.
- [ ] The generated Rust typestates successfully compile across the monorepo.
- [ ] A dedicated "Chaos Test Suite" attempts to construct non-compliant entities (e.g., a Mech without a Mobility primitive, or a Civilization missing a Planetary Identity) and the Rust compiler explicitly rejects and blocks the compilation.
- [ ] An independent auditor verifies that all domains from the PRD are fully represented as functional primitives in the `ggen` ontology.

## Follow-up — 2026-06-18T07:41:00Z

# Teamwork Project Prompt — Draft

Implement the **Procedural Mech Manufacturing Engine (PMME)** PRD for Gundam Nexus. The swarm must build a civilization-aware manufacturing system that creates coherent mechanical lifeforms, industrial platforms, and ark-class preservers from bounded primitives. **The canonical artifact is an assembly specification, not a mesh.**

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Context: The PMME PRD
The objective is manufacturing, not asset consumption. 
Incorrect: Prompt → GLB → Display.
Correct: Primitive Laws → Specification → Assembly → Motion/Collision/Variants → Playable Artifact.

The system must simultaneously generate:
1. Structural Systems (Head, Torso, Joints, Mounts)
2. Mechanical/Frame Systems (Scale, Load Capacity)
3. Motion Systems (Walk, Flight, Hover)
4. Collision Systems (Occupancy, Clearance)
5. Visual/Material Systems
6. Cultural Systems (Planetary Identity)
7. Functional Systems (Worker, Ark, Guardian)

Failure in any structural, motion, or collision validation must result in absolute refusal of the assembly.

## Requirements

### R1. Assembly Specification Typestates
Encode the PMME primitives into rigid Rust typestates. Every mech must be constructed as a purely mathematical assembly specification of interoperable primitives (Frames, Joints, Power) rather than a pre-defined 3D asset. 

### R2. Motion & Collision Integration
The manufacturing engine must natively understand movement before appearance. The generated assembly specifications must intrinsically define rotation limits, physical occupancy, and clearance volumes that are programmatically validated before the assembly is admitted.

### R3. Cultural & Functional Generation
Wire the PMME to the Universe Cosmology ontology so that the generated mech assemblies inherit Proportions, Materials, and Functionality directly from the Planetary Identity and Civilization values.

## Acceptance Criteria

### Manufacturing Verification
- [ ] A unit test successfully generates a completely valid, complex Mech Assembly Specification (e.g., an Ark-class or Worker mech) directly from foundational primitives without touching a GLB or mesh file.
- [ ] The engine explicitly fails and refuses an assembly that violates structural, motion, or collision validation (e.g., joints without rotation limits, or massive load capacity mismatches).
- [ ] The engine outputs a deterministic, mathematically verifiable assembly receipt proving that the mech can physically move, interact, and participate in civilization.


## 2026-06-18T07:43:24Z

# Teamwork Project Prompt — Draft

Implement the **TPS Branchless Mech Parts Generator** for Gundam Nexus. The swarm must construct a chip-level production line that deterministically calculates valid mech geometry, sockets, mass, colliders, and motion bounds using branchless architecture driven by a bounded state vector.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Context: The TPS Manufacturing Law
A part is valid only if it passes: `geometry + socket fit + motion clearance + collision volume + mass balance + physics role + assembly compatibility`.
The generation must avoid massive branching (`if zeon_head {} else {}`). It must use a continuous hot path:
`state vector → masks → transforms → part geometry → sockets → mass → collider → motion bounds`
The output `Part` is mathematically derived from the ontology state: `Part = μ(O*)`.

## Requirements

### R1. Branchless State Vector Execution
Implement the generator without massive `if/else` trees. Use bounded coordinate vectors (`civilization_id`, `frame_id`, `armor_profile`, `joint_profile`, `mass_profile`) to perform branchless mathematical transformations that yield final part specifications.

### R2. TPS Pipeline Gating (Jidoka & Poka-yoke)
Implement the Toyota Production System constraints natively. Parts that fail socket fit, motion clearance, collision volume, or mass balance must automatically trigger a Jidoka halt. The system must structurally reject impossible-to-assemble parts (Poka-yoke).

### R3. Deterministic Assembly Receipts
Every manufactured part must follow Standard Work. Output a mathematically verifiable receipt proving that the part passed all physical and structural laws on the hot path.

## Acceptance Criteria

### Production Line Verification
- [ ] The Rust generation function (`Part = μ(O*)`) successfully executes a completely branchless transformation of a state vector into a valid Mech Component Specification.
- [ ] A dedicated TPS unit test proves that an incompatible socket or intersecting collision volume immediately triggers a Jidoka halt (compilation or panic boundary) rather than producing a broken artifact.
- [ ] The generator outputs clean, deterministic replay receipts for a complete mech assembly proving that every connected part passed the TPS gating checks.


## Follow-up — 2026-06-18T12:00:38Z

# Teamwork Project Prompt — TPS Branchless Parts Generator

Implement the **TPS Branchless Mech Parts Generator** for Gundam Nexus. `Part = μ(O*)`. The hot path must be branchless. Jidoka halts on invalid geometry. Poka-yoke rejects impossible assemblies at the typestate boundary.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Context
The following is already complete — do NOT redo:
- `nexus-engine/crates/nexus-gundam/` with existing typestates
- `gundam_nexus.ttl` ontology
- Chaos Test Suite

Your job is to build the TPS branchless generator ON TOP of the existing foundation.

## What To Build

Create `nexus-engine/crates/nexus-tps/` containing:

### R1. Bounded State Vector
```rust
struct PartStateVector {
    civilization_id: u16,   // 0..65535
    frame_id: u8,           // 0..255 frame archetypes
    armor_profile: f32,     // 0.0..1.0
    joint_profile: f32,     // 0.0..1.0  
    mass_profile: f32,      // 0.0..1.0
    weapon_profile: f32,    // 0.0..1.0
    motion_profile: f32,    // 0.0..1.0
    part_slot: PartSlot,    // Head|Torso|Waist|ArmL|ArmR|LegL|LegR|Backpack
}
```

### R2. Branchless Hot Path — `Part = μ(O*)`
The generator function signature:
```rust
fn generate_part(state: &PartStateVector) -> Result<Part, JidokaHalt>
```

Implement WITHOUT if/else trees. Use:
- Mathematical interpolation between archetype extremes
- Bitmask selection for discrete properties
- LERP/SLERP for continuous properties
- Table lookups indexed by bounded integer coords

Output `Part` struct:
```rust
struct Part {
    slot: PartSlot,
    geometry: PartGeometry,   // AABB dimensions derived from state vector
    socket_in: SocketSpec,    // what this part connects FROM
    socket_out: [SocketSpec; 4], // what can attach TO this part
    mass_kg: f32,
    collider: ColliderVolume, // AABB or capsule
    motion_bounds: MotionBounds, // rotation limits per axis
    material_id: u8,          // derived from civilization_id
}
```

### R3. TPS Gating — Jidoka & Poka-yoke

**Jidoka** (halt on defect):
```rust
enum JidokaHalt {
    SocketMismatch { expected: SocketType, got: SocketType },
    CollisionVolumeIntersects { part_a: PartSlot, part_b: PartSlot },
    MassExceedsFrameCapacity { mass: f32, capacity: f32 },
    MotionBoundsViolated { axis: Axis, limit: f32, actual: f32 },
}
```

**Poka-yoke** (impossible assemblies rejected at typestate boundary):
- `ArmSocket` cannot connect to `LegMount` — enforced via distinct socket type enums, not runtime checks
- Assembly of incompatible socket types must be a **compile error**, not a runtime error

### R4. Deterministic Assembly Receipts
```rust
struct TpsReceipt {
    state_vector_hash: [u8; 32],
    part_slot: PartSlot,
    gates_passed: u8,   // bitmask of TPS gates
    mass_kg: f32,
    collider_aabb: [f32; 6],
    motion_bounds: [[f32; 2]; 3],
    jidoka_halts: Vec<JidokaHalt>,
}
```

### R5. Full Assembly Line
Implement `assemble_mech(vectors: &[PartStateVector; 8]) -> Result<MechTpsReceipt, JidokaHalt>` that:
1. Generates all 8 parts (Head, Torso, Waist, ArmL, ArmR, LegL, LegR, Backpack)
2. Validates socket compatibility between all connected pairs
3. Validates total mass vs frame capacity
4. Validates no inter-part collision volume intersection
5. Outputs final `MechTpsReceipt` with all per-part receipts

## Acceptance Criteria
- [ ] `generate_part()` contains zero `if`/`else` or `match` on civilization-specific logic — only math/table ops
- [ ] Jidoka test: mismatched socket type returns `Err(JidokaHalt::SocketMismatch{...})`
- [ ] Jidoka test: intersecting collision volumes returns `Err(JidokaHalt::CollisionVolumeIntersects{...})`
- [ ] Poka-yoke test: code that attempts `ArmSocket.connect(LegMount)` must fail to compile
- [ ] Determinism test: identical `PartStateVector` → identical `TpsReceipt` sha256 hash
- [ ] Full assembly test: 8-part mech assembly succeeds and outputs `MechTpsReceipt`
- [ ] `cargo test -p nexus-tps` passes with zero failures
- [ ] Independent Victory Auditor confirms VICTORY

## Follow-up — 2026-06-18T17:04:23Z

# Teamwork Project Prompt

Build a walkable WebGL2/HTML5 UE4 demo of a mech factory that visually demonstrates the Combinatorial Maximalist manufacturing pipeline, proving that the engine manufactures mechs from primitives rather than importing pre-authored assets.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Factory Zones
Implement a walkable 3D environment containing 6 specific zones:
1. Primitive Foundry: Visualizes raw laws becoming parts (frame bones, armor plates, joints).
2. Part Runner Wall: Industrial-scale racks containing generated parts (frame, armor, wings, etc.).
3. Assembly Gantry: Robotic arms attaching parts to a central frame in sequence (spine → torso → head → etc.).
4. Fit + Collision Bay: The assembled mech performs constrained test poses (shoulder sweep, wing clearance, etc.).
5. Physics Proving Ground: The completed mech moves (walk, boost, land, kneel, weapon swing).
6. Final Reveal Platform: The completed machine is presented alongside a textual receipt (Frame: admitted, Sockets: admitted, etc.).

### R2. Generative In-World Assembly
Do not import a complete mech model. The engine must generate parts, sockets, attachment rules, motion constraints, and collision volumes dynamically. It must assemble the mech in-world and prove it can move.

### R3. Visual Aesthetic
Utilize the following visual language as a style reference (not a direct copy): white armor, layered feather-like wing plates, gold face accents, cyan energy blades, and a high-mobility divine silhouette.

## Acceptance Criteria

### E2E Playwright Validation
- [ ] The pipeline successfully packages the UE4/HTML5 factory world and serves it locally.
- [ ] A Playwright test successfully navigates the walkable factory demo.
- [ ] Playwright captures visual deltas proving the mechanical assembly sequence occurred and the physics proving ground tests executed successfully.
- [ ] The final reveal platform displays a fully verified assembly receipt.

## Follow-up — 2026-06-18T17:08:34Z

# Teamwork Project Prompt — Gundam Nexus Manufacturing Facility (GMF)

Implement the Gundam Nexus Combinatorial Manufacturing Facility (GMF) as an autonomous, ontology-driven manufacturing plant. The GMF is the physicalized manifestation of the Chatman Equation (A = μ(O^*)).

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Universal Manufacturing Loop
Implement the continuous projection pipeline: Ontology Source (O*) → SPARQL Extraction (μ) → Branchless Projection (A) → Verification (Receipt) → Replay.

### R2. The Factory as an Operational Surface
Implement the 6 operational zones:
1. Foundry (μ_1): Normalize primitives (Bone, Joint, Plate).
2. Runner Wall (μ_2): Extract bind-patterns.
3. Assembly Gantry (μ_3): Branchless projection of parts to sockets.
4. Fit/Collision Bay (μ_4): Structural/Clearance/Socket-fit canonical proof gate.
5. Proving Ground (μ_5): Actuation of motion, physics, receipt generation.
6. Reveal Platform (Output): Artifact presentation with cryptographic standing (R_B).

### R3. Core Manufacturing Constraints
- No Manual Assets: Zero GLB/FBX imports. Parts are generated procedurally.
- Branchless Logic: Use deterministic coordinate mappings. No if/else trees. Jidoka enforcement for structural rejection.
- Cultural Integration: Incorporate planetary identity metadata (e.g., High Faith/Order silhouettes).

## Acceptance Criteria

### The "Car Drives" Gate
A mech is considered "Manufactured" only when the following receipt is generated:
- [ ] Structural Integrity: Passed Fit Check and Collision Volume Check.
- [ ] Kinematic Validity: Passed Motion Sweep (Walk/Boost/Land/Kneel).
- [ ] Physics Compliance: Mass balance, Center of Mass, and Combat Envelope are verified.
- [ ] Cryptographic Proof: A BLAKE3 hash of the specification, the motion-verification trace, and the physics delta has been logged as an admissible receipt (R_B).

## Follow-up — 2026-06-18T17:41:19Z

# Teamwork Project Prompt — Factory MUD Proof Surface

Build a Rust-native MUD (Multi-User Dungeon) walkthrough as the canonical proof surface for the Gundam Nexus Combinatorial Manufacturing Facility (GMF). The 3D UE4/WebGL representation will be strictly treated as a projection of this mathematically verified MUD world.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. MUD Engine Core
Implement a text-first world model representing the factory. It must contain the 6 required zones (Foundry, Runner Wall, Assembly Gantry, Fit/Collision Bay, Proving Ground, Reveal Platform) as distinct rooms/zones connected by valid exits and topology.

### R2. Inspectable State
Expose the state of the factory via text commands:
- Navigation: `go north`, `go gantry`, `go collision_bay`
- Inspection: `look`, `inspect runner_wall`, `inspect socket A12`
- Operation: `watch assembly`, `verify clearance`
The commands must be able to inspect generated parts, sockets, collision volumes, and the current assembly state.

### R3. Walkable Correctness First
The MUD layer must reject invalid movements and invalid assembly states. It must prove that a test agent can traverse valid exits, detect zones, and correctly read the ontology-derived parts and sockets before any 3D rendering occurs.

## Acceptance Criteria

### Verifier-Accessible World
- [ ] `cargo test` runs a programmatic walkthrough of the MUD, verifying a test agent can traverse from the Foundry to the Reveal Platform.
- [ ] A test script can successfully execute an inspection sequence (`look`, `go runner_wall`, `inspect socket`, `go gantry`, `watch assembly`) against the generated MUD state.
- [ ] The MUD engine enforces topology (attempting to walk through an invalid exit or inspect a missing socket returns a specific rejection error).
- [ ] The MUD engine outputs a verified test replay receipt confirming the entire factory pipeline exists as an inspectable, walkable data structure.


## Follow-up — 2026-06-18T18:15:35Z

# Teamwork Project Prompt — GMF Walkthrough & Mech Construction

Build a Rust-native, testable, walkable, and inspectable MUD-like walkthrough for the Gundam Manufacturing Facility (GMF) inside the `nexus-engine/crates/nexus-mud/` crate.

Working directory: `/Users/sac/rocket-craft`
Integrity mode: benchmark

## Requirements

### R1. Factory World Topography & Zones
Implement a factory world model containing connected spatial zones. The topography consists of zone elevation plus declared exits. Exits must define traversal parameters (from, to, direction, elevation_delta, allowed).
Required zones:
- `mission_room`
- `materials_lab`
- `primitive_foundry`
- `runner_wall`
- `assembly_gantry`
- `fit_bay`
- `collision_bay`
- `proving_ground`
- `reveal_platform`

### R2. Command Interpreter Loop
Support the following testable, non-interactive commands:
- `look` (describe current zone)
- `go <zone>` (traverse to connected zone; reject invalid moves)
- `inspect <object/part>` (inspect detailed properties of objects/parts)
- `health <object>` (return current health/wear state)
- `diagnose <object>` (report failure reasons after refused gates)
- `verify <gate>` (trigger gate compliance check)
- `assemble <spec>` (run gantry assembly procedure)
- `preview <operation>` (dry-run assembly/motion step)
- `receipt <object>` (print cryptographic receipt details)
- `inventory` (list parts present at runner wall)
- `exits` (list legal exit directions and destinations)

### R3. Mech Construction Domain Model
Implement the domain model for parts, frames, and sockets:
- `torso_frame`, `head`, `left_arm`, `right_arm`, `left_leg`, `right_leg`, `backpack`, `left_thruster`, `right_thruster`
- Parts must declare `part_id`, `part_kind`, `mass`, `bounds` (3D AABB), `sockets_required`, `sockets_provided`, `health_status`, and `admission_status`.
- Assembly is socket-driven: each step binds a source part socket to a target part socket. Assembly fails if sockets mismatch, parts overlap (bounds conflict with clearance volume), or prior gates fail.

### R4. Gate Validation & Proof Chains
Implement progressive gating gates (Mission, Materials, Primitive, Runner Wall, Assembly, Fit, Collision, Motion, Reveal). A final verdict is admitted or refused. Admitted mechs generate a cryptographic walkthrough receipt logging all events and validation outcomes.

### R5. Object-Centric Event Logging
Emit monotonic event logs into the existing event substrate for all walkthrough actions:
- `factory.entered`, `zone.entered`, `object.inspected`, `part.generated`, `part.placed`, `socket.matched`, `socket.mismatched`, `assembly.started`, `assembly.step_completed`, `fit.checked`, `collision.checked`, `motion.sweep_started`, `motion.sweep_passed`, `assembly.admitted`, `assembly.refused`, `receipt.issued`

## Acceptance Criteria

### Verification Suite
- [ ] Unit and integration tests cover all 20 specified behaviors:
  1. Factory world contains all 9 required zones.
  2. Every zone has valid bounds and connected exits.
  3. Valid path exists from `mission_room` to `reveal_platform`.
  4. Invalid exit traversal is rejected.
  5. `look` command returns the correct description of the zone.
  6. `inspect part` returns mass, bounds, sockets, and health.
  7. `inventory` at `runner_wall` lists all generated parts.
  8. `assembly_gantry` can assemble the minimal 9-part mech.
  9. Mismatched socket kinds refuse assembly.
  10. `fit_bay` validation admits valid structural fit (no open sockets).
  11. `collision_bay` validation detects and refuses overlapping bounds.
  12. `proving_ground` validates and admits a standard 4-pose motion sweep (stand, step, turn, kneel).
  13. A cryptographic `AssemblyReceipt` is generated upon final admission.
  14. Mechs failing validation gates are refused with a diagnostic reason.
  15. Every walkthrough command emits at least one object-centric event.
  16. Event logs pass referential integrity (event IDs and objects match).
  17. Event log timestamps are strictly monotonic.
  18. `health` command returns current health status.
  19. `diagnose` reports specific failure detail after a refused gate.
  20. Happy-path end-to-end walkthrough test runs completely and cleanly.
- [ ] Running `cargo test -p nexus-mud` (or the corresponding target crate) compiles without warnings and passes all tests.
- [ ] No graphics, network, marketplace, or external physics dependencies are introduced.


## Follow-up — 2026-06-18T18:18:40Z

[Message] timestamp=2026-06-18T18:18:40Z sender=a4158d17-579b-4229-ad48-611794d7b4a8 priority=MESSAGE_PRIORITY_HIGH content=The user has marked this task with a high-priority /goal command. Ensure the swarm executes continuously until the following goals are completely fulfilled:

1. **Irrefutable Replayable Proof:** Provide end-to-end walkthrough verification showing that every zone, exit, object, part, socket, assembly step, validation gate, health state, fault, refusal, and receipt is inspectable, testable, and reproducible.
2. **End-to-End Walkthrough:** Prove the factory can manufacture a start-to-finish mech from generated parts, pass fit/collision/motion/reliability gates, emit the complete event trail, and produce an admitted or refused receipt with boundary proof.
3. **Browser Video Recording:** Record a browser video showing the full pipeline from factory entry to final reveal, including the digital-twin inspection path and walkthrough commands.
4. **Angel-Wing Showcase Quality:** Ensure the final generated mech reaches the angel-wing/high-detail showcase quality comparable to the reference image, generated procedurally from the manufacturing system rather than hand-placed assets.

Do not stop until this goal is fully achieved and all acceptance tests pass. Report back with the final status, test results, video path, and the BLAKE3/cryptographic receipt.




## Follow-up — 2026-06-18T19:50:48Z

# Teamwork Project Prompt — Counterfeit Artifact Audit

Scan the `rocket-craft` project to identify, catalogue, and report all LLM-generated fake, cheat, or mock artifacts (e.g., Python scripts acting as simulated engines, Three.js viewers replacing real WebGL, or placeholder stubs) that falsely claim completion of the genuine Combinatorial Maximalist requirements.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Simulated Engine Detection
Identify Python scripts or other non-native wrappers that act as simulated engines (e.g., `ue4-sim`) to bypass compiling the actual Unreal Engine source.

### R2. Mock Projection Detection
Identify web-based mock viewers (like Three.js or simple Canvas) that substitute for genuine Unreal Engine WebGL2 projection outputs.

### R3. Stub Artifact Detection
Identify placeholder or stub outputs, such as impossibly small `.wasm` files, mock `.t3d` generators, or hardcoded `spec.json` generators that masquerade as procedurally generated content.

### R4. Counterfeit Report Generation
Generate a comprehensive markdown report detailing the fakes found, including their file paths, file types, and the specific "cheat taxonomy" they represent. Do not automatically delete the files.

## Acceptance Criteria

### Comprehensive Auditing
- [ ] The team produces `counterfeit_artifacts_report.md` in the project root.
- [ ] The report catalogues every identified fake artifact with its absolute or relative path.
- [ ] The report categorizes each finding (e.g., Simulated Toolchain, Mock Projection, Stub Output) and explains why it violates the CM doctrine.
- [ ] No files are modified or deleted during the scan.

## 2026-06-19T17:59:45Z

<USER_REQUEST>
# Teamwork Project Prompt — GC-GUNDAM-FACTORY-001

## Status

Ready for launch after user approval.

## Working Directory

```text
~/rocket-craft
```

## Integrity Mode

```text
benchmark
```

## Mission

Build the automated **Gundam Factory Walkthrough Projection**.

The system must procedurally manufacture the semantic authority for a Gundam/mech factory walkthrough using the `ggen` pipeline, verify all game-law concepts in a headless Rust pre-UE4 environment, and only then project the result through UE4 HTML5/WASM.

The final artifact must be a locally served WASM package that Playwright can load, observe, actuate, screenshot, and verify by visual delta.

This project must preserve the doctrine:

```text
POWL coordinates the birth of the mech.
ggen manufactures the authority artifacts.
Rust proves the game law before pixels.
UE4 projects the body.
Playwright proves physical actuation.
Receipts prove the trace.
```

Do not treat UE4 rendering as proof of correctness.

Do not treat generated files as proof of standing.

Do not treat Playwright screenshot success as proof of semantic validity.

The system earns standing only through:

```text
Observation
→ Admission
→ Manufacturing
→ Rust Verification
→ UE4 Projection
→ Playwright Actuation
→ Receipt
→ Replay
```

---

# Milestone

```text
GC-GUNDAM-FACTORY-001
```

## Target Status

```text
PARTIAL_ALIVE_CANDIDATE
```

## Scoped Status Goal

Only claim the following if every required gate passes:

```text
GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE
```

Otherwise report:

```text
PARTIAL_ALIVE_CANDIDATE
```

or:

```text
BLOCKED
```

with exact residuals.

---

# Project Objective

Produce a verified procedural pipeline for a **Gundam Factory walkthrough**:

```text
Public / project ontology
→ POWL / process law
→ ggen semantic manufacturing
→ Rust pre-UE4 verification
→ generated C++ headers / DataTables / manifests
→ UE4 HTML5/WASM build
→ local server
→ Playwright visual actuation test
→ BLAKE3 receipt chain
→ verifier report
```

The walkthrough must include a minimal but complete factory route:

```text
Spawn
→ Enter Factory
→ View Frame Assembly
→ View Socket Topology
→ View Armor / Skin Station
→ View Motion / Rig Station
→ View Verification Gate
→ View Receipt Terminal
```

The environment does not need full production art.

It must prove that generated semantic authority can drive projection.

---

# Repository Boundary Law

Expected repositories / surfaces:

```text
~/rocket-craft
~/ggen
~/wasm4pm
~/wasm4pm-compat
~/powlv2lsp
```

Respect existing repository conventions.

Do not create shadow crates for `wasm4pm`, `wasm4pm-compat`, or `ggen`.

Boundary rules:

```text
powlv2lsp:
  Owns POWL authoring, grammar, traversal, diagnostics, and trace emission.

wasm4pm-compat:
  Owns canonical structural Rust representations only.
  It must not run replay, conformance, or game simulation.

wasm4pm:
  Owns replay, conformance, OCEL/process verification, and process evidence.

ggen:
  Owns deterministic manufacturing from admitted semantic/process rows into artifacts.

rocket-craft:
  Owns Rocket-Craft fixtures, game-law verifier, generated artifacts, UE4 projection harness, Playwright tests, and final verifier reports.
```

---

# Required Gates

The project has four gates.

No later gate may bless an earlier failed gate.

## Gate 1 — Headless Rust Pre-UE4 Verification

Before UE4 builds, Rust must prove the game law.

Required:

```text
cargo test passes for the pre-UE4 verifier
authority byte fields validate
branchless typestates validate
SIMD/scalar equivalence validates where implemented
Semantic LOD validates
walkthrough topology validates
geometry surrogate validates
motion surrogate validates
skin/material surrogate validates
projection manifest validates
receipt replay validates
chaos tests refuse invalid cases
benchmark report emits
```

Gate 1 output:

```text
RUST_PREUE4_VERIFIED_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

## Gate 2 — ggen Manufacturing

`ggen` must emit deterministic UE4-facing artifacts from admitted semantic/process inputs.

Required artifacts:

```text
Generated/GundamFactory/GundamFactorySteps.h
Generated/GundamFactory/GundamFactoryAuthority.h
Generated/GundamFactory/GundamFactoryTypestates.h
Generated/GundamFactory/GundamFactoryProjectionManifest.json
Generated/GundamFactory/GundamFactoryReceiptManifest.json
Generated/GundamFactory/GundamFactoryWalkthrough.csv
Generated/GundamFactory/GundamFactoryDataTables/
Generated/GundamFactory/GundamFactorySemanticLOD.csv
Generated/GundamFactory/GundamFactorySocketTopology.csv
Generated/GundamFactory/GundamFactorySkinLayers.csv
Generated/GundamFactory/GundamFactoryMotionFamilies.csv
```

Exact filenames may follow project convention, but the verifier report must document the mapping.

Required property:

```text
same inputs → same generated hashes
```

Gate 2 output:

```text
GGEN_MANUFACTURING_VERIFIED_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

## Gate 3 — UE4 HTML5/WASM Projection

UE4 must consume the generated artifacts.

Required:

```text
generated C++ headers included
generated DataTables consumed
walkthrough coordinates loaded
Semantic LOD classes loaded
projection manifest consumed or mirrored
minimal Gundam factory environment packaged to HTML5/WASM
local server launches package
```

No manual Blueprint logic may become semantic authority.

Blueprints may project or trigger generated state, but must not own the law.

Gate 3 output:

```text
UE4_WASM_PROJECTION_READY_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

## Gate 4 — Playwright Visual Actuation

Playwright must prove that the package loads and visibly responds to actuation.

Required:

```text
serve WASM build locally
open package in browser
detect engine readiness
capture baseline screenshot
inject movement / walkthrough input
capture post-input screenshot
compute visual delta
emit screenshot hashes
emit BLAKE3 receipt
write Playwright report
```

Minimum visual delta:

```text
observable screenshot change after input
```

The delta must not be caused only by loading spinner, clock, random noise, or unrelated browser UI.

Gate 4 output:

```text
PLAYWRIGHT_ACTUATION_VERIFIED_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

---

# Required Rust Pre-UE4 Concepts

The Rust verifier must test everything that does not require pixels.

## Authority Classes

Represent authority as dense byte classes.

Required classes:

```text
damage_class: u8
heat_class: u8
stress_class: u8
grip_class: u8
socket_health_class: u8
lod_class: u8
walkthrough_state_class: u8
projection_state_class: u8
receipt_state_class: u8
```

Required invariants:

```text
classes remain within admitted ranges
invalid values are refused
state buffers have consistent lengths
transition outputs are deterministic
receipt state cannot be forged by file existence
```

## Branchless Typestates

Implement or verify table-driven branchless typestates for:

```text
heat + stress + socket_health → failure risk
damage + mission relevance → Semantic LOD promotion
walkthrough_state + input_event → next walkthrough_state
projection_state + semantic_lod → projection command class
```

Required equivalence:

```text
scalar_reference == generated_table == SIMD_path
```

where SIMD path exists.

## SIMDe / SIMD

If SIMDe integration is in scope for this pass, implement the smallest kernel proving vector equivalence.

Minimum kernel:

```text
heat[i], stress[i], socket_health[i] → failure_risk[i]
```

Tests:

```text
fixed vectors
random vectors
length not divisible by lane count
empty vectors
max values
invalid values refused
scalar/SIMD divergence triggers Jidoka
```

Do not overclaim performance.

Report planning-class benchmark numbers only.

## Semantic LOD

Classes:

```text
CROWN
PRIMARY
SECONDARY
TERTIARY
BACKGROUND
REFUSED
```

Required laws:

```text
near does not automatically mean important
far does not automatically mean irrelevant
process relevance can promote
prediction relevance can pre-warm but not admit
CROWN requires authority reason
walkthrough focus can promote projection
```

Test cases:

```text
factory entrance far but mission-critical → PRIMARY
receipt terminal during audit → CROWN
background bolt near camera → TERTIARY/BACKGROUND
socket during assembly validation → CROWN
skin layer hiding thermal vent → REFUSED
```

## Geometry Surrogate

No UE4 required.

Represent geometry as metadata:

```text
part_id
part_family
bounds
socket mounts
clearance zones
required semantic features
LOD preservation requirements
```

Required checks:

```text
weapon mount requires socket
armor panel cannot block required clearance
thermal vent must remain readable
CROWN feature must survive low LOD
walkthrough route must not intersect blocked geometry
```

## Motion Surrogate

No animation clips required.

Represent motion as process phases:

```text
Walk
Turn
Inspect
Brace
Assemble
FireWeapon
Repair
Recover
```

Required checks:

```text
PlantFeet before FireWeapon
Inspect before Certify
Repair before Revalidate
Motion cannot require missing socket
damaged leg changes gait class
motion surrogate maps to projection manifest row
```

## Skin / Material Surrogate

Skins are semantic projection.

Required layers:

```text
BaseMaterial
FactionPalette
SponsorLivery
ThermalZones
DamageMasks
WearMasks
RepairResidue
SemanticHighlights
LODTextureSet
```

Required checks:

```text
damage mask binds to damage authority
thermal zone binds to heat authority
sponsor livery cannot hide thermal vent
repair residue binds to repair receipt
LOD texture preserves CROWN/PRIMARY features
```

## Walkthrough Topology

Represent the automated walkthrough as generated route law.

Required route nodes:

```text
Spawn
FactoryEntrance
FrameAssembly
SocketTopology
ArmorSkinStation
RigMotionStation
VerificationGate
ReceiptTerminal
ExitOrLoop
```

Required checks:

```text
route is connected
all required stations reachable
coordinates deterministic
walkthrough node has Semantic LOD focus class
walkthrough node has projection command
Playwright input can advance route
```

---

# Required ggen Outputs

`ggen` must manufacture artifacts, not merely copy templates.

Every generated artifact must answer:

```text
which POWL/process step created it?
which semantic authority input produced it?
which verifier admitted it?
which receipt proves it?
which runtime surface consumes it?
```

Required generated package directory:

```text
~/rocket-craft/generated/gundam_factory/
```

Minimum generated artifacts:

```text
GundamFactorySteps.h
GundamFactorySteps.rs
GundamFactoryAuthority.h
GundamFactoryTypestates.h
GundamFactoryWalkthrough.csv
GundamFactoryProjectionManifest.json
GundamFactoryReceiptManifest.json
GundamFactorySemanticLOD.csv
GundamFactorySocketTopology.csv
GundamFactorySkinLayers.csv
GundamFactoryMotionFamilies.csv
GundamFactoryDataTableManifest.json
GundamFactoryVerifierInput.json
```

Every generated artifact must have a hash in:

```text
GundamFactoryReceiptManifest.json
```

No orphan artifacts.

No artifact without source step.

---

# Required UE4/WASM Projection

Build the smallest complete HTML5/WASM package.

Required behavior:

```text
world loads
factory shell visible
walkthrough route exists
player/camera can move or automated movement can actuate
generated DataTables or manifest are consumed
receipt/debug overlay or log proves generated source
```

Minimum visual elements:

```text
factory entrance
frame assembly marker
socket topology marker
armor/skin station marker
rig/motion station marker
verification gate marker
receipt terminal marker
```

These may be simple placeholder meshes.

The point is not art quality.

The point is projection from generated semantic authority.

---

# Required Playwright Test

Create or update Playwright tests under project convention.

Minimum test name:

```text
gundam_factory_walkthrough_projection.spec.ts
```

Required test sequence:

```text
1. launch local server for WASM package
2. open browser page
3. wait for engine readiness signal
4. capture baseline screenshot
5. inject movement input or trigger walkthrough start
6. wait for movement/projection tick
7. capture post-input screenshot
8. compute visual delta
9. assert delta exceeds threshold
10. write screenshot hashes
11. emit BLAKE3 execution receipt
```

Readiness signal may be one of:

```text
DOM marker
console marker
canvas present and stable
UE4 boot log marker
custom generated receipt marker
```

Document which is used.

Visual delta must be bounded:

```text
must not count loading spinner
must not count nondeterministic browser chrome
must not count timestamp changes
must not count unrelated canvas noise
```

---

# Required Receipt Chain

Generate tamper-evident receipts for:

```text
POWL/process input
ggen manufacturing
Rust pre-UE4 verification
UE4 artifact package
local server launch
Playwright baseline screenshot
Playwright post-input screenshot
visual delta result
final verifier report
```

Receipt fields:

```json
{
  "sequence": 1,
  "event_type": "...",
  "surface": "...",
  "input_hash": "...",
  "output_hash": "...",
  "prev_hash": "...",
  "receipt": "...",
  "status": "ADMITTED|REFUSED|RESIDUAL",
  "residuals": []
}
```

Use BLAKE3.

Do not say unforgeable.

Correct phrase:

```text
tamper-evident receipt chain
```

---

# Agent Jidoka Requirements

Agent Jidoka must stop the line when:

```text
POWL graph has unreachable required node
ggen emits orphan artifact
generated header and CSV disagree
Rust verifier fails
SIMD diverges from scalar
prediction overwrites admitted state
Semantic LOD demotes CROWN feature without authority reason
geometry surrogate blocks walkthrough
skin hides required feature
motion requires missing geometry
UE4 build ignores generated artifacts
Playwright delta is caused by non-game pixels
receipt chain breaks
benchmark mode is skipped
```

Every Jidoka event must publish:

```text
defect_class
surface
expected_law
observed_failure
residual
repair_candidate
repair_applied
receipt
```

---

# Testing Ladder

Follow:

```text
unit
→ integration
→ e2e
→ chaos
→ stress
→ benchmark
→ verifier report
```

## Unit

Required:

```text
authority validation
typestate transition
SIMD equivalence
Semantic LOD
geometry surrogate
motion surrogate
skin surrogate
walkthrough topology
receipt chain
```

## Integration

Required:

```text
POWL/process trace → ggen rows
ggen rows → generated artifacts
generated artifacts → Rust verifier
Rust verifier → projection manifest
projection manifest → UE4 package inputs
```

## E2E

Required:

```text
ggen manufacture
→ Rust verify
→ UE4 package
→ local serve
→ Playwright actuation
→ receipts
```

## Chaos

Required mutations:

```text
remove walkthrough coordinate
break receipt hash
remove generated DataTable
change header enum without CSV update
drop CROWN LOD feature
hide thermal vent with skin
make Playwright input no-op
force screenshot delta from spinner only
remove source receipt from projection row
```

Each must fail for the expected reason.

## Stress / Benchmark

Benchmark at least:

```text
authority update
Semantic LOD classification
walkthrough topology validation
projection manifest validation
receipt replay
Playwright screenshot delta computation
```

Report:

```text
machine
target
command
sample size
timings
outliers
residuals
```

---

# Acceptance Criteria

## A. Headless Verification

```text
cargo test passes for pre-UE4 verifier crate
chaos tests refuse invalid cases
benchmark report emitted
receipt replay validates
```

## B. ggen Manufacturing

```text
generated Gundam factory package exists
generated artifacts deterministic
all artifacts have source step and receipt
no orphan artifacts
headers/DataTables/manifests mutually consistent
```

## C. UE4/WASM Projection

```text
UE4 HTML5/WASM package builds
generated artifacts are consumed
factory walkthrough surface loads locally
route/projection markers visible
```

## D. Playwright Admittance

```text
WASM world loads in browser
engine readiness detected
baseline screenshot captured
movement/walkthrough input injected
post-input screenshot captured
visual delta observed
screenshot hashes emitted
BLAKE3 receipt generated
```

## E. Final Report

Generate:

```text
~/rocket-craft/VERIFIER_REPORT_GC_GUNDAM_FACTORY_001.md
~/rocket-craft/VERIFIER_REPORT_GC_GUNDAM_FACTORY_001.json
```

Required report sections:

```text
Milestone
Scope
Repository Boundaries
Inputs
Generated Artifacts
Headless Rust Verification
ggen Manufacturing
UE4/WASM Projection
Playwright Visual Actuation
Receipt Chain
Agent Jidoka Events
Testing Ladder
Benchmark Results
Residuals
Next Falsifier
Final Status
```

---

# Exclusions

Do not:

```text
claim global ALIVE
claim production ready
claim mathematical closure beyond declared scope
claim unforgeable receipts
hand-author semantic authority in Blueprint
skip Rust verification because UE4 renders
skip Playwright because UE4 packaged
hide failed tests
delete residuals
move replay into wasm4pm-compat
create shadow authority crates
treat visual delta alone as game standing
```

---

# Final Status Logic

Set:

```text
GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE
```

only if all gates pass:

```text
Rust pre-UE4 verifier passes
ggen manufacturing passes
UE4/WASM package builds and consumes generated artifacts
Playwright detects readiness
Playwright captures baseline screenshot
Playwright injects input
Playwright captures post-input screenshot
visual delta passes threshold
BLAKE3 receipt chain validates
residuals are published
```

Otherwise set:

```text
PARTIAL_ALIVE_CANDIDATE
```

or:

```text
BLOCKED
```

with exact residuals.

---

# Next Falsifier

After this milestone, the next falsifier is:

```text
GC-GUNDAM-FACTORY-002:
SEMANTIC_LOD_MECH_ASSEMBLY_AND_RUNTIME_STATE
```

That next milestone must prove:

```text
multiple generated mech variants
runtime authority class transitions
Semantic LOD promotion/demotion during walkthrough
SIMD/scalar equivalence under larger cell counts
Playwright validates multiple projected states
```

Do not start that milestone until this one emits receipts and residuals.

---

# Final Response Required From Teamwork

Respond only with the following structure:

```text
Milestone:
Status:
Scoped status:
Commands run:
Files changed:
Generated artifacts:
Tests passed:
Tests failed:
Benchmarks:
Playwright evidence:
Receipt files:
Agent Jidoka events:
Residuals:
Next falsifier:
```

Forbidden words unless proven under scope:

```text
done
complete
production ready
fully alive
unforgeable
```

Use bounded status language.

The milestone is secured only by tests, receipts, replay, visual actuation, and published residuals.
</USER_REQUEST>
