# Project: Rocket-Craft Remediation

## Architecture
This project focuses on the remediation of code quality, correctness, and compliance issues across two main workspaces:
- **`unify-rs`**: The core framework containing multiple crates (`unify-bp`, `unify-integration-tests`, `unify-rdf`, `unify-core`, `unify-rocket`, `unify-ffi`, `unify-mcp`, `unify`, etc.).
- **`wasm-threads`**: The WASM runtime/logic layer containing crates like `wasm-patterns`, `wasm-tests`, `wasm-ui`, etc.

The target is to replace all stubs/placeholders with robust implementations, remove debug macros/logs, harden tests to use schema-compliant structural validations instead of substring/text-based checks, and eliminate overclaiming terms.

## Milestones
| # | Name | Scope | Dependencies | Status | Conv ID |
|---|------|-------|-------------|--------|---------|
| 1 | Technical Exploration | Identify all stubs, placeholders, print statements, and overclaim terms | None | DONE | df135978-48b4-4519-9ebf-9d88830e5ec0 |
| 2 | Complete Stubs (R1) | Implement complete logic for `pwa_export.rs`, `fixtures.rs`, `pipeline.rs`, etc. | M1 | IN_PROGRESS | 6469f9db-14b9-489b-92a0-cae939dc947e |
| 3 | Replace Single-Line (R2) | Implement single-line stubs, catch-alls, lifecycle `on_start/on_stop`, and helpers | M2 | IN_PROGRESS | 6469f9db-14b9-489b-92a0-cae939dc947e |
| 4 | Harden Assertions (R3) | Refactor substring searches in tests to schema-compliant structural validations | M3 | IN_PROGRESS | 6469f9db-14b9-489b-92a0-cae939dc947e |
| 5 | Clean Logs & Overclaims (R4/R5) | Replace `println!` with tracing/logging and remove overclaims | M4 | IN_PROGRESS | 6469f9db-14b9-489b-92a0-cae939dc947e |
| 6 | E2E & Compliance Verify | Verify 100% tests pass, `anti-llm-cheat-lsp` has 0 violations, E2E Playwright passes | M5 | PLANNED | |

## Interface Contracts
- **PWA Export**: Fully functional PWA packaging and exporting.
- **RDF Pipeline**: Robust RDF processing pipeline.
- **WASM Actor Lifecycle**: Proper state management in `on_start` and `on_stop`.
- **Structured Assertions**: Testing methods validating precise types/structures rather than raw print/match logs.

## Code Layout
- `unify-rs/`
  - `unify-bp/src/pwa_export.rs`
  - `unify-integration-tests/src/fixtures.rs`
  - `unify-rdf/src/pipeline.rs`
  - `unify-core/src/lib.rs`
  - `unify-rocket/src/lib.rs`
  - `unify-ffi/build.rs`
  - `unify-mcp/src/main.rs`
  - `unify/src/commands.rs`
  - `unify/src/main.rs`
  - `manifest.rs` (find exact path in exploration)
  - `classify.rs` (find exact path in exploration)
- `wasm-threads/`
  - `wasm-patterns/src/actor.rs`
  - `wasm-tests/tests/pattern_integration.rs`
  - `wasm-ui/tests/hud.rs`
  - `wasm-ui/tests/message_bridge.rs`
