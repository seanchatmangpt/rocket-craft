# Original User Request

## Initial Request — 2026-06-19T17:23:43-07:00

# Teamwork Project Prompt: Asset Manufacturing LSP (ggen-asset-lsp)

Goal: Implement the Asset Manufacturing LSP (`ggen-asset-lsp`) using `~/lsp-max`.
Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Mission
You are tasked with building `ggen-asset-lsp`, a 3D Asset Manufacturing Language Server. This is not a standard code LSP. It treats USD, MaterialX, textures, rigs, renders, and receipts as a living, diagnosable compiler surface. It uses the framework at `/Users/sac/lsp-max`.

## Requirements

### R1. Crate Initialization
Create `crates/ggen-asset-lsp` inside the `rocket-craft` workspace. It must depend on the LSP framework crates located at `/Users/sac/lsp-max`. 

### R2. USD/MaterialX Diagnostic Authority (GC-ASSET-LSP-001)
Implement the LSP server to index the directory `generated/mech_assets/reference_fabric_001/`. It must parse `.usda` and `.mtlx` files to detect missing payloads, missing material bindings, and unreceipted USD prims.

### R3. Visual Proof Routing
The LSP must parse headless render outputs (`visual_gap_report.json`) and `usdchecker` logs, projecting them as `PublishDiagnostics` errors directly onto the USDA text in the editor (e.g., highlighting `def Mesh` if the silhouette IOU falls below threshold).

### R4. Code Actions for Source Law
Provide LSP Code Actions that target the **generator parameter source** (the SPARQL query, Tera template, or Rust parameter row), NOT the generated USD output. Asset instances are immutable; the source law is what must be repaired.

### R5. OCEL Integration
Emit an OCEL event for the LSP diagnostic lifecycle whenever the LSP runs a validation pass or a repair action.

## Acceptance Criteria

### Implementation
- [ ] `crates/ggen-asset-lsp` compiles successfully against the local `~/lsp-max` path.
- [ ] The binary can launch and respond to `initialize` and `textDocument/didOpen` requests.

### Diagnostics & Routing
- [ ] A missing `payload = @mesh.usd@` in a parsed USDA file correctly triggers an LSP Diagnostic.
- [ ] Failures in `visual_gap_report.json` are successfully mapped to diagnostics on the root `Xform` or `Mesh` in the USDA.

## Follow-up — 2026-06-20T00:46:45Z

EMERGENCY CORRECTION: The asset you verified was a false-positive admitted by a weak whole-image metric. The pipeline is now moving to **GC-MECH-ASSET-FABRIC-001B** (Part-Aware Morphology Convergence).

Update your diagnostic engine to support the new `VIS200` series taxonomy for morphology failures.
Do NOT use franchise-specific language.

New diagnostics to implement:
- VIS201 ERROR: part-graph similarity below threshold.
- VIS202 ERROR: wing morphology mismatch.
- VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates.
- VIS204 ERROR: core body massing exceeds compactness bound.
- VIS205 ERROR: blade placement/angle mismatch.
- VIS206 ERROR: armor segmentation density below threshold.
- VIS207 ERROR: edge-density distribution mismatch.
- VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate.

See `/Users/sac/rocket-craft/.agents/SPR_MORPHOLOGY_CONVERGENCE.md` for the full spec. The LSP must surface these errors when the new gap report includes per-component morphology residuals.

## Follow-up — 2026-06-20T00:49:38Z

EMERGENCY CORRECTION: The USD output you are diagnosing failed modularity constraints. Every file was a duplicate of the full assembly. 

Add the following `USD300` diagnostics to the Asset LSP immediately:
- USD301 ERROR: duplicate USD geometry fingerprint.
- USD302 ERROR: part file renders full assembly.
- USD303 ERROR: part-local file contains foreign component prims.
- USD304 ERROR: expected part root missing.
- USD305 ERROR: mirrored part lacks mirror transform proof.
- USD306 ERROR: generated USD files share identical source template expansion.
- USD307 ERROR: part bounding box overlaps full-asset bounds.

See `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`. The LSP must flag these if multiple `.usda` files share the exact same primitive composition or if `SM_Head.usda` contains a Torso mesh.
