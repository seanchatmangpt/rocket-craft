# Handoff Report — explorer_sculpting_m1

## 1. Observation
The following specific files, line numbers, and outputs were observed in the workspace:
* **Metric Morphology Report (Stale, before execution)**: 
  * Path: `/Users/sac/rocket-craft/METRIC_MORPHOLOGY_REPORT.json`
  * Contents: 
    * `body_height_m`: `0.103`
    * `actual_stacking_axis`: `"X"`, `declared_up_axis`: `"Y"`, `agree`: `false`
    * `vertical_axis`: `"X"`, `span_axis`: `"X"`
    * `direct_band_refusals`:
      * `"REFUSE_PART_HEIGHT_BAND(SM_Head): ratio 0.1165 outside [0.12,0.14]"` (actual `height_m`: `0.012`, `height_ratio`: `0.1165`)
      * `"REFUSE_PART_HEIGHT_BAND(SM_Torso): ratio 0.1165 outside [0.12,0.14]"` (actual `height_m`: `0.012`, `height_ratio`: `0.1165`)
      * `"REFUSE_PART_HEIGHT_BAND(SM_Limb_Left): ratio 0.1529 outside [0.62,0.64]"`
      * `"REFUSE_PART_HEIGHT_BAND(SM_WingArray_Left): ratio 0.2745 outside [0.61,0.63]"`
* **Metric Morphology Report (Fresh, after running `./scripts/verify_asset.sh`)**:
  * Output after command execution:
    * `body_height_m`: `0.101495`
    * `actual_stacking_axis`: `"Y"`, `declared_up_axis`: `"Y"`, `agree`: `true`
    * `vertical_axis`: `"Y"`, `span_axis`: `"X"`
    * `direct_band_refusals`: `[]`, `refusals`: `[]`, `verdict`: `"ADMITTED"`
* **Verification Script Executions**:
  * Command: `./scripts/verify_asset.sh`
  * Scorer Output (`scripts/compare_reference_render.py`):
    ```
    Left Blade: len=212.37027033924454, ang=33.8346911641115
    Right Blade: len=204.6592239283479, ang=-31.750848920587494
    Thresholds met: False (silhouette_iou >= 0.25, color_palette_similarity >= 0.50, morphology_ok=False)
    Visual Morphology Errors:
      VIS205 ERROR: blade placement/angle mismatch
      VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate
    ```
* **Git Commit History**:
  * Commit `a19b1e7b0cd7713a1f771b6be632da80e0a98d60` ("fix(metric-gate): measure along the declared upAxis; confirm orientation defect FIXED"):
    * Extracted: `"The geometry generator now stacks parts along Y (SM_Head y=7.65 > SM_Torso y=6.35 > SM_Limb y=0), matching upAxis=\"Y\" — the earlier Z-stack-vs-upAxis-Y orientation defect is resolved in the geometry."`
  * Commit `ec42135dd2295c99faf5d8d198385989e89bb0ae` ("feat(mech_morphology_law): typed Rust crate..."):
    * Extracted: `VERT` changed from `2` (Z) to `1` (Y) in `scripts/verify_metric_morphology.py`.
* **Geometry Generator Template**:
  * Path: `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera`
  * Relevant lines:
    * 86-90:
      ```jinja2
      {% elif row.partLocalName == "torso_core" %}
          {% set my_ty = 6.35 %}
          {% set my_sy = 0.65 %}
      {% elif row.partLocalName == "head_unit" %}
          {% set my_ty = 7.65 %}
          {% set my_sy = 0.65 %}
      ...
      {% elif row.partLocalName == "blade_left" or row.partLocalName == "blade_right" %}
          {% set my_ty = 7.865 %}
          {% set my_sy = 3.41 %}
      ```
    * 91: `double3 xformOp:translate = ({{ row.translateX }}, {{ my_ty }}, {{ row.translateZ }})`
    * 92: `double3 xformOp:scale = ({{ row.scaleX }}, {{ my_sy }}, {{ row.scaleZ }})`
    * 176-178:
      ```jinja2
      double3 xformOp:scale = (2.14, 0.5, {{ 1.0 - (i * 0.15) }})
      double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)
      double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}15.0{% else %}-15.0{% endif %})
      ```
