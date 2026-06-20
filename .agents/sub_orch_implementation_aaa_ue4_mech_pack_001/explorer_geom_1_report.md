# Milestone 1: F1 Geometry & Morphology Exploration Report — FLAGSHIP_UE4_MECH_PLANT_001

## 1. Executive Summary

This report outlines the concrete implementation strategy for the **F1 Geometry & Morphology** milestone of `FLAGSHIP_UE4_MECH_PLANT_001`. Based on the baseline audits and target requirements, the previous asset suffered from morphological defects (parallel/barcode line wings and blocky core bodies), which led to quality gate refusals. 

To satisfy the F1 flagship requirements (`CTQ-F1-001` through `CTQ-F1-013` and `VJ-CRIT-001` through `VJ-CRIT-006`), we recommend transitioning from simple procedural primitives to a **hierarchical, multi-layered, and segmented hard-surface part grammar**. This document details recommendations for geometry representation, structural division, material lookdev integration, destruction states, locomotion animations, weapon variant loadouts, and verifier gates.

---

## 2. Geometry & Morphology Recommendations

### A. Layered Swept Feather Panels (Wings)
- **Gap Identified**: Previous wing geometry used flat, parallel arrays of box/line shapes resembling barcodes (`VIS203` and `VJ-CRIT-001` errors).
- **Target Specification**: We must generate at least 48 distinct feather panels per wing array (Primary + Secondary), layered to overlap and tapering towards the tip.
- **Mathematical Formulations for generator_parameters.ttl**:
  - **Translation Path (Progressive Curve)**: Splay the feathers along a quadratic bezier curve or elliptical arc. For feather index $i$ in $[0, N-1]$, let $t = i / (N-1)$:
    $$x_i = x_{start} \pm t \cdot \Delta x$$
    $$y_i = y_{start} - t \cdot \Delta y$$
    $$z_i = z_{start} + t \cdot \Delta z - (t^2) \cdot c_{curve}$$
  - **Length & Width Tapering (Length Gradient)**: Scale the feathers so they are longer and wider at the root, tapering down near the tip:
    $$s_{z, i} = z_{base} + z_{var} \cdot (1.0 - t)$$
    $$s_{x, i} = x_{base} - t \cdot x_{var}$$
  - **Thickness**: Enforce thickness (e.g., $s_{y, i} = 0.5 - t \cdot 0.25$) to maintain 3D volume instead of flat planes.
  - **Angular Fan-Out (Rotation Gradient)**: Rotate each panel along Y and Z axes to create a fan effect:
    $$r_{y, i} = \theta_{y, start} + t \cdot \Delta\theta_y$$
    $$r_{z, i} = \theta_{z, start} + t \cdot \Delta\theta_z$$
  - **Bevel and Overlap**: Shift translate Y by a alternating offset (e.g., $(-1)^i \cdot y_{offset}$) to force physical overlap in depth, preventing coplanar rendering (Z-fighting) and creating mechanical depth.
- **USDA Mesh Definition (`part_mesh.usda.tera`)**:
  - Implement a dedicated `feather_panel` mesh primitive in `part_mesh.usda.tera` that includes modeled bevels along the outer quad edge. The mesh must define 8 vertices forming a beveled quad plate with non-zero thickness, bevel angles, and quad index connectivity.

### B. Angular Armor Core Shell Hierarchy (Torso)
- **Gap Identified**: Core torso massing was overly compact or blob-like, failing cinematic silhouette requirements (`CTQ-F1-001`, `VJ-CRIT-004`).
- **Target Specification**: Torso must read as a layered assembly of distinct armor shells mounted onto an internal frame.
- **Structural Primitive Breakdown**:
  1. **Center Chest Plate**: Angular beveled box forming the primary frontal torso armor (WhiteArmor).
  2. **Pectoral Plates**: Symmetrical mirrored plates (Left/Right) angled outward using transform coordinates (WhiteArmor).
  3. **Upper Collar Guard**: A high neck-guard plate shielding the head transition (WhiteArmor).
  4. **Torso Side Flaps / Skirt Shields**: Mirror-rotated protective panels along the lower abdomen (WhiteArmor).
  5. **Internal Core Spine**: A thick cylindrical/box frame visible from the sides (DarkFrame).
  6. **Lower Abdominal Joint (exposed frame)**: exposed cylinder joints representing structural connections underneath armor (DarkFrame).
  7. **Rear Booster Mount**: A heavy plate interface connecting the backpack and thrusters (DarkFrame).
  8. **Chest Crest**: Symmetrical gold visor trim/crest representing insignia (GoldVisor).

