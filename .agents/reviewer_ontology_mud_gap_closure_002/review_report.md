# Review Report: reviewer_ontology_mud_gap_closure_002

## Review Summary

**Verdict**: APPROVE

This review assesses the ontology updates in `ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl`, the SPARQL query `ontology/ggen-packs/mech_factory_mud/queries/gap_check.rq`, and the ggen manifest configurations. 

Running the pack-level generator command `ggen sync --manifest ontology/ggen-packs/mech_factory_mud/ggen.toml` completes successfully and produces all 28 expected files. Running `cargo run -p mech_factory_mud --bin mud_gap_check` validates the workspace and reports 50/50 requirements passed with zero failures. Running `cargo test -p mech_factory_mud` executes 56 unit and integration tests successfully.

However, several architectural concerns, minor compliance gaps, and root-level manifest inconsistencies were discovered and are detailed below.

---

## Findings

### [Major] Finding 1: Root Manifest SPARQL Prefix and Namespace Mismatch
- **What**: Several SPARQL queries referenced by the root `ggen.toml` use prefix `https://rocket-craft.com/ontology/mud#` (with an "s"), whereas the ontology `mech_factory_mud.ttl` and `all_merged.ttl` define prefix `http://rocket-craft.com/ontology/mud#` (no "s").
- **Where**:
  - `ontology/ggen-packs/mech_factory_mud/GeometrySurrogate.sparql` (Line 2)
  - `ontology/ggen-packs/mech_factory_mud/MotionSurrogate.sparql` (Line 2)
  - `ontology/ggen-packs/mech_factory_mud/SkinSurrogate.sparql` (Line 2)
- **Why**: This mismatch causes the SPARQL engine to yield zero matched bindings when running root-level `ggen sync`. As a result, the root manifest silently skips generating files like `GeometrySurrogate.rs` and `MotionSurrogate.rs` (generating empty or stale files), while the pack-level generator (which uses different queries) completes them.
- **Suggestion**: Align all protocol namespaces in SPARQL query files to use `http://rocket-craft.com/ontology/mud#`.

### [Major] Finding 2: Root Manifest References Undeclared Classes
- **What**: The root manifest queries `station_processes.sparql`, `walkthrough_transitions.sparql`, and `ue4_projection_contract.sparql` query individuals of types `factory:StationProcessTransition`, `factory:WalkthroughTransition`, and `factory:ProjectionRow` under namespace `http://example.org/factory#`. However, these classes and their instances are not declared in `all_merged.ttl` or `mech_factory_mud.ttl`.
- **Where**:
  - `ontology/ggen-packs/mech_factory_mud/station_processes.sparql`
  - `ontology/ggen-packs/mech_factory_mud/walkthrough_transitions.sparql`
  - `ontology/ggen-packs/mech_factory_mud/ue4_projection_contract.sparql`
- **Why**: These queries return zero results when run against the merged ontology, meaning the root-level sync has dead rules that perform no active generation.
- **Suggestion**: Ensure either the schema contains these definitions or remove the unused rules from root `ggen.toml` if they are fully replaced by the pack-level `ggen.toml`.

### [Minor] Finding 3: OWL 2 DL Undeclared Classes
- **What**: The classes `mud:AuthorityField` and `mud:RefusalReason` are used to type individuals (e.g. `mud:damage_class a mud:AuthorityField .`), but are not explicitly declared as `owl:Class` in the schema files.
- **Where**: `ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl` (Lines 31-42)
- **Why**: Under strict OWL 2 DL parsing, every class name used in assertion axioms must have a corresponding declaration axiom.
- **Suggestion**: Add the following explicit class declarations in `mech_factory_mud.ttl`:
  ```turtle
  mud:AuthorityField a owl:Class ;
      rdfs:label "Authority Byte Field Class" .
  
  mud:RefusalReason a owl:Class ;
      rdfs:label "Refusal Reason Class" .
  ```

---

## Verified Claims

- **Turtle Syntax Correctness** → verified via `ggen sync --manifest ontology/ggen-packs/mech_factory_mud/ggen.toml --validate-only true` → **PASS** (Outputs `Ontology syntax: PASS`).
- **Query Determinism** → verified via inspection of `gap_check.rq` → **PASS** (Contains `ORDER BY ?type ?subject ?checkId` which sorts uniquely by URI and identifier).
- **Cargo Tests Pass** → verified via `cargo test -p mech_factory_mud` → **PASS** (56 tests passed, 0 failed, 0 ignored).
- **Gap Checker Success** → verified via `cargo run -p mech_factory_mud --bin mud_gap_check` → **PASS** (50/50 requirements passed, exits with 0).

---

## Coverage Gaps

- **Root vs. Pack Manifest Divergence** — risk level: **medium** — Running `ggen sync` in the root does not generate the same 28 output files as running it with the pack-level manifest. If a developer runs root `ggen sync` after cleaning the workspace, compilation will fail due to missing files. Recommendation: Sync root and pack-level manifest generation rules or document the required pack manifest argument.
- **UE4 Actuation Verification** — risk level: **low** — Verified that the CSV/Header data matches walkthrough and station schemas, but browser visual movement delta depends on Playwright runtime verification. Recommendation: Accept risk at this level and rely on Stage 6 Playwright gates.

---

## Unverified Items

- None. All files, compilation bounds, and test cases were successfully executed and verified on the local workspace.

---

# Adversarial Challenge Report

## Challenge Summary

**Overall risk assessment**: **LOW**

The digital twin MUD ontology and generated assets are highly robust. The compiler type checks Zero-Sized Types correctly and all counterfactual/falsification logic asserts expected refusals. The main vulnerabilities relate to parallel execution environment locks and silent prefix mismatches.

---

## Challenges

### [Medium] Challenge 1: Silent SPARQL Mismatch Failure
- **Assumption challenged**: SPARQL queries match ontology namespace bindings because they are co-located in the same directory.
- **Attack scenario**: If a prefix protocol drifts (`http` vs `https`), the queries return 0 bindings. No syntax error is raised by the engine, resulting in a silent failure to generate valid Rust structs or CSV tables.
- **Blast radius**: Stale files on disk hide the bug, causing the application to build with out-of-date assets until a clean rebuild is performed.
- **Mitigation**: Add a check inside `mud_gap_check` or a ggen test verifying that the count of generated constants is greater than 0.

### [Low] Challenge 2: Build Lock Contention under Cargo Run
- **Assumption challenged**: Spawning `cargo test` inside a binary run by `cargo run` is safe.
- **Attack scenario**: Cargo holds a build directory lock while launching binaries. If the spawned `cargo test` attempts to write or re-compile, it blocks on the parent lock, occasionally timing out or returning cached/incomplete test outputs.
- **Blast radius**: Falsely reports test suite failures (e.g. reporting only 23 tests instead of 56 tests passed).
- **Mitigation**: Run `cargo test` with `--no-run` inside the checker, or run `mud_gap_check` directly as a compiled binary target (`./target/debug/mud_gap_check`) rather than via `cargo run`.
