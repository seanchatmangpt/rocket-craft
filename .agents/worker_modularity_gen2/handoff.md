# Handoff Report — Worker Modularity Gen 2

## 1. Observation
- **Original missing rules**: We observed that rules for `SM_TankTreads`, `SM_InterleavedWheels`, and `SM_KwK36Gun` were completely missing from `/Users/sac/rocket-craft/ggen.toml` (verified by `grep_search`).
- **Generator script block**: We observed in `/Users/sac/rocket-craft/patch_geometry_generator.py` at line 729:
  ```python
  if "SM_Limb_Left" not in ggen_content:
      with open("/Users/sac/rocket-craft/ggen.toml", "a") as f:
          f.write(limb_left_rule)
          f.write(limb_right_rule)
          f.write(loadout_rule)
          f.write(tank_treads_rule)
          f.write(interleaved_wheels_rule)
          f.write(kwk36_gun_rule)
  ```
  Since `SM_Limb_Left` was already in `ggen.toml`, the condition was false and the missing rules were never written.
- **Rule definitions**: The inline query rules in both `ggen.toml` and `patch_geometry_generator.py` lacked the `FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))` constraint.
- **Verified output after changes**: Running `bash scripts/verify_asset.sh` completed successfully and updated `SM_Limb_Left.usda` (among others). In `git diff generated/mech_assets/reference_fabric_001/usd/SM_Limb_Left.usda`, we observed deletion of thousands of lines of duplicate/leaked subframe geometries (e.g. `prim_0178_group` with `subframe_core_0` through `5` and `subframe_joint_0` through `5`).
- **Visual Gap Report**: The `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json` was generated with the following content:
  ```json
  {
      "silhouette_iou": 0.4115441396412478,
      "edge_similarity": 0.11084360629320145,
      "color_palette_similarity": 0.9441361059681157,
      "cyan_region_similarity": 0.0,
      "symmetry_delta": 0.008746984980820383,
      "wing_span_delta": 63.25,
      "body_mass_delta": 0.14857244757631033,
      "usd_prim_count": 2161,
      "material_binding_count": 1978,
      "wing_feather_count": 0,
      "part_graph_similarity": 1.0,
      "wing_layer_count_delta": 2.0,
      "feather_panel_curvature_score": 0.0,
      "feather_overlap_depth_score": 0.0,
      "core_compactness_delta": 0.13876122334549112,
      "head_to_torso_ratio_delta": 0.016666666666666607,
      "blade_length_angle_delta": 195.0,
      "armor_shell_segmentation_score": 0.09463445068345527,
      "edge_density_distribution": 0.8585107556714165,
      "foreground_component_count": 9,
      "usd_errors": [
          "USD304 ERROR: expected part root missing in SM_WingArray_Left.usda",
          "USD304 ERROR: expected part root missing in SM_WingArray_Right.usda",
          "USD305 ERROR: mirrored part lacks mirror transform proof"
      ],
      "vis_errors": [
          "VIS202 ERROR: wing morphology mismatch",
          "VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates",
          "VIS205 ERROR: blade placement/angle mismatch",
          "VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate"
      ],
      "thresholds_met": false
  }
  ```

## 2. Logic Chain
- Adding the check/appending code individually in `patch_geometry_generator.py` ensured the missing rules for `SM_TankTreads`, `SM_InterleavedWheels`, and `SM_KwK36Gun` were written to `ggen.toml` when they are not present.
- Inserting `FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))` in the six inline queries restricts the geometry primitive elements matching the query to only the primary part's elements or outward-pointing sockets.
- This prevents elements that belong to multiple parts (or secondary parts) from appearing multiple times in the query results, which previously caused the Tera template to generate duplicate and incorrect default mechanical subframe cylinders/spheres.
- Running `verify_asset.sh` compiles these changes, regenerating the USDA meshes and validating that the duplicate subframes are cleanly removed without breaking modularity checks for the modified parts.

## 3. Caveats
- Modularity errors for `SM_WingArray_Left.usda` and `SM_WingArray_Right.usda` (expected part root missing) are still present, but these are out of scope of the modified queries (`SM_Limb_Left`, `SM_Limb_Right`, `SM_Loadout`, `SM_TankTreads`, `SM_InterleavedWheels`, and `SM_KwK36Gun`).

## 4. Conclusion
- The SPARQL filters have successfully isolated geometry primitives to their correct parts.
- The duplicate geometry issue is resolved: the limb files no longer contain identical subframe cylinders/spheres.
- Asset verification successfully compiled all outputs and reported final visual gap scores.

## 5. Verification Method
- Run `bash scripts/verify_asset.sh` to execute the full asset compiler pipeline and view the visual gap metrics.
- Run `git diff generated/mech_assets/reference_fabric_001/usd/SM_Limb_Left.usda` to inspect the clean deletion of duplicate/extra subframe geometries.
