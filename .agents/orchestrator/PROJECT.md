# Project: Rocket-Craft Deep Architecture Retrofit

## Architecture
This project implements a systematic 10-phase codebase retrofit across the entire `rocket-craft` codebase. It rewrites and migrates existing stubs, mocks, and placeholder components to actively and directly use the real implementations of:
- **`ggen`**: RDF/SPARQL-based state-machine code generator.
- **`chicago-tdd-tools`** (locally named `rocket-combinatorial-engine`): BDD-driven combinatorial state space exploration engine.
- **`unrdf`**: Local RDF parser and TripleStore library (consolidated in `tools/unrdf`).
- **`lsp-max`**: Standardized LSP framework (replacing `tower-lsp`).

All dependencies are successfully redirected to local path dependencies, ensuring a self-contained, reproducible builds environment.

## Milestones
| Phase | Name | Scope | Status |
|---|---|---|---|
| 1 | Baseline Verification | Verify workspace-wide builds and test suites | DONE |
| 2 | Migrate `unrdf` | Redirect all Git references to local `tools/unrdf` | DONE |
| 3 | Migrate `chicago-tdd-tools` | Redirect all Git references to local `chicago-tdd-tools` | DONE |
| 4 | Verify `lsp-max` | Migrate `anti-llm-cheat-lsp` from `tower-lsp` to `lsp-max` | DONE |
| 5 | Refactor Stubs | Scan and eliminate stubs in `genie-core`/`unify-rdf` | DONE |
| 6 | Refactor State Machines | Migrate combat and player session machines to `Machine<L, P>` | DONE |
| 7 | Configure `ggen` Code-Gen | Build `ggen.toml`, templates, ontology, run `ggen sync` | DONE |
| 8 | Wire Playwright with TDD | Verify Playwright E2E visual delta tests generate receipts | DONE |
| 9 | Combinatorial Exploration | Run BDD-driven `combinatorial-engine` on state spaces | DONE |
| 10| Forensic Auditing | Independent Forensic Auditor validation & CLEAN VERDICT | DONE |

## Code Layout
- `chicago-tdd-tools/` (contains `rocket-combinatorial-engine`)
- `tools/unrdf/` (contains consolidated `unrdf`)
- `unify-rs/anti-llm-cheat-lsp/` (uses `lsp-max`)
- `nexus-engine/` (uses code-generated `Machine<L, P>` kernels)
