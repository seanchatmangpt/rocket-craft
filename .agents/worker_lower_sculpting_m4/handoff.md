# Handoff Report: Milestone 4 (Lower Body & Limbs Geometry Candidate Patches)

## 1. Observation
- The mecha limbs (arms and legs) were originally defined as monolithic geometric cylinders (`mud:prim_arm_left`, `mud:prim_leg_left`, `mud:prim_arm_right`, `mud:prim_leg_right`) in `ontology/source_law/104_reference_fabric.ttl` (lines 556-594) translated at vertical Y = 0.0:
  ```ttl
  mud:prim_arm_left rdf:type mud:GeometryPrimitive ;
      mud:belongsToPart mud:arm_left ;
      mud:primitiveFamily "arm" ;
      mud:translateX -1.5 ; mud:translateY 0.0 ; mud:translateZ 0.5 ;
      mud:scaleX 1.0 ; mud:scaleY 1.0 ; mud:scaleZ 1.0 ;
  ```
- The limb rendering in `generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera` (lines 198-281) mapped `row.type == "arm"` and `row.type == "leg"` to basic cylinders and boxy armor shells.
- Sockets and pegs are defined logically in `ontology/source_law/118_limb_anatomical_joints.ttl` (e.g. `mud:Socket_Elbow_Left` and `mud:Peg_Forearm_Elbow_Left` using `mud:insertsIntoSocket`).
- Running `./scripts/verify_asset.sh` merges the Turtle files into `ontology/all_merged.ttl`, triggers `ggen sync` to generate the USD mesh definitions, performs pre-render metric morphology check, renders front/angled views, and scoring:
  ```bash
  >> [0/3] merge source_law -> all_merged.ttl
  >> [1/3] ggen sync
  >> [1b] pre-render metric morphology gate
  >> METRIC MORPHOLOGY: ADMITTED
  >> [2/3] delete stale renders + render_reference_fabric.py
  >> [3/3] compare_reference_render.py
  ```
- `METRIC_MORPHOLOGY_REPORT.md` lists the vertical bounds and ratio-band checks:
  ```markdown
  | part | y_min_m | y_max_m | height_m | ratio | band | in_band |
  |---|---|---|---|---|---|---|
  | SM_Limb_Left | -0.006 | 0.058 | 0.064 | 0.6305 | [0.6207, 0.6407] | True |
  ```
- The foot bottom is required to contact the ground band at Y = -0.6 (or -0.006m). Sockets must not smuggle geometry. All outputs must remain candidate geometry (`CANDIDATE_GEOMETRY`).

## 2. Logic Chain
- To segment the arms and legs, we replaced the monolithic cylinders in `104_reference_fabric.ttl` with 5 segmented geometry primitives each:
  - Left arm: `upper_arm`, `elbow`, `forearm`, `wrist`, `hand`
  - Right arm: `upper_arm`, `elbow`, `forearm`, `wrist`, `hand`
  - Left leg: `thigh`, `knee`, `shin`, `ankle`, `foot`
  - Right leg: `thigh`, `knee`, `shin`, `ankle`, `foot`
- The segments were stacked vertically along `translateY` (mapping to the USD vertical Y axis) such that the total length remains 1.0 for the arm and 1.2 for the leg:
  - Arm: Y ranges from -0.5 to 0.5 (segments: `upper_arm` scaleY=0.3, `elbow` scaleY=0.15, `forearm` scaleY=0.3, `wrist` scaleY=0.1, `hand` scaleY=0.15).
  - Leg: Y ranges from -0.6 to 0.6 (segments: `thigh` scaleY=0.5, `knee` scaleY=0.15, `shin` scaleY=0.4, `ankle` scaleY=0.08, `foot` scaleY=0.07). The foot Y bottom is at `-0.565 + (-0.5) * 0.07 = -0.6`, perfectly aligning with the lower body ground band.
- To represent joint sockets anatomically and avoid geometry smuggling (violating the SHACL `maxCount 0` rule of inverse `belongsToPart` for `mud:Socket`), we created empty Xform geometry primitives in `104_reference_fabric.ttl` with `primitiveFamily "socket"`. In `part_mesh.usda.tera`, if `row.type == "socket"`, it renders as `def Xform` without nested meshes.
- We updated `part_mesh.usda.tera` to support the rendering of the detailed subassemblies, defining custom meshes for each segment (e.g. claw claws for the hand, flat sole for the foot, and tapered guards for the forearm/shin).
- Running `./scripts/verify_asset.sh` compiles and validates the changes, confirming that:
  - `ggen sync` runs successfully.
  - Pre-render metric morphology gate conforms and outputs `ADMITTED`.
  - Bounding box calculation for the left/right limbs remains unchanged (`-0.006m` to `0.058m`), ensuring ratio-bands and bipedal legs continue to conform.
  - Sockets contain no mesh payloads.

## 3. Caveats
- No caveats. The physical foot alignment is verified to touch the ground band at exactly Y = -0.6 in USD.
- As requested, these changes are candidate geometry (`CANDIDATE_GEOMETRY`) only.

## 4. Conclusion
- The limbs are successfully sculpted from cylinders into detailed segments (upper arm, elbow, forearm, wrist, hand claw; thigh, knee, shin, ankle, foot) and empty attachment sockets.
- The foot bottom contacts the ground band at Y = -0.6. Sockets are free of smuggled geometry.
- All SHACL morphology validations conform and the pre-render gate passes.

## 5. Verification Method
- **SHACL Validation**:
  ```bash
  python3 validate_shacl.py
  ```
  Expected output: `Conforms: True`
- **Asset Compilation and Pre-Render Gate**:
  ```bash
  ./scripts/verify_asset.sh
  ```
  Expected output:
  - `>> METRIC MORPHOLOGY: ADMITTED`
  - In `METRIC_MORPHOLOGY_REPORT.md` / `METRIC_MORPHOLOGY_REPORT.json`, `SM_Limb_Left` and `SM_Limb_Right` must list `y_min_m` as `-0.006` (Y = -0.6) and `in_band` as `True`.
- **Mesh Inspection**:
  - In `generated/mech_assets/reference_fabric_001/usd/SM_Limb_Left.usda`, search for socket primitives (e.g., `def Xform "prim_socket_knee_left"`). Verify they contain no mesh or geometry definitions.
