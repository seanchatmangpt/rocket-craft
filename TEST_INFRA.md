# F1-Grade Cinematic Mecha E2E Testing Infrastructure (TEST_INFRA)

## 1. Introduction & Objectives
This document establishes the end-to-end (E2E) testing and validation framework for the F1-grade mecha production assets under target `FLAGSHIP_UE4_MECH_PLANT_001` (GC-AAA-UE4-MECH-001). Elevated from a toy-grade asset, this mecha represents a flagship cinematic production asset (valued at $2M–$5M) with fully integrated geometry, high-fidelity materials, rigging, heavy animation cycles, destruction states, multiple loadouts, and strict IP distance non-confusion verification.

The final authority of this system is governed by the **TPS/DfLSS Playwright Manufacturing Strategy**. A generated mech is not admitted unless it passes the F1 Admission command `just verify-flagship-ue4-mech`, which executes offline structural checks, local database persistence validation, and automated browser-native WebGL walkthroughs.

---

## 2. The 4-Tier Acceptance Testing Methodology

To guarantee flagship-level quality and compliance, the testing suite is structured into four distinct coverage tiers:

```
┌─────────────────────────────────────────────────────────────────┐
│              Tier 4: Real-World Walkthrough Scenarios           │
│         (Playwright browser walkthrough, input actuation, delta)│
└────────────────────────────────┬────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│           Tier 3: Cross-Feature Interactions                    │
│      (Materials bound to wings, sockets on skeletal joints, etc.)│
└────────────────────────────────┬────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│           Tier 2: Boundary & Corner Cases                       │
│     (Empty meshes, duplicate fingerprints, bounding overlaps, etc.)│
└────────────────────────────────┬────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│           Tier 1: Feature Coverage (CTQ-F1 Gates)               │
│       (Modular USD, morphology, MaterialX, cook, IP distance, etc.)│
└─────────────────────────────────────────────────────────────────┘
```

### Tier 1: Feature Coverage (CTQ-F1 Gates)
Validates the presence, structure, and correctness of core mecha asset features across the 13 F1 Gates. Every feature must have at least five distinct, programmatically verified test cases.

#### Gate 1: Cinematic Silhouette Complexity (CTQ-F1-001)
1. **Silhouette IoU Threshold** - Verify `silhouette_iou >= 0.25` in visual gap reports.
2. **Edge Similarity Check** - Verify edge similarity metrics are non-zero.
3. **Wing Span Check** - Verify wing span delta is within the acceptable range.
4. **Mass/Volume Verification** - Verify body mass delta is within the tolerance bounds.
5. **Component Count** - Verify at least 100+ primitives exist.

#### Gate 2: Modular Part Identity (CTQ-F1-002 / USD301-307)
1. **USD301: Unique Fingerprints** - Verify that each USD part contains a unique fingerprint or docstring preventing asset confusion.
2. **USD302: Component Isolation** - Verify that individual part USDs do not reference or render the full assembly.
3. **USD303: Foreign Component Prevention** - Verify that parts contain only their own internal meshes and no foreign components.
4. **USD304: Expected Roots** - Verify that the expected assembly root and component Xforms are present in the USD files.
5. **USD305: Mirror Coordinate Transforms** - Verify that mirrored parts (e.g., left and right wings/blades) contain appropriate mirroring transforms.
6. **USD306: Composition References** - Verify that the master assembly file correctly references component USD files.
7. **USD307: Root Metadata** - Verify that defaultPrim, upAxis, and metersPerUnit metadata are valid.

#### Gate 3: Hard-Surface Detail Density (CTQ-F1-003)
1. **High Prim Count** - Ensure total assembly prim count is >= 1000.
2. **Symmetry Delta Verification** - Verify symmetry delta is below 0.05.
3. **LOD Segmentation Depth** - Verify mesh segmentation levels of detail.
4. **Joint Structure Definition** - Verify joints have detailed hard-surface constraints.
5. **Panel Gap Variance** - Verify armor shell panel segmentation depth variance is non-zero.

