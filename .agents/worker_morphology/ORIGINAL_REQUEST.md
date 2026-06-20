## 2026-06-20T00:47:47Z

Your identity: You are Worker 3 (archetype: worker/teamwork_preview_worker).
Your working directory is /Users/sac/rocket-craft/.agents/worker_morphology
Your task: Implement the Morphology Convergence Update (GC-MECH-ASSET-FABRIC-001B) in `crates/ggen-asset-lsp`.

Specifically, you must:
1. Update `crates/ggen-asset-lsp/src/diagnostics.rs` to support the new `VIS200` series diagnostic taxonomy for morphology failures.
2. Read the spec in `/Users/sac/rocket-craft/.agents/SPR_MORPHOLOGY_CONVERGENCE.md` carefully to see what keys/metrics are computed.
3. In `diagnostics.rs`, extend the parsing of `visual_gap_report.json` (inside `run_diagnostics` function) to inspect the following metrics and generate the corresponding LSP diagnostics:
   - VIS201: If `part_graph_similarity` < 0.90 (or if it is present and below 0.90), emit `Diagnostic` with code "VIS201" and message "VIS201 ERROR: part-graph similarity below threshold".
   - VIS202: If `wing_layer_count_delta` is present and > 0, or `feather_overlap_depth_score` < 0.90, or `feather_panel_curvature_score` < 0.90, emit `Diagnostic` with code "VIS202" and message "VIS202 ERROR: wing morphology mismatch".
   - VIS203: If `wing_panels_are_line_primitives` is true (in the JSON) or if `feather_panel_curvature_score` < 0.50, emit `Diagnostic` with code "VIS203" and message "VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates".
   - VIS204: If `core_compactness_delta` > 0.10 or `head_to_torso_ratio_delta` > 0.10, emit `Diagnostic` with code "VIS204" and message "VIS204 ERROR: core body massing exceeds compactness bound".
   - VIS205: If `blade_length_angle_delta` > 0.10, emit `Diagnostic` with code "VIS205" and message "VIS205 ERROR: blade placement/angle mismatch".
   - VIS206: If `armor_shell_segmentation_score` < 0.90, emit `Diagnostic` with code "VIS206" and message "VIS206 ERROR: armor segmentation density below threshold".
   - VIS207: If `edge_density_distribution` < 0.90, emit `Diagnostic` with code "VIS207" and message "VIS207 ERROR: edge-density distribution mismatch".
   - VIS208: If the coarse silhouette check passed (e.g. `silhouette_iou` >= 0.90) but the overall status is "FAILED" due to morphology errors, emit `Diagnostic` with code "VIS208" and message "VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate".
   - Note: Do NOT use franchise-specific language.
4. Add or update unit tests in `diagnostics.rs` to verify that these new `VIS200` series diagnostics are correctly detected and projected onto the USDA file.
5. Run `cargo test -p ggen-asset-lsp` to verify all tests pass successfully.
6. Write your report to `/Users/sac/rocket-craft/.agents/worker_morphology/handoff.md` and send a message back to the orchestrator.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
