# Project: Swarm Audit & Pipeline Manufacturability

## Architecture
The Project Orchestrator coordinates a multi-disciplinary swarm of subagents to audit, refine, and validate the `eden_server` and `ue4_ontology` RDF packs. This ensures that the combined graph can physically manufacture a working, multi-resolution Eden/GMF world with valid walkthroughs, byte-class typestates, and unforgeable receipts, using ONLY `ggen` as the manufacturing authority.

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

| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Exploration & Gap Audit | Audit `eden_server` and `ue4_ontology` packs against R1-R12, identify gaps, and produce Gap Report | None | DONE |
| 2 | Ontology & DL Refactoring | Refactor schemas in strict OWL 2 DL covering gameplay, resolutions, LOD, and authority | M1 | DONE |
| 3 | Validation & SHACL Hardening | Write SHACL shapes for all constraints and integrate custom SPARQL ASK rules in ggen.toml | M2 | DONE |
| 4 | The ALIVE Proof Generation | Configure generation rules in ggen.toml to output the 10 required deliverables | M3 | DONE |
| 5 | Actuation & Verification | Run tests, verify Playwright E2E visual motion delta, and check receipt signatures | M4 | IN_PROGRESS |
| 6 | Forensic Audit & Handoff | Forensic Auditor performs final integrity verification, produce handoff | M5 | PLANNED |

## Interface Contracts
- **Ontology Registries**: All Turtle files must reside in `/Users/sac/.ggen/packs/eden_server/ontology/` and `/Users/sac/.ggen/packs/ue4_ontology/`.
- **Validation Authority**: Running `ggen sync --validate-only true` in either pack must succeed with exit code 0.
- **Determinism**: All SPARQL Construct and Select queries must use `ORDER BY`.
- **Layer Separation**: Generated code files must target package/consumer subdirectories (e.g. `src/`, `output/`, `models/`), not the pack root.
- **No Cheating**: Absolutely zero hardcoded values, mock runtimes, or placeholder artifacts.
- **The ALIVE Proof**: Using ONLY the generated ontology, SPARQL, SHACL, and `ggen` manifests, the system must generate:
  1. A walkable GMF factory
  2. A complete mech assembly line
  3. A race facility
  4. A market facility
  5. A deterministic MUD walkthrough
  6. Renderable artifacts with valid Render BOMs
  7. Semantic LOD classifications
  8. Authority typestates
  9. Receipt paths
  10. States-of-resolution projections