#### Gate 4: PBR Channel Completeness (CTQ-F1-004)
1. **BaseColor Definition** - Verify BaseColor inputs are defined across all materials.
2. **Roughness Definition** - Verify roughness and specular roughness are defined.
3. **Metallic Definition** - Verify metalness values are present.
4. **Ambient Occlusion Channel** - Verify ambient occlusion maps are defined.
5. **Emissive Channels** - Verify emissive inputs are present for emissive materials (`M_CyanBlade`, `M_GoldVisor`).
6. **Wear / Decal Channels** - Verify Wear and Decal mask presence or support.

#### Gate 5: Material Variation Richness (CTQ-F1-005)
1. **4K/8K Texture Mapping Policy** - Verify texture manifest references high-resolution textures.
2. **4 Unique Material Families** - Verify all 4 mecha materials exist (`M_CyanBlade`, `M_DarkFrame`, `M_GoldVisor`, `M_WhiteArmor`).
3. **Emissive Color Space** - Verify emissive colors map to valid RGB coordinates.
4. **Material Bindings Count** - Verify that the number of material bindings is >= 1000.
5. **Wear & Decal Layering** - Verify the presence of wear shaders in the MTLX network.

#### Gate 6: Rig/Skeleton/Socket Completeness (CTQ-F1-006)
1. **Skeletal Joints Mapping** - Verify skeleton joint names exist and map correctly.
2. **Socket Attachment** - Verify sockets are declared and bound to appropriate skeletal joints.
3. **Rigging Hierarchy** - Verify joints establish a valid hierarchy.
4. **VFX Sockets** - Verify specialized sockets for visual effects (e.g., muzzle flash, thruster flares) are defined.
5. **IK Targets** - Verify Inverse Kinematics targets are mapped in the rigging structure.

#### Gate 7: Heavy Animation Coverage (CTQ-F1-007)
1. **Idle Cycle** - Verify heavy idle animations are bound.
2. **Walk Cycle** - Verify walking animation states.
3. **Deploy Cycle** - Verify weapon/shield deploy state animation curves.
4. **Root Motion Configuration** - Verify root motion is enabled for movement states.
5. **Blendspace Bounds** - Verify animation blending limits.

#### Gate 8: Destruction-State Coverage (CTQ-F1-008)
1. **Broken Armor States** - Verify damaged armor meshes exist or are defined.
2. **Exposed Frames** - Verify exposed internal frame components.
3. **Thruster VFX Sockets** - Verify thruster sockets exist on wing arrays.
4. **Exposed Wiring/Cabling** - Verify exposed structural details.
5. **Damage Zones Mapping** - Verify damage zones are linked to skeletal joints.

#### Gate 9: Multiple Loadout Support (CTQ-F1-009)
1. **Weapon Attachments** - Verify blade attachments (`SM_Blade_Left`, `SM_Blade_Right`).
2. **Shield Socket Attachments** - Verify shield sockets.
3. **Loadout Config Files** - Verify multiple loadout configurations are parsed.
4. **Weapon Sockets Map** - Verify weapons map to correct hand/wing sockets.
5. **Attachment Scale Policies** - Verify scale rules for attachments.

#### Gate 10: UE4 Import/Cook Proof (CTQ-F1-010)
1. **Cook Receipt Presence** - Verify `cook-receipt.json` exists in the build output.
2. **Cook Verdict PASS** - Verify that the cook verdict is explicitly `PASS`.
3. **Companion Files** - Verify HTML, JS, and data companions exist.
4. **Unreal Package Size Check** - Verify package sizes are non-zero.
5. **UI Input Patched Status** - Verify UI input patching is enabled.

