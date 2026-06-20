# GC-MECH-ASSET-FABRIC-001B: Part-Aware Morphology Convergence

## Status
GC-MECH-ASSET-FABRIC-001: PARTIAL_ALIVE (render plumbing), BUILD_BROKEN (visual morphology)
The previous metric (silhouette_iou >= 0.25) over-admitted a blob with barcode wings. The factory is alive, but the quality gate admitted scrap. The old asset is now a negative false-positive fixture.

## Morphology Metrics (The New Gate)
The verifier must compute part-aware morphology metrics, not just whole-image metrics.
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
The Asset LSP must map visual_gap_report morphology failures to generic diagnostics:
- `VIS201 ERROR`: part-graph similarity below threshold.
- `VIS202 ERROR`: wing morphology mismatch.
- `VIS203 ERROR`: generated wing panels are line-primitives, expected layered swept plates.
- `VIS204 ERROR`: core body massing exceeds compactness bound.
- `VIS205 ERROR`: blade placement/angle mismatch.
- `VIS206 ERROR`: armor segmentation density below threshold.
- `VIS207 ERROR`: edge-density distribution mismatch.
- `VIS208 ERROR`: candidate passed coarse silhouette but failed morphology gate.

## Generator Repair Directives
Patch the source law (templates/parameters):
- **wing_primitive_family**: switch from parallel_line_array to layered_swept_feather_panel. Require tapered curved quad/mesh plates, thickness, bevel, overlap, rotation gradient, and length gradient.
- **core_body_shape**: reduce spherical bulk; increase angular armor shell hierarchy; add torso/head/shoulder distinction.
- **blade_family**: require two long cyan rods/blades matching angle/foreground placement.
- **detail_distribution**: increase local panel density on wings, shoulders, forearms, lower assemblies.

## Admission
Admission requires per-component residuals in `visual_gap_report.json` and LSP diagnostics for morphology failures.
