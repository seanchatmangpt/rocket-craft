# Rocket-Craft Photorealistic Sculpting Plan (POWL v2 Parallel Plan)

## Architecture
- **Source Law Compilation:** Authoritative TTL files in `ontology/source_law/` are merged via `scripts/merge_ontology.py` to produce `ontology/all_merged.ttl`.
- **Mesh Generation:** `ggen sync` processes `all_merged.ttl` using queries and templates in `generated/mech_assets/reference_fabric_001/templates/` to produce `.usda` files in `generated/mech_assets/reference_fabric_001/usd/`.
- **Pre-Render Morphology Gate:** `scripts/verify_metric_morphology.py` checks that generated USD bounds satisfy RDFS/SHACL laws (including `110_bipedal_metric_envelope_law.ttl`, `116_metric_morphology_bands.ttl`, `117_reference_fabric_metric_binding.ttl`).
- **Render & Score Loop:** `scripts/render_reference_fabric.py` deletes stale renders, runs `usdrecord`, and outputs edge maps/silhouettes. `scripts/compare_reference_render.py` calculates visual delta/metrics against targets.
- **Playwright Verification:** `verify_mecha_pipeline.sh` spins up the server and runs Playwright E2E walkthrough test on the staged WebGL page.

## POWL v2 Parallel Execution Graph
```
ValidateBipedalKitCoherence (M2) 
   │
   ├──► GenerateUpperBodyGeometry (M3) [Speculative parallel candidate branch]
   ├──► GenerateLowerBodyGeometry (M4) [Speculative parallel candidate branch]
   └──► GenerateWingAndShieldGeometry (M5) [Speculative parallel candidate branch]
   │
   ▼ (Join branches)
AssembleFullMech (M6) ──► GenerateMaterialZones ──► FreshRender & E2E Validation
```

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Exploration & Baseline | Run verify scripts, analyze current geometry/metrics, read reference image, outline precise edits. | None | DONE |
| 2 | Bipedal Kit Coherence Law | Add SHACL constraints for neck, shoulder/hip sockets, joints, manipulator grip, shield/wing mounts. Ensure SHACL passes. | M1 | DONE |
| 3 | Upper Body Geometry | Sculpt head (face depth, cheek guards, helmet volume), torso (layered armor, waist, abdomen), and shoulders (pauldrons). | M2 (Speculative) | DONE |
| 4 | Lower Body Geometry | Sculpt limbs (upper arm, forearm, elbows, wrists) and legs (thighs, knees, shins, ankles, feet). | M2 (Speculative) | DONE |
| 5 | Wing & Shield Geometry | Sculpt wing feather arrays (segmented curved overlapping panels) and shield (thickness hierarchy, handle, forearm mount). | M2 (Speculative) | DONE |
| 6 | Assemble & Verify | Join branches, merge source law, generate material zones, run fresh renders, E2E walkthrough test, and receipts. | M2, M3, M4, M5 | ADMITTED |

## Interface Contracts
- **Socket Connectivity:** Sockets must align exactly at part boundaries. Socket names and attachment target parts must match anatomical topology.
- **Mesh Parametrization:** Procedural loops in `part_mesh.usda.tera` must be governed by properties extracted from the RDF graph.
- **No Hardcoded Mocks:** All generated meshes must originate from the compiled source law without hardcoded bypasses.