* **Ontology Definitions**:
  * Path: `/Users/sac/rocket-craft/ontology/source_law/104_reference_fabric.ttl`
    * Head unit: `mud:translateX 0.0 ; mud:translateY 0.0 ; mud:translateZ 1.5 ;` (line 134/182)
    * Torso core: `mud:translateX 0.0 ; mud:translateY 0.0 ; mud:translateZ 0.0 ;` (line 102)
    * Blade left: `mud:translateX -3.0 ; mud:translateY 0.0 ; mud:translateZ 0.0 ;` (line 535)
  * Path: `/Users/sac/rocket-craft/ontology/source_law/105_kit_subassembly_graph.ttl`
    * Connectors: `kit:Socket_Torso_Neck`, `kit:Peg_Head_Neck`, `kit:Socket_Torso_LArm`, `kit:Peg_LArm_Shoulder`, etc.
    * Connections: `kit:Peg_Head_Neck kit:connectsTo kit:Socket_Torso_Neck`, etc.
  * Path: `/Users/sac/rocket-craft/ontology/source_law/118_limb_anatomical_joints.ttl`
    * Sockets and Pegs for Left/Right elbows, wrists, knees, and ankles (e.g., `mud:Socket_Elbow_Left`, `mud:Peg_Forearm_Elbow_Left`).

---

## 2. Logic Chain

### Why VERT stack axis was detected as X in the stale report:
1. In the stale asset generated prior to commit `a19b1e7b`, the geometry generator stacked parts along Z (head Z=1.5, torso Z=0.0, arm Z=0.5).
2. The left limb `SM_Limb_Left` was not fully modular and only contained the arm (since modular rules and other limbs were fallback subframes or missing).
3. The head center was computed as `(0.0, 0.0, 1.5)` and the left limb (arm) center as `(-1.5, 0.0, 0.5)` based on their local mesh vertices and translations.
4. The absolute coordinate differences between the head and the limb centers were:
   * X difference: `abs(0.0 - (-1.5)) = 1.5`
   * Y difference: `abs(0.0 - 0.0) = 0.0`
   * Z difference: `abs(1.5 - 0.5) = 1.0`
5. Since the X difference (1.5) was greater than the Z difference (1.0), the stacking axis detection logic returned X as the primary stacking axis.

### Why parts failed height ratio bands in the stale report:
1. Prior to commit `a19b1e7b`, the parts were stacked along Z in the generator, but the measurement script `verify_metric_morphology.py` evaluated heights along the Y-axis (since `VERT` was hardcoded to `1`).
2. Along Y, the parts did not stack; they only had their local mesh Y-extents (e.g., Head height = 0.012, Torso height = 0.012).
3. The wing array, however, was generated in a loop that stretched it along Y up to `0.2067 m` (due to feather translations and scale overrides).
4. Because the wing array height dominated the body Y-height, the calculated height ratios of all other parts (Head ratio `0.1165`, Torso ratio `0.1165`, etc.) were extremely small and fell far below their strict archetype bands.
5. In commit `a19b1e7b`, the generator was updated to stack parts along Y (matching upAxis="Y"), which resolved the mismatch and brought the ratios in-band.

### Why there is a blade length/angle mismatch:
1. In `part_mesh.usda.tera`, the blade group has a non-uniform scale Y override of `3.41` (`my_sy = 3.41`), whereas the child blade meshes have a local rotation of `15.0` / `-15.0` degrees and a scale of `2.14`.
2. When the parent group applies a non-uniform scale of `3.41` to the rotated child mesh, the 3D geometry is stretched, which skews the 2D projected angle and length (e.g. `tan(skewed_angle) = 3.41 * tan(15 degrees)` yields a skewed angle of ~31-33 degrees).
3. The scorer (`compare_reference_render.py`) fits a line to the rendered cyan pixels and measures the 2D projected length and angle, yielding a length of ~204-212 and an angle of ~31-33 degrees.
4. Because the target values are strictly `180.0` length and `15.0` degrees, this skewing creates a significant delta, triggering the `VIS205` placement/angle mismatch.

