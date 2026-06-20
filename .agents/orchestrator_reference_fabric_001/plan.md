# Plan — GC-MECH-ASSET-FABRIC-001B (Part-Aware Morphology Convergence)

## Phase 1: Negative Fixture Setup
- [ ] Spawn Worker to copy current generated assets under `generated/mech_assets/reference_fabric_001/` to `fixtures/negative/blob_barcode_wing_false_positive/`.
- [ ] Confirm existence and integrity of negative false-positive fixture.

## Phase 2: Morphology Metrics & Diagnostics
- [ ] Spawn Worker to implement component-aware morphology similarity metrics in `scripts/compare_reference_render.py`:
  - `part_graph_similarity`
  - `wing_layer_count_delta`
  - `feather_panel_curvature_score`
  - `feather_overlap_depth_score`
  - `core_compactness_delta`
  - `head_to_torso_ratio_delta`
  - `blade_length_angle_delta`
  - `armor_shell_segmentation_score`
  - `edge_density_distribution`
  - `foreground_component_count`
- [ ] Implement LSP Visual Diagnostic mapping (VIS201 to VIS208).

## Phase 3: Generator Patching
- [ ] Spawn Worker to modify `generate_ttl.py` to:
  - Generate wings as layered swept feather-panel meshes (with thickness, bevel, overlap, rotation gradient, and length gradient).
  - Generate core body with angular armor shell hierarchy (torso/head/shoulder distinction).
  - Generate two long cyan rods/blades matching angle/placement.
- [ ] Run `ggen sync` via worker to update USD and MaterialX files.

## Phase 4: Headless Rendering & Validation
- [ ] Run headless renders and visual comparison suite to calculate new morphology metrics.
- [ ] Update `scripts/asset_fabric_gap_check.py` to include the new morphology metrics and VIS diagnostic assertions.

## Phase 5: Verification & Auditing
- [ ] Run gap checker and verify that 100% check gates pass.
- [ ] Spawn Forensic Auditor to perform integrity validation under new morphology rules.
- [ ] Submit final status to Sentinel.
