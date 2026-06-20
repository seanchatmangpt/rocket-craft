# Swarm Audit & Pipeline Manufacturability Plan

This plan details the milestones to aggressively audit, align, and validate the `eden_server` and `ue4_ontology` RDF packs to satisfy the pure TPS "ALIVE Proof" manufacturing pipeline requirement.

## Architecture & Integration
The orchestrator coordinates a swarm of up to 20 subagents to bridge the gaps in the ontologies so `ggen` can manufacture all 10 core deliverables.

```
                  ┌─────────────────────────────────────┐
                  │        Project Orchestrator         │
                  └──────────────────┬──────────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         ▼                           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
│  M1: Exploration │ ──────►│  M2: Refactoring │ ──────►│   M3: Validation │
│   (Gap Audit)    │        │  (Ontology/DL)   │        │   (SPARQL/SHACL) │
└──────────────────┘        └──────────────────┘        └──────────────────┘
                                                                 │
         ┌───────────────────────────────────────────────────────┘
         ▼
┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
│   M4: Generation │ ──────►│  M5: Verification│ ──────►│ M6: Final Audit  │
│  (The ALIVE Proof)│       │ (Tests & Replays)│        │   & Handoff      │
└──────────────────┘        └──────────────────┘        └──────────────────┘
```

## Milestones

| # | Milestone Name | Scope | Dependencies | Status |
|---|----------------|-------|--------------|--------|
| 1 | Exploration & Gap Audit | Spawn Explorer subagents to audit `eden_server` and `ue4_ontology` packs against R1-R12, identify missing surfaces/typestates, and produce a detailed Gap Report. | None | PLANNED |
| 2 | Ontology & DL Refactoring | Enrich the turtle schemas to cover all 12 gameplay cells, 8 resolution levels, semantic importance LODs, and authority models in strict OWL 2 DL. | M1 | PLANNED |
| 3 | Validation & SHACL | Write SHACL shapes for all new constraints and integrate custom SPARQL ASK validation rules in `ggen.toml` with `strict_mode = true`. | M2 | PLANNED |
| 4 | The ALIVE Proof Gen | Configure generation rules in `ggen.toml` to output the 10 required deliverables from the graph and verify successful `ggen sync` compile. | M3 | PLANNED |
| 5 | E2E Actuation & Verification | Run the test suites, verify Playwright E2E visual motion delta, console logs, and cryptographic receipt generation. | M4 | PLANNED |
| 6 | Forensic Audit & Handoff | Forensic Auditor performs final integrity verification. Deliver final handoff. | M5 | PLANNED |

## Interface Contracts & Validation Rules
1. **Ontology Validity**: Validated via `ggen sync --validate-only true` (must exit 0).
2. **Determinism**: All SPARQL Construct and Select queries must use `ORDER BY`.
3. **Layer Separation**: Generated code files must target package/consumer subdirectories, not the pack root.
4. **No Cheating**: Absolutely zero hardcoded values, mock runtimes, or placeholder artifacts.
