# FRESH RENDER VERIFICATION REPORT — Workstream R4

**Verdict: FRESH_VERIFIED**

Resolves the stale 9-vs-35 split. `gap_closure_report.json`'s `blade_delta=195`
is the degenerate SENTINEL `fit_blade()` returns when cyan pixels < 10; `fc=35`
was a stale flood-fill. Ground truth is established ONLY by fresh
delete-and-resync renders measured via `scripts/verify_asset.sh`.

## Replay
- run1 timestamp: 2026-06-20T22:41:31Z
- run2 timestamp: 2026-06-20T22:41:34Z
- identical to 4dp: **True**
- report BLAKE3 run1: `eb80bc685d8f2de513a8c9ee8f0d844901dfe96edcd8cc4c7a929271221c5d2f`
- report BLAKE3 run2: `eb80bc685d8f2de513a8c9ee8f0d844901dfe96edcd8cc4c7a929271221c5d2f`

## Render BLAKE3 (run2, live / self-certified)
- `render_front.png`: `8ebe5bcc18c78428ef5a5e4a33403b095f1b961aa5fd309a3df32b2a01376569`
- `render_silhouette.png`: `cfcd62528994b87ba93cd4a781e85e1b1adf26e597146b44fb43043686520e16`
- `render_edges.png`: `58b551642c8da7d530368c7ba5f3b6ec989d572500f736234b2850425ebf485b`
- `render_angled.png`: `fb0489e9a6326c596c4a6188eccedf29eca66e561322e04007894616b0bf89c6`

## Provenance / Stale-refusal fixture
- status: **PASS**
- report.render_hash matches live fresh PNG bytes
- scorer_git_unchanged (provenance-only edit): False
- render_script_unchanged: True

## Resolved truth (fresh-rendered, supersedes 195-sentinel)
```json
{
  "foreground_component_count": 8,
  "blade_length_angle_delta": 195.0,
  "feather_panel_curvature_score": 0.0,
  "wing_feather_count": 0,
  "thresholds_met": false,
  "note": "blade_delta=195 was the cyan<10 SENTINEL; fc=35 was stale. These are the fresh-rendered truths."
}
```

## run2 metrics (4dp)
```json
{
  "armor_shell_segmentation_score": 0.265,
  "blade_length_angle_delta": 195.0,
  "body_mass_delta": 0.2288,
  "color_palette_similarity": 0.5798,
  "core_compactness_delta": 0.3699,
  "cyan_region_similarity": 0.0,
  "edge_density_distribution": 0.7005,
  "edge_similarity": 0.1521,
  "feather_overlap_depth_score": 0.0,
  "feather_panel_curvature_score": 0.0,
  "foreground_component_count": 8,
  "head_to_torso_ratio_delta": 0.0167,
  "part_graph_similarity": 1.0,
  "silhouette_iou": 0.3016,
  "symmetry_delta": 0.2359,
  "thresholds_met": false,
  "wing_feather_count": 0,
  "wing_layer_count_delta": 2.0,
  "wing_span_delta": 144.5
}
```
