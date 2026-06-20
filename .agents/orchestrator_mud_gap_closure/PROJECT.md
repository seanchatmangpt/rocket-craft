# Project: Mech Factory MUD
# Scope: Autonomous Gap-Closure Mode

## Architecture
- Module/package boundaries: crates/mech_factory_mud (game server, verify, replay, falsify, counterfactual modes).
- ggen ontology and templates generating code for stations, routes, parts, etc.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Initial Gap Diagnostics | Run gap check script, identify failing checks | None | DONE |
| 2 | Code Generation Sync | Get ggen syncing >= 15 files and emitting required Rust and UE4 artifacts | M1 | DONE |
| 3 | Station and Route Alignment | Verify FactoryStations.csv and WalkthroughRoute.csv have correct canonical rows | M2 | DONE |
| 4 | MUD Crate Logic and Tests | Implement and pass MUD game logic and 45 tests | M3 | DONE |
| 5 | Verify & Replay Modes | Make verify/replay CLI command pass | M4 | DONE |
| 6 | Falsify & Counterfactual Modes | Make falsify and counterfactual CLI commands pass 8 cases each | M5 | DONE |
| 7 | Full Acceptance verification | Ensure 0 failed/ignored tests and 0 requirements failed | M6 | DONE |

## Interface Contracts
### Rust ↔ ggen
- Ontological typestates and generated files in crates/mech_factory_mud/src/generated_constants.rs and generated/mech_factory_mud/rust/