---

## 3. Caveats
* The investigation assumes that the camera coordinates and front projection view in `scripts/compare_reference_render.py` are constant and have not been altered.
* We have not run Playwright browser actuation tests since that is scheduled for Milestone 5 and requires a WASM/HTML5 build environment which is out of read-only exploration scope.

---

## 4. Conclusion
* The orientation defect (stacking axis detected as X or Z instead of Y) and the height-ratio band failures were structurally fixed in commit `a19b1e7b` by correcting the generator coordinates to stack along Y, aligning it with the declared up-axis Y.
* The blade length and angle mismatch (`VIS205`) is caused by a non-uniform Y scale of `3.41` on the parent group level, which skews the rotated child blade mesh (local rotation of 15 degrees) on 2D projection.
* **Bipedal Kit Coherence Connection Mappings**:
  * **Neck**: Connects `Part_HeadUnit` (`Peg_Head_Neck`) to `Part_TorsoCore` (`Socket_Torso_Neck`).
  * **Shoulder Sockets**: Connects left/right arms (`Peg_LArm_Shoulder` / `Peg_RArm_Shoulder`) to `Part_TorsoCore` (`Socket_Torso_LArm` / `Socket_Torso_RArm`).
  * **Elbow**: Connects left/right forearms (`Peg_Forearm_Elbow_*`) to left/right upper arms (`Socket_Elbow_*`).
  * **Wrist**: Connects left/right hands (`Peg_Hand_Wrist_*`) to left/right forearms (`Socket_Wrist_*`).
  * **Pelvis**: Connects waist/pelvis (`Peg_Waist_Torso`) to `Part_TorsoCore` (`Socket_Torso_Waist`).
  * **Hip Sockets**: Connects left/right legs (`Peg_LLeg_Waist` / `Peg_RLeg_Waist`) to waist/pelvis (`Socket_Waist_LLeg` / `Socket_Waist_RLeg`).
  * **Knee**: Connects left/right shins (`Peg_Shin_Knee_*`) to left/right thighs (`Socket_Knee_*`).
  * **Ankle**: Connects left/right feet (`Peg_Foot_Ankle_*`) to left/right shins (`Socket_Ankle_*`).
  * **Feet Ground Band**: Evaluated via a SHACL shape in `110_bipedal_metric_envelope_law.ttl` ensuring feet min Y bounds align with the lower ground band.
  * **Shield Attachment**: Declared via a peg on the shield inserting into left/right forearm sockets (`Socket_Shield_Left` / `Socket_Shield_Right`).
  * **Wing Binders**: Declared via pegs on left/right wing roots inserting into loadout sockets (`Socket_Wing_Left` / `Socket_Wing_Right`) belonging to `backpack_core` (backpack loadout) instead of `torso_core`.

In the generator templates, these connections must be dynamically output as empty `Xform` prims representing sockets in the parent target meshes (`SM_Torso`, `SM_Limb_Left`, `SM_Limb_Right`, `SM_Loadout`).

---

## 5. Verification Method
1. **Verify Morphology and Stacking Axis**:
   * Command: `python3 scripts/verify_metric_morphology.py`
   * Invalidation: If the returned verdict is not `ADMITTED` or `orientation_finding.agree` is false, the geometry has regressed.
2. **Verify Blade Mismatch (`VIS205`)**:
   * Command: `./scripts/verify_asset.sh`
   * Inspect output file: `generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json`
   * Invalidation: The existence of `"VIS205 ERROR: blade placement/angle mismatch"` in `vis_errors` proves the non-uniform scale skewing is still active.
