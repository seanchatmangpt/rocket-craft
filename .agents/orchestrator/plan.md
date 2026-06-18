# Remediation Plan - Rocket-Craft Project

This plan coordinates the resolution of all implementation gaps, stubs, placeholders, single-line functions, assertion shortcuts, debug macros, and overclaiming terms in the Rocket-Craft project, ensuring full compliance and passing verification.

## Milestones

### Milestone 1: Technical Exploration and Baseline Assessment
- **Objective**: Identify all stubs, placeholders, debug macros, overclaiming language, and check baseline compile and test status.
- **Tasks**:
  - Spawn an Explorer to locate target files (`pwa_export.rs`, `fixtures.rs`, `pipeline.rs`, `classify.rs`, `unify-core/src/lib.rs`, `unify-rocket/src/lib.rs`, `manifest.rs`, `wasm-patterns/src/actor.rs`, `pattern_integration.rs`, `unify-integration-tests/src/lib.rs`, `hud.rs`, `message_bridge.rs`, `build.rs`, `unify-mcp/src/main.rs`, `commands.rs`, `unify/src/main.rs`).
  - Read files and collect details on what logic needs to be fully implemented.
  - Run baseline cargo compilation and tests using a Worker.
  - Run the `anti-llm-cheat-lsp` compliance scanner to identify starting violations.
- **Verification**: Exploration report documenting target files, stubs, and initial test/scan status.

### Milestone 2: Complete Stubs and Placeholders (R1)
- **Objective**: Implement complete, production-ready logic for all files containing `STUB` or placeholders.
- **Tasks**:
  - Implement `unify-rs/unify-bp/src/pwa_export.rs`.
  - Implement `unify-rs/unify-integration-tests/src/fixtures.rs`.
  - Implement `unify-rs/unify-rdf/src/pipeline.rs`.
  - Fix any other stubs/placeholders in the workspace.
- **Verification**: Code review and local workspace compilation.

### Milestone 3: Replace Single-Line and Catch-All Stubs (R2)
- **Objective**: Replace single-line stubs and catch-alls with robust implementation logic.
- **Tasks**:
  - Replace description method in `classify.rs` with descriptive logic.
  - Implement `namespace`, `noun`, and `verb` functions in `unify-core/src/lib.rs` and `unify-rocket/src/lib.rs`.
  - Fix `manifest.rs` empty catch-all match arms with proper error handling/logging.
  - Implement `on_start` and `on_stop` in `wasm-patterns/src/actor.rs`.
  - Replace dummy helper functions in `wasm-tests/tests/pattern_integration.rs`.
- **Verification**: Compilation and unit test passes.

### Milestone 4: Harden Assertions & Eliminate Test Shortcuts (R3)
- **Objective**: Replace substring matches and shortcut assertions with schema-compliant structural validation.
- **Tasks**:
  - Refactor assertions in `unify-integration-tests/src/lib.rs`.
  - Refactor HUD display/substring searches in `wasm-ui/tests/hud.rs` and `wasm-ui/tests/message_bridge.rs`.
- **Verification**: Test suite runs and passes.

### Milestone 5: Remove Debug Macros and Overclaim Language (R4, R5)
- **Objective**: Clean up print statements and remove unverified overclaiming status tags.
- **Tasks**:
  - Replace `println!` with structured logging/tracing in `unify-ffi/build.rs`, `unify-mcp/src/main.rs`, `unify/src/commands.rs`, and `unify/src/main.rs`.
  - Remove terms like "zero violations", "solved", "done" from comments/logs if unverified.
- **Verification**: Compliance scanner (`anti-llm-cheat-lsp`) returns 0 errors/violations.

### Milestone 6: Final Verification & Integration
- **Objective**: Full verification of all acceptance criteria.
- **Tasks**:
  - Compile `unify-rs` and `wasm-threads` workspaces without warnings/errors.
  - Run `cargo test --workspace` and ensure 100% pass rate.
  - Run E2E Playwright tests.
- **Verification**: Audit check pass with zero violations, 100% tests pass.
