# Project: Asset Manufacturing LSP (ggen-asset-lsp)
# Scope: Complete LSP server implementation using lsp-max for USDA/MaterialX diagnostics, visual proof routing, generator code actions, and OCEL integration.

## Architecture
The `ggen-asset-lsp` treats 3D assets (USD, MaterialX, textures, rigs, renders, receipts) as a diagnosable compiler surface. It maps asset pipeline errors and headless render results directly into LSP diagnostics and routes quick-fixes to generator parameter sources.

- **Workspace Path**: `/Users/sac/rocket-craft`
- **Crate Path**: `crates/ggen-asset-lsp`
- **Asset Scope**: `generated/mech_assets/reference_fabric_001/`
- **External Framework**: `/Users/sac/lsp-max`

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Exploration & Architecture Definition | Inspect lsp-max framework and examples, locate reference assets, define diagnostics mapping rules. | None | DONE |
| 2 | Crate Setup & Workspace Cargo Setup | Create crates/ggen-asset-lsp, update root Cargo.toml, configure dependencies on lsp-max. | M1 | DONE |
| 3 | Core LSP Server & Diagnostics | Implement LSP server shell, USD/MaterialX parsing, visual gap report routing. | M2 | DONE |
| 4 | Code Actions & OCEL Integration | Implement generator code actions, emit OCEL events for validation/repair lifecycles. | M3 | DONE |
| 5 | E2E Verification | Run review rounds and challenger tests on the core LSP server. | M4 | DONE |
| 6 | Morphology & Modularity Updates | Implement VIS200 morphology and USD300 modularity diagnostics (fingerprints, part boundaries, transform proofs). | M5 | IN_PROGRESS |
| 7 | Final Forensic Audit & Handoff | Run the Forensic Auditor to check for integrity violations and prepare the final handoff. | M6 | PLANNED |

## Interface & Diagnostic Contracts
### Diagnostics Rules
- **Missing Payload**: A prim of type `Mesh` or similar that should reference a payload, but lacks a valid reference (e.g. `payload = @mesh.usd@` missing).
- **Missing Material Binding**: A prim lacking a material binding, or referencing a non-existent material.
- **Unreceipted USD Prim**: Prims that do not have associated cryptographic receipts.
- **Visual Gap Routing**: If `visual_gap_report.json` indicates silhouette IOU < threshold, project error on the root Xform/Mesh in USDA.
- **usdchecker Logs**: Project usdchecker errors onto the matching USDA lines.

### Code Actions
- Targets the **source** (e.g., `template.usda.tera`, SPARQL queries, or Rust parameter row), NOT the generated USDA.

### OCEL Events
- Emit OCEL events (e.g. `validate`, `repair`) whenever validation or repairs are executed.