#### Gate 11: In-Engine Presentation Proof (CTQ-F1-011)
1. **Canvas Rendering** - Verify Playwright can find and focus the rendering canvas.
2. **Engine Readiness** - Verify engine calledMain is true.
3. **In-Engine Screenshot Capture** - Capture screenshots before and after movement.
4. **Actuated Motion Delta** - Verify visual delta is above background threshold.
5. **Non-Black Pixels Check** - Verify > 1000 non-black pixels are rendered.

#### Gate 12: IP-Distance/Non-Confusion Proof (CTQ-F1-012)
1. **Distance Metric `d(x, P) > tau`** - Verify admissibility of the manufactured asset against the reference dataset.
2. **Rejection Bounds** - Verify mutated/falsification cases are correctly rejected (`REFUSED`) by the validation engine.
3. **Falsification Suite Passes** - Ensure at least 8 falsification cases successfully pass.
4. **Counterfactual Suite Passes** - Ensure at least 8 counterfactual cases successfully pass.
5. **Gap Closure Total** - Verify all 19 checks pass in the gap closure report.

#### Gate 13: Receipt/Replay Proof (CTQ-F1-013)
1. **BLAKE3 Receipt Chain** - Verify that the receipts log contains a sequential, chained cryptographic sequence.
2. **Hash Continuity** - Verify that `prev_hash` of sequence `N` matches the `receipt` hash of sequence `N-1`.
3. **Signed Walkthrough Receipt** - Verify Playwright walkthrough outputs a signed json receipt.
4. **Supabase Persistence** - Verify mecha receipt is successfully written to the database.
5. **BLAKE3 Manifest Verification** - Verify BLAKE3 hashes match on-disk file states.

---

### Tier 2: Boundary & Corner Cases
Enforces structural rules, quality constraints, and extreme inputs.
1. **Empty Meshes** - Verify that mesh files with zero vertices or points are rejected.
2. **Duplicate Fingerprints** - Verify that duplicate asset fingerprints trigger confusion errors.
3. **Bounding Box Overlaps** - Verify bounding box intersections between distinct components are within valid margins.
4. **V-fin IP Proximities** - Verify that V-fin antenna distances do not collide.
5. **Zero-Volume Components** - Verify that components have non-zero volume/mass.

---

### Tier 3: Cross-Feature Interactions
Validates interaction between distinct pipeline components.
1. **Materials Bound to Wing Feathers** - Verify that MaterialX materials are bound to the specific mesh primitives of wing arrays.
2. **Sockets Attached to Skeleton Joints** - Verify sockets reference valid skeletal joints in the USD graph.
3. **Walkthrough Event Telemetry** - Verify walkthrough telemetry events contain matching keys.

---

### Tier 4: Real-World Walkthrough Scenarios
Playwright loads the mecha asset viewer, focuses the WebGL canvas, injects movement inputs (holding `W` and `Space` for 8 seconds), captures screenshots, computes visual movement delta, writes a BLAKE3 signed receipt to `test-results/mecha-playwright-receipt.json`, and verifies persistence to Supabase.

---

## 3. Playwright Manufacturing Strategy Alignment

The 4-tier acceptance methodology directly feeds into the **Playwright Manufacturing Strategy**:
1. **Gate 0 (Source Admission):** Validated by `mecha_offline.test.ts` (Vitest).
2. **Gate 1-3 (Packaging Admission):** `cook-receipt.json` verification.
3. **Gate 4-7 (Runtime and Verification):** Playwright automated browser walkthrough with movement delta computation.

---

## 4. Repair Routing Taxonomy

If the E2E testing pipeline detects a failure, the pipeline routes the defect to:
- **Design Engineering Cell**: For silhouette or IP-distance violations.
- **Chassis / Hard-Surface Cell**: For modular USD assembly or joint failures.
- **Surface Engineering Cell**: For MaterialX, texture policy, or decal failures.
- **Rig and Motion Cell**: For skeleton, socket, or animation coverage failures.
- **Destruction Cell**: For damage zone or broken armor failures.
- **UE4 Integration Cell**: For import, cook, or packaging failures.
- **Verifier / Race-Control Cell**: For receipt signing, hash continuity, or database persistence failures.

