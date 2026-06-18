# Project: Gundam Nexus Manufacturing Facility (GMF)

## Architecture
The Gundam Nexus Manufacturing Facility (GMF) is an autonomous, ontology-driven manufacturing plant representing the Chatman Equation (A = μ(O^*)).
The factory is organized as 6 operational zones (Foundry, Runner Wall, Assembly Gantry, Fit/Collision Bay, Proving Ground, Reveal Platform) that physically project raw ontological primitives into a walkable 3D WebGL2 environment.

```
                      ┌──────────────────────────────────────┐
                      │        Ontology Source (O*)          │
                      └──────────────────┬───────────────────┘
                                         │
                                         ▼ (SPARQL Extraction μ)
                      ┌──────────────────────────────────────┐
                      │    Branchless Parts Projection (A)   │
                      └──────────────────┬───────────────────┘
                                         │
                                         ▼ (Manufacturing Gates)
                      ┌──────────────────────────────────────┐
                      │      Verification & Receipt (R_B)     │
                      └──────────────────────────────────────┘
```

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Workspace Setup & Diagnostics | Verify clean build and test status of all monorepo workspaces. | None | DONE |
| 2 | GMF Backend & CLI Integration | Integrate PMME/TPS typestates and generation rules into unify CLI and genie-core. | M1 | PLANNED |
| 3 | Operational Surface & 3D Web Front | Implement 6 operational zones and procedurally render the assembled mech in WebGL. | M2 | PLANNED |
| 4 | E2E Playwright Verification | Execute the HTML5 pipeline, run E2E Playwright verification, verify visual delta, log BLAKE3 receipt. | M3 | PLANNED |
| 5 | Quality and Integrity Validation | Run Chaos tests, execute independent Forensic Auditor reviews, and finalize. | M4 | PLANNED |

## Interface Contracts
- **E2E Playwright Receipt**: `/Users/sac/rocket-craft/pwa-staff/test-results/tps-dflss-receipt.json` must exist with verdict `"PASS"`.
- **WASM Manufacturing Output**: `Brm-HTML5-Shipping.wasm` / `Brm-HTML5-Shipping.data` dynamically generated.

## Code Layout
- `unify-rs/genie-core/` (Genie core world specification, parser, evolver, and deployment)
- `nexus-engine/crates/nexus-tps/` (Branchless mech parts generator and Jidoka validation)
- `nexus-engine/crates/nexus-pmme/` (Procedural mech manufacturing engine typestates)
- `genie-web/` (Interactive 3D WebGL2 dashboard)
- `pwa-staff/` (PWA server, offline worker, and Playwright E2E tests)
