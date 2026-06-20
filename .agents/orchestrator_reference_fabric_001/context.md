# Context — Reference Visual Convergence Manufacturing (GC-MECH-ASSET-FABRIC-001B)

## Workspace
- Target Project Directory: `/Users/sac/rocket-craft`
- Working Directory: `/Users/sac/rocket-craft/.agents/orchestrator_reference_fabric_001`
- Original Request Timestamp: 2026-06-20T00:19:04Z
- Active Milestone: `GC-MECH-ASSET-FABRIC-001B` (Part-Aware Morphology Convergence)
- Expected Scoped Status: `REFERENCE_TO_USD_RENDER_VERIFIED_UNDER_SCOPE`

## Inputs
- Source Reference Image: `/Users/sac/Documents/Papers/61gOtV1wnAL._AC_SL1200_.jpg`
- Local Copy Path: `references/mech/61gOtV1wnAL._AC_SL1200_.jpg`
- Morphology Specification: `/Users/sac/rocket-craft/.agents/SPR_MORPHOLOGY_CONVERGENCE.md`

## Output Target Directory
All manufactured artifacts and coordination files must go under `generated/mech_assets/reference_fabric_001/`.

## Negative False-Positive Fixture
The old scrap asset (blob with barcode wings) must be copied to:
`fixtures/negative/blob_barcode_wing_false_positive/`

## Visual Morphology Metrics
The verifier must compute part-aware morphology metrics:
- `part_graph_similarity`: matches broad part hierarchy (core, head, wings, weapons).
- `wing_layer_count_delta`: wings are layered plates, not barcode lines.
- `feather_panel_curvature_score`: wing panels curve and taper.
- `feather_overlap_depth_score`: plates overlap in depth.
- `core_compactness_delta`: core massing resembles humanoid target.
- `head_to_torso_ratio_delta`: proportions match.
- `blade_length_angle_delta`: cyan rods/blades match length/angle/placement.
- `armor_shell_segmentation_score`: armor reads as multi-depth shells/panels.
- `edge_density_distribution`: detail appears in the correct regions.
- `foreground_component_count`: generated object has the right part complexity.

## LSP Diagnostic Taxonomy (VIS Namespace)
Visual gap report morphology failures must map to:
- `VIS201 ERROR`: part-graph similarity below threshold.
- `VIS202 ERROR`: wing morphology mismatch.
- `VIS203 ERROR`: generated wing panels are line-primitives, expected layered swept plates.
- `VIS204 ERROR`: core body massing exceeds compactness bound.
- `VIS205 ERROR`: blade placement/angle mismatch.
- `VIS206 ERROR`: armor segmentation density below threshold.
- `VIS207 ERROR`: edge-density distribution mismatch.
- `VIS208 ERROR`: candidate passed coarse silhouette but failed morphology gate.

## Generator Repair Directives
- **wing_family**: switch from parallel_line_array to layered_swept_feather_panel (tapered curved quad/mesh plates, thickness, bevel, overlap, rotation gradient, length gradient).
- **core_body_shape**: reduce spherical bulk, increase angular armor shell hierarchy, torso/head/shoulder distinction.
- **blade_family**: require two long cyan rods/blades matching angle/foreground placement.
- **detail_distribution**: increase local panel density on wings, shoulders, forearms, lower assemblies.