### C. Torso/Head Distinctions
- **F1 Requirement**: Clear proportion boundaries. The torso volume must exceed head volume by a factor of 2.0 (`VIS206`).
- **Separation Strategy**:
  - The head and torso must occupy entirely separate files (`SM_Head.usda` and `SM_Torso.usda`).
  - Introduce a cylindrical neck joint primitive (`type = "cylinder"`, `mat = "M_DarkFrame"`) positioned at the torso-head boundary to physically separate the white armor of the torso collar from the white armor of the head shell.
  - Symmetrical V-fin crests (left/right) must be bound to the head USD file to define the classic mecha silhouette, but positioned to avoid bounding box overlap with torso or shoulder prims.

### D. Cyan Blade Rods
- **F1 Requirement**: Two long, high-emissive cyan blade rods placed symmetrically on the arms/shoulders (`CTQ-F1-009`, `VJ-CRIT-005`).
- **Symmetry & Angle**:
  - Place `blade_left` and `blade_right` at symmetrical X-axis coordinates (e.g., $X = \pm 25.0$, $Z = 75.0$).
  - Angle them slightly outward (e.g., $R_y = \pm 15.0$, $R_z = \mp 15.0$) to avoid collision with the legs or wings.
  - Length (scale Z) must be at least $40.0$ to ensure proper visual scale.
  - Materials: Must bind to `M_CyanBlade` which has high emissive inputs in MaterialX (`emission = 0.8`, `emission_color = 0.0 0.8 1.0`), producing the signature glowing energy effect.

---

## 3. Destruction, Animation & Loadout Integration Strategy

### A. Destruction States (CTQ-F1-008)
To support battle damage without manual model editing:
1. **Double-Layered Geometry**: Generate internal mechanical structures (using `M_DarkFrame`) under all major external armor plates (using `M_WhiteArmor`).
2. **Armor Break Variant**: Add a variant parameter in `ggen` templates or queries (e.g., `mud:DestructionState`). When activated, it:
   - Subtracts selected outer white armor plates.
   - Replaces them with fractured, smaller, and offset "debris" plate meshes (representing broken armor).
   - Exposes the internal cabling/frame primitives underneath.
3. **VFX Attachment Sockets**: Define dedicated empty coordinate transform scopes (e.g., `def Scope "Socket_VFX_Smoke"`) in the USDA templates at the boundaries of exposed frame structures. These sockets are targeted in-engine to spawn spark/fluid/smoke particles.

### B. Heavy Locomotion Animations (CTQ-F1-007)
We must structure skeletal rigging to support weight-driven locomotion:
1. **UsdSkel Skeletal Bindings**: Declare `SkelRoot` in `ASSET_ReferenceFabric_001.usda`. Structure the part files so their root `Xforms` act as skeletal joint transforms (`root -> torso -> neck -> head`, `torso -> shoulder_l -> arm_l`, etc.).
2. **Animation Cycles**:
   - **Idle Loop**: Slowly oscillate the torso translation along Z and rotation along X (breathing) at $1.5\text{ Hz}$ frequency.
   - **Walk Cycle**: Implement bipedal strides using joint rotation time-samples. Ensure weight-bearing legs scale down slightly under compression and torso moves in a wave pattern to simulate weight.
   - **Deploy State**: Rotate the wing roots forward/outward, slide the blade rails into hand sockets, and toggle the head visor emissive parameter from $0.0$ to $0.5$ progressively.

### C. Multiple Weapon Loadouts (CTQ-F1-009)
Define hand/arm sockets (`def Xform "Socket_Weapon_Left"`) inside `SM_Torso.usda` and `ASSET_ReferenceFabric_001.usda`:
1. Use **USD Variant Sets** inside `ASSET_ReferenceFabric_001.usda`:
   - `variantSet "loadout" = { "dual_blades", "heavy_cannon", "shield_shield" }`.
2. Under each variant, bind the reference of the weapon part (`SM_Blade_Left.usda`) to the respective arm socket transform. This prevents double assembly references and naming collisions.

---

## 4. Verification & QA Alignment (CTQ-F1-001 to 013, VJ-CRIT-001 to 006)

To ensure Milestone 1 passes all verification gates, the following alignment steps must be completed:

