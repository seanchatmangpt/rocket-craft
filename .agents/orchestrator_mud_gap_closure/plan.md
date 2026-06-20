# Verification Plan — Mech Factory MUD Autonomous Gap-Closure Mode

## Phase 1: Diagnosis
1. Dispatch `teamwork_preview_explorer` to run initial diagnostic tools:
   - Run `python3 scripts/mud_gap_check.py` to identify current failing gaps.
   - Run `cargo check -p mech_factory_mud` and `cargo test -p mech_factory_mud` to assess compilation/test status.
   - Inspect existing ggen configuration files, ontologies, and template files.

## Phase 2: Code Generation and Alignment
1. Address ggen file syncing and output generation gaps.
2. Resolve any CSV alignment/canonical rows issues in FactoryStations.csv and WalkthroughRoute.csv.

## Phase 3: Game Logic Implementation
1. Fix/implement MUD game logic.
2. Run `cargo test -p mech_factory_mud` to ensure all 45+ unit/integration tests pass.

## Phase 4: CLI Commands
1. Run and verify `verify` and `replay` CLI commands.
2. Run and verify `falsify` and `counterfactual` commands across 8 cases.

## Phase 5: Final Validation
1. Run `python3 scripts/mud_gap_check.py` to verify that `Requirements failed: 0` is reached.
2. Validate that `cargo run -p mech_factory_mud -- verify` outputs `PASS`.
3. Check that workspace has 0 ignored or failed tests.
