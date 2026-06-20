## 2026-06-20T00:49:36Z
You are a teamwork_preview_worker agent.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_morphology`
Your task is to implement the GC-MECH-ASSET-FABRIC-001B visual morphology convergence loop.

Follow these instructions exactly:
1. **Negative Fixture Setup**:
   - Create the directory `/Users/sac/rocket-craft/fixtures/negative/blob_barcode_wing_false_positive/` if it does not exist.
   - Copy all current generated assets from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/` into `fixtures/negative/blob_barcode_wing_false_positive/` (preserving folder layout).

2. **Patch Generator Parameters & Ontology**:
   - Modify `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_generation/generate_ttl.py` (or write a modified version and run it) to generate the new morphology graph parameters.
   - In `generate_ttl.py`, update:
     - **Torso Core & Core Body Shape**: Reduce spherical bulk, increase angular armor shell hierarchy, and add torso/head/shoulder distinction by defining primitive Familien with angular coordinates, scales, and rotation offsets.
     - **Wings (Primary/Secondary wing feathers)**: Switch from simple parallel lines to layered swept feather-panel meshes. Generate 48 wing feathers (24 left, 24 right) with smooth gradients for translation, rotation, and length. In your geometry generation, make sure the meshes are defined as tapered curved plates with thickness, bevel, and overlap.
     - **Blades**: Ensure the left/right blades are represented as two long cyan rods/blades with correct angle/placement.
     - **Detail Distribution**: Increase local panel density on wings, shoulders, forearms, lower assemblies.
   - Run the script to update `generator_parameters.ttl`, and merge the new Turtle content into `/Users/sac/rocket-craft/ontology/all_merged.ttl`.

3. **Implement Morphology Metrics in verifier**:
   - Modify `scripts/compare_reference_render.py` to calculate the 10 part-aware morphology metrics from the rendered front image:
     - `part_graph_similarity` (compare part hierarchy to reference)
     - `wing_layer_count_delta` (verify multiple wing plate layers)
     - `feather_panel_curvature_score` (verify tapered/curved panels)
     - `feather_overlap_depth_score` (verify overlapping plates)
     - `core_compactness_delta` (measure compactness profile of core body)
     - `head_to_torso_ratio_delta` (measure head vs torso pixel heights)
     - `blade_length_angle_delta` (verify cyan blade length/angle placement)
     - `armor_shell_segmentation_score` (verify multi-depth panel segments)
     - `edge_density_distribution` (measure detail frequency in wings/shoulders/core)
     - `foreground_component_count` (count disconnected foreground parts)
   - Map morphology check failures to diagnostic errors:
     - `VIS201 ERROR`: part-graph similarity below threshold.
     - `VIS202 ERROR`: wing morphology mismatch.
     - `VIS203 ERROR`: generated wing panels are line-primitives, expected layered swept plates.
     - `VIS204 ERROR`: core body massing exceeds compactness bound.
     - `VIS205 ERROR`: blade placement/angle mismatch.
     - `VIS206 ERROR`: armor segmentation density below threshold.
     - `VIS207 ERROR`: edge-density distribution mismatch.
     - `VIS208 ERROR`: candidate passed coarse silhouette but failed morphology gate.
   - Add these metrics to `visual_gap_report.json`, `visual_gap_report.md`, `verifier_report.json`, `verifier_report.md`.

4. **Update Gap Checker**:
   - Modify `scripts/asset_fabric_gap_check.py` to include the new morphology convergence checks, check for diagnostic VIS error codes under failure modes, and write `reports/gap_closure_report.json` and `reports/gap_closure_report.md` detailing pass/fail status.
   - Ensure the new gate checks pass.

5. **Execution & Validation**:
   - Run `ggen sync` to generate the new USD/MaterialX files.
   - Run the render pipeline `python3 scripts/render_reference_fabric.py` to render the new visual morphology.
   - Run the comparison suite `python3 scripts/compare_reference_render.py` and the gap checker `python3 scripts/asset_fabric_gap_check.py` to verify 100% checks pass.
   - If any metric fails, adjust parameters in `generate_ttl.py` and re-run.

6. **Receipts & OCEL**:
   - Update `asset_manufacturing.ocel.json` and `asset_receipts.jsonl` to cover the new generation steps, metrics, and artifact hashes.
7. Create a handoff report in `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_morphology/handoff.md` detailing the changes and metrics.
8. Send a completion message to the parent (conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d).