### A. Offline Test Alignment (`mecha_offline.test.ts`)
- **USD Identity Checks (USD301-307)**:
  - Fix absolute material path issues (`USD302` failure) by updating `part_mesh.usda.tera` to bind materials relative to the local part namespace, or compose them at the assembly level using USD over-rides, preventing sub-part files from referencing root assembly scopes.
  - Ensure all 7 USDA files are verified for unique fingerprints (`USD301`), expected roots (`USD304`), mirroring coordinate transform signs on the X-axis (`USD305`), and valid root unit metadata (`USD307`).
- **Morphology Metrics (VIS201-208)**:
  - Verify part-aware metrics in `visual_gap_report.json`:
    - `silhouette_iou` must be $\ge 0.25$.
    - `wing_feather_count` must be $\ge 48$ (Primary + Secondary).
    - `color_palette_similarity` must be $\ge 0.50$.
    - Torso core volume must be $\ge 2.0 \times$ head volume.
- **MaterialX Completeness**: Verify all 4 material files exist and define baseColor, roughness, metalness, and emission.

### B. E2E Playwright Walkthrough (Mecha & Gundam)
- **Mecha Walkthrough**: Succeeded at $388\text{px}$ visual delta (verdict: PASS). Keep current key sequence (W + Space).
- **Gundam Walkthrough**: Currently fails because actuated visual delta is $55\text{px}$ (under the $70\text{px}$ threshold). 
  - *Recommendation*: The movement actuation duration (8 seconds) is correct, but the camera position in the `barbarian-1` map is either obstructed or far from moving assets. Update the level transition script in `gundam_factory_walkthrough_projection.spec.ts` to either move the camera spawn forward or increase the viewer's run speed using engine console variables (`m.WalkSpeed` or `CharacterSpeed 600`) before movement key injection.
- **HTML5 General Pipeline**: Playwright fails to resolve `@noble/hashes/blake3` without the file extension.
  - *Recommendation*: Update import paths in `tests-e2e/tps-dflss.spec.ts` (and any other TypeScript files) to use `@noble/hashes/blake3.js` instead of `@noble/hashes/blake3` to satisfy ESM export constraints.

### C. AI Vision Judge Rubric Integration
- Ensure `ai_vision_judge_report.json` conforms to the schema:
  - Score $\ge 4.5$, verdict: `PASS`, disposition: `PASS_FLAGSHIP`.
  - Zero critical defects (`VJ-CRIT-001` through `VJ-CRIT-006` must show `status: PASS`).
  - If a defect occurs, verify that the verifier script halts the line and logs a negative fixture receipt for trace loops.

---

## 5. Concrete Action Plan for Implementation Track

We recommend executing the following steps in sequence to implement Milestone 1:

```
Step 1: Update TypeScript imports in tests-e2e/ to resolve blake3 path issues.
  └── Step 2: Use generate_ttl_morphology.py parameter progression algorithms to create generator_parameters.ttl.
        └── Step 3: Refine part_mesh.usda.tera to render beveled feather_panels instead of flat boxes.
              └── Step 4: Run validate_ontology.sh to ensure SHACL rules check out.
                    └── Step 5: Execute ggen sync to regenerate USDA and MaterialX files.
                          └── Step 6: Execute just verify-flagship-ue4-mech to run the E2E verification loop.
```

1. **Fix general HTML5 pipeline exports**: Update the import path of `@noble/hashes/blake3` to `@noble/hashes/blake3.js` in all spec files.
2. **Execute morphology parameter generator**: Run `python3 .agents/worker_reference_fabric_001_morphology/generate_ttl_morphology.py` to overwrite `generated/mech_assets/reference_fabric_001/graph/generator_parameters.ttl` with the conforming F1 geometry parameters (182 primitives, layered feathers, angular torso).
3. **Refine templates**: Update `part_mesh.usda.tera` to support quad plate geometry with bevel vectors.
4. **Regenerate asset files**: Run `validate_ontology.sh` and then `ggen sync` to generate the updated USDA and MaterialX files.
5. **Run test suite verification**: Run `npx vitest run mecha_offline.test.ts` to verify all Tiers 1-3 offline gates pass.
6. **Walkthrough verification**: Execute `./verify_mecha_pipeline.sh` and check that the Playwright walkthrough completes successfully, produces a signed receipt, and passes the qualitative AI Vision Judge check.