---

## 5. F1 Admission Command

To execute the entire verification pipeline, run:

```bash
just verify-flagship-ue4-mech
```

A non-zero exit code indicates a validation defect. Detailed outputs are printed directly to `stdout` and `stderr`.

---

## 6. Qualitative AI Vision Judge Cell Check

The E2E testing infrastructure integrates the qualitative AI Vision Judge check to verify cinematic and AAA-production grade standards of the generated assets under `FLAGSHIP_UE4_MECH_PLANT_001`. A metric pass (IoU, mass) without a visual judge pass is refused.

### The Rubric Namespace (VJ001-VJ012)
- **VJ001**: Silhouette authority (matches silhouette precisely)
- **VJ002**: Hard-surface sophistication (intricate panelling and bevel details)
- **VJ003**: Part hierarchy clarity (clear visual relationships between chassis, limbs, and joints)
- **VJ004**: Armor layering quality (layering shows mechanical depth, no intersections)
- **VJ005**: Material richness (cinematic, high-fidelity MaterialX inputs)
- **VJ006**: Texture/decal density (texture and decals are distributed logically)
- **VJ007**: Mechanical plausibility (joint connections and structural logic make physical sense)
- **VJ008**: Loadout integration (weapons and auxiliary parts attach cleanly to sockets)
- **VJ009**: Destruction-state quality (damaged meshes exhibit realistic material fracturing)
- **VJ010**: Animation-pose credibility (poses convey weight and power in keyframes)
- **VJ011**: UE4 in-engine presentation (perfect presentation inside UE4 viewport)
- **VJ012**: Flagship/cinematic impression (stunning cinematic key-art impression)

### Qualitative Evaluation Criteria
Qualitative evaluations are conducted to determine admission and disposition based on whether the mecha asset exhibits any defects from the defect taxonomy.

### Updated Binary Disposition and Defect Taxonomy (VJ-CRIT)
In accordance with the corrected DfLSS rules, the judge emits binary classifications and a structured defect taxonomy:
- **VJ-CRIT-001**: Silhouette lacks flagship authority (routes to morphology ggen rule revision)
- **VJ-CRIT-002**: Hard-surface detail below production threshold (routes to chassis/hard-surface cell)
- **VJ-CRIT-003**: Material response not cinematic/PBR-rich (routes to surface engineering cell)
- **VJ-CRIT-004**: Part hierarchy reads as primitive/proxy (routes to chassis cell)
- **VJ-CRIT-005**: Destruction/loadout integration absent or toy-like (routes to destruction/loadout cells)
- **VJ-CRIT-006**: UE4 presentation fails flagship standard (routes to UE4 integration cell)

### Disposition Rules
- `PASS_FLAGSHIP`: All critical CTQs pass; no unresolved major defects.
- `REFUSE_NON_FLAGSHIP`: Fails cinematic/AAA/F1 visual bar.
- `REFUSE_TECHNICAL`: Broken geometry, rig, material, UE4 import, etc.
- `REFUSE_IP_RISK`: Fails IP-distance / non-confusion gate.
- `HOLD_FOR_ROOT_CAUSE`: Measurement conflict or unclear defect source.
- `REPLAY_REQUIRED`: Looks acceptable but process proof missing.

Any critical visual defect will stop the line and trigger a pipeline exit code of 1.

### Example Passing Evaluation Report (`ai_vision_judge_report.json`)
```json
{
  "asset_id": "reference_fabric_001",
  "disposition": "PASS_FLAGSHIP",
  "critical_defects": [],
  "major_defects": [],
  "minor_defects": [],
  "admission": true
}
```

### Verification Command
To verify the AI Vision Judge report file:
```bash
./verify_mecha_pipeline.sh
```
This script checks the existence of mecha proof images, searches for `ai_vision_judge_report.json`, prompts for admission confirmation if running interactively, and asserts the strict schema conformance, disposition, and critical defects constraints.


