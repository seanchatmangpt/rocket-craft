# F1 Geometry & Morphology Exploration Report — FLAGSHIP_UE4_MECH_PLANT_001

## Executive Summary
This report details the concrete implementation strategy for the geometry, morphology, and animation pipelines of `FLAGSHIP_UE4_MECH_PLANT_001`. By leveraging OpenUSD, MaterialX, and `ggen` compiler ontology constraints, the system ensures deterministic, artist-free asset generation that complies with F1 cinematic standards (CTQ-F1-001 through CTQ-F1-013) and qualitative vision requirements (VJ-CRIT-001 through VJ-CRIT-006).

---

## 1. Analysis of Current Codebase & Assets

We have investigated the following core components of the current mecha asset pipeline:

### A. Turtle Ontologies (`ontology/all_merged.ttl` & `generated/mech_assets/reference_fabric_001/graph/`)
1. **Namespace `mud`** (`https://rocket-craft.com/ontology/mud#`): Defines part grammar classes (`mud:MechPart`, subclasses like `mud:TorsoCore`, `mud:HeadUnit`, `mud:PrimaryWingFeathersLeft`, `mud:BladeLeft`) and instances representing the logical structure of the mecha.
2. **Geometry Primitive Families**: Currently supports `mud:TaperedBox`, `mud:FeatherPanel`, `mud:BladePrism`, and `mud:Cylinder` (originally maps to `cylinder`).
3. **Geometry Primitives**: Encompasses 192+ individuals (from `mud:prim_0001` to `mud:prim_0192`) specifying 3D translations, scales, rotations, and PBR material bindings.
4. **PBR Materials**: Defines `mud:M_WhiteArmor`, `mud:M_CyanBlade` (highly emissive/metallic), `mud:M_DarkFrame` (rough mechanical frame), and `mud:M_GoldVisor` (visor metallic finish).
5. **Visual Target Expectations**: Encodes measurements like `aspectRatio`, `symmetry`, and color channel proportions (white, dark, cyan, gold, red).

### B. SPARQL Queries
1. **`candidate_parts.rq`**: Recursively extracts all subclasses of `mud:MechPart` with explicit `ORDER BY ?part` to ensure compilation determinism.
2. **`usd_prims.rq`**: Selects all attributes of `mud:GeometryPrimitive` individuals (transforms, material bindings, local names) sorted by `?prim`.
3. **`verifier_expectations.rq`**: Selects the target proportions and bounding boxes of the reference mecha for AI Vision evaluation.

### C. GGen Templates
1. **`asset.usda.tera`**: Assembles the master OpenUSD assembly file (`ASSET_ReferenceFabric_001.usda`) by referencing sub-component files (`SM_Torso.usda`, `SM_Head.usda`, etc.) and declaring the `Materials` scope containing the mapped MaterialX shaders.
2. **`part_mesh.usda.tera`**: Translates extracted SPARQL geometry primitive rows into individual `Mesh` definitions. It defines macros to generate polygon coordinate index lists for primitive shapes (`tapered_box`, `feather_panel`, `blade_prism`, `cylinder`) and binds them to materials.
3. **`materials.mtlx.tera`**: Renders the MaterialX documents for lookdev.

### D. Offline Test (`pwa-staff/mecha_offline.test.ts`)
The test suite enforces the following validations:
* **USD Identity**: Asserts default prim fingerprint uniqueness (`USD301`), component encapsulation (`USD302`), non-containment of foreign components (`USD303`), mirroring coordinate checks (`USD305`), assembly references (`USD306`), and axis/units scale standard (`USD307`).
* **Morphology Metrics**: Asserts silhouette IoU (>= 0.25), edge similarity, wing feather count (>= 48), core compactness, cyan blade placement, and head-to-torso volume ratio (torso volume > 2x head volume).
* **MaterialX Completeness**: Validates baseColor, roughness, metalness, and emissive channels across lookdev documents.
* **UsdSkel Rigging & Sockets**: Asserts joints mapping, socket attachments, hierarchy structure, and VFX scope.
* **AI Vision Judge**: Evaluates rubric critical gates (`VJ-CRIT-001` through `VJ-CRIT-006`) with a threshold score >= 4.5.

---

## 2. F1 Flagship Requirements Strategy

The strategy to satisfy the F1 flagship gates is detailed below:

| Requirement ID | CTQ / VJ-CRIT Description | Implementation Strategy & Ontology Mapping |
| :--- | :--- | :--- |
| **CTQ-F1-001** | Cinematic Silhouette Complexity | Enforce a high density of parts and geometric primitives in the ontology. Target silhouette contour complexity through the use of curved, layered wing feathers and wing binder primitives. |
| **CTQ-F1-002** | Modular Part Identity | Utilize distinct classes in the ontology for each part (e.g. `mud:TorsoCore`, `mud:HeadUnit`). Populate USDA files via isolated, targeted SPARQL queries that bind `?CURRENT_PART_ID` to prevent duplicate geometry definitions in the assembly files. |
| **CTQ-F1-003** | Hard-Surface Detail Density | Populate greebles, bevels, and grooves as individual `GeometryPrimitive` instances. Define layered panels with overlapping Z-offsets to establish high visual depth. |
| **CTQ-F1-004** | PBR Channel Completeness | Automatically generate `.mtlx` MaterialX files with fully mapped metallic, roughness, normal, baseColor, and emissive textures, preventing untextured white blocks. |
| **CTQ-F1-005** | Material Variation Richness | Enforce multi-material assignments in the ontology. Inner structures use `M_DarkFrame`, primary shells use `M_WhiteArmor`, visors use `M_GoldVisor`, blades use `M_CyanBlade`, and joints/decals use micro accents (yellow/red). |
| **CTQ-F1-006** | Rig/Skeleton/Socket Completeness | Encode joint bounds and IK chain constraints inside the Turtle ontology. Render them into `UsdSkel` schema definitions in the master USD file, linking limb meshes to skeleton bones. |
| **CTQ-F1-007** | Heavy Animation Coverage | Establish a motion state machine in the ontology containing `walk`, `idle`, and `deploy` states. The gait/motion curves are compiled as skeleton transform keys in the USDA files. |
| **CTQ-F1-008** | Destruction-State Coverage | Define destruction variants (`mud:destructionState`) in the ontology. Outer armor plates can be toggled hidden or replaced by fractured mesh coordinate variants to expose the underlying dark frame. |
| **CTQ-F1-009** | Multiple Loadout Support | Utilize USD `variantSets` in the assembly template. Weapon socket nodes mount weapon references based on the selected active variant (e.g., standard vs heavy melee loadout). |
| **CTQ-F1-010** | UE4 Import/Cook Proof | Automate USD/FBX import using command-line scripts. Hook into the `package-brm-html5.sh` execution to ensure cooked packages exist and load without blueprint errors. |
| **CTQ-F1-011** | In-Engine Presentation Proof | Headless offscreen renders via `usdrecord` and local Playwright screenshots computed against a baseline to confirm the presence of high-fidelity WebGL pixels. |
| **CTQ-F1-012** | IP-Distance Non-Confusion | Define non-confusion trademark shapes in the ontology. Calculate the Jaccard distance/difference score against copyrighted silhouettes to guarantee `d(x, P) > tau`. |
| **CTQ-F1-013** | Receipt/Replay Proof | Hash each compiled asset into a sequential, tamper-evident cryptographic receipt chain (`asset_receipts.jsonl`) logged to the database. |
| **VJ-CRIT-001** | Silhouette lacks flagship authority | Refine coordinate curves for swept wing feathers. Adjust parameters until silhouette IoU >= 0.25 is achieved. |
| **VJ-CRIT-002** | Hard-surface detail below threshold | Ensure beveled panel meshes are used for outer armor instead of flat flat boxes. |
| **VJ-CRIT-003** | Material response not cinematic/PBR | Map roughness and metallic textures dynamically with wear/dust masks to avoid artificial plastic looks. |
| **VJ-CRIT-004** | Part hierarchy reads as primitive/proxy | Establish clear joint and pivot offsets in the ontology, avoiding overlapping intersections that break structural plausibility. |
| **VJ-CRIT-005** | Destruction/loadout integration absent | Define clean sockets for weapon attachments and damage zones with local coordinate bounds. |
| **VJ-CRIT-006** | UE4 presentation fails flagship standard | Resolve cooking warnings and compile errors. Standardize viewport camera settings during automated testing. |

---

## 3. Concrete Feature Implementation Strategy

### A. Swept Feather Panels
To achieve a high-complexity silhouette and satisfy the minimum 48 wing-feather panel requirement (`VIS202`):
1. **Mathematical Sweep Curves**: Do not manually place panels. In the generator scripts, define a quadratic Bezier curve or trigonometric sweep function in 3D space:
   $$P(t) = (1-t)^2 P_0 + 2(1-t)t P_1 + t^2 P_2$$
   where $P_0, P_1, P_2$ represent the wing root, midpoint, and tip coordinates.
2. **Ontology Generation**: Sample the curve at regular intervals to generate individual `GeometryPrimitive` triples with translating and scaling parameters (e.g., feathers become smaller and rotate outwards towards the tips).
3. **Mirroring Transforms**: Ensure left and right wings are perfect mirrored pairs. The right wing's primitives mirror the X translation and Z rotation relative to the left:
   $$X_{right} = -X_{left}, \quad Rz_{right} = -Rz_{left}$$
   This satisfies `USD305` constraints.

### B. Angular Armor Core Shell Hierarchy
To create the high-density armor shelling (`CTQ-F1-003`, `VJ-CRIT-002`):
1. **Parent-Child Structural Mapping**: In `asset_fabric.ttl`, define a property `mud:mountedOn` linking armor primitives to their structural frame primitives:
   ```turtle
   mud:prim_armor_chest_left rdf:type mud:GeometryPrimitive ;
       mud:belongsToPart mud:torso_core ;
       mud:mountedOn mud:prim_frame_chest ;
       mud:primitiveFamily "beveled_panel" ;
       mud:materialBinding mud:M_WhiteArmor .
   ```
2. **Beveled Panels**: Introduce a `beveled_panel` primitive mesh family in `part_mesh.usda.tera` to yield crisp highlight edges.
3. **Overlapping Shell Layering**: Stack plates sequentially with minute normal-direction offsets (e.g., 0.1cm gaps) to form structural panel lines. The dark frame underneath should peep through these gaps (`M_DarkFrame` material), creating shadow-rich details.

### C. Torso/Head Distinctions
To enforce volume ratios and structural proportions (`VIS206`):
1. **Volume Ratio Validation Rule**: Define a SHACL shape or SPARQL constraint that multiplies the 3D bounds ($ScaleX \times ScaleY \times ScaleZ$) of all primitives belonging to the `mud:TorsoCore` family, asserting it is at least double that of the `mud:HeadUnit` family:
   $$\sum Vol_{Torso} \ge 2.0 \times \sum Vol_{Head}$$
2. **Antenna Layout**: Attach `mud:v_fin_left` and `mud:v_fin_right` parts directly to `mud:head_unit` coordinates. They should sweep upwards and outwards to define the iconic flagship profile.

### D. Cyan Blade Rods
To satisfy `VIS205` and `VJ-CRIT-003`:
1. **Mesh Definitions**: Use `blade_prism` geometry primitive family (triangular prism) representing sharp energy blades.
2. **Emissive Lookdev Binding**: Bind the blade meshes to `M_CyanBlade`. In `asset.usda.tera`, the shader for `M_CyanBlade` maps emissive inputs:
   ```usd
   float inputs:emission = 0.8
   color3f inputs:emission_color = (0.0, 0.8, 1.0)
   float inputs:metallic = 0.9
   float inputs:roughness = 0.15
   ```
3. **Attachment Sockets**: Place the blades at the coordinates of the weapon attachment sockets on `mud:arm_left` and `mud:arm_right`.

### E. Destruction States
To satisfy `CTQ-F1-008` and `VJ-CRIT-005`:
1. **State Ontology**: Introduce `mud:destructionState` (e.g. `mud:StateNormal`, `mud:StateDamaged`, `mud:StateBroken`) in the ontology.
2. **USD Layer / Variant Sets**: Implement a USD `variantSet` called `destruction_state` in `ASSET_ReferenceFabric_001.usda`:
   ```usd
   variantSets = ["destruction_state"]
   variants = {
       "destruction_state" = {
           "normal" {
               # Standard armor geometry visible
           }
           "damaged" {
               # Some M_WhiteArmor panels hidden, exposing M_DarkFrame
           }
           "broken" {
               # Replaces beveled armor meshes with fractured mesh variants
           }
       }
   }
   ```
3. **VFX damage sockets**: Add empty transform scopes `Scope "VFX_Sockets"` (e.g. `socket_damage_torso`, `socket_damage_l_wing`) located at the coordinate intersections of the armor shells to trigger particle effects in Unreal.

### F. Heavy Animations (Idle, Walk, Deploy)
To support skeletal motion and playback clearance (`CTQ-F1-007`):
1. **SkelAnim Schema mapping**: Declare a UsdSkel hierarchy mapping torso, head, shoulder, arm, leg, and wing bones.
2. **Animation curve parameters**: Idle utilizes minor rotational sine waves on the torso and shoulder joints to simulate breathing. Walk uses alternating hip and knee transformations. Deploy expands the wings by increasing coordinate rotation angles on the wing binder joints.
3. **SIMD Gait Verification Kernel**: Implement a compiler-validated SIMD gait kernel in Rust that reads joint angles and checks bounding box clearance to guarantee feet do not clip into the torso during motion sweeps, verifying safety at compile-time.

### G. Weapon Loadouts
To support multiple weapon variations (`CTQ-F1-009`):
1. **Loadout triples**: Map weapon configurations in Turtle:
   ```turtle
   mud:LoadoutStandard mud:hasLeftWeapon mud:blade_left ; mud:hasRightWeapon mud:blade_right .
   mud:LoadoutHeavyMelee mud:hasLeftWeapon mud:heavy_mace_left .
   ```
2. **USD Variant Sets for Loadouts**: Use `variantSet "loadout"` in the master assembly. Swapping the active variant updates the weapon references without requiring runtime code changes.

---

## 4. Proposed Architecture & Workflow

The fully automated F1 mecha manufacturing pipeline is outlined below:

```
[Turtle Ontology] (all_merged.ttl)
      │
      ▼ (Quality Gates / SHACL check)
 [ggen sync] ───────────────────────► [OpenUSD Primitives & MaterialX]
      │                                   (USDA meshes, .mtlx files)
      ▼                                       │
 [Rust Telemetry Dictionary]                  ▼ (usdrecord offscreen render)
      │                             [Headless PNG Renders]
      ▼ (Unreal Import Command)               │
 [UE4 / HTML5 WASM Engine]                    ▼ (compare_reference_render.py)
      │                             [AI Vision Judge Report]
      ▼ (Playwright E2E Walkthrough)          │
 [BLAKE3 Signed Receipt] ◄────────────────────┘ (Verify verdict == PASS)
```

### Steps for Implementation
1. **Define/Refine Ontology**: Define the 192+ primitives, wings, armor, and joint transformations in `all_merged.ttl`.
2. **Run compiler**: Execute `/Users/sac/.local/bin/ggen sync` to generate the `.usda` meshes, lookdev `.mtlx` materials, and `texture_program.rs` programs.
3. **Generate Textures and Renders**: Execute `scripts/render_reference_fabric.py` to headlessly generate PBR texture maps and render the 4 views (`render_front.png`, `render_angled.png`, `render_silhouette.png`, `render_edges.png`) using `usdrecord`.
4. **Evaluate Metrics**: Run `python3 scripts/compare_reference_render.py` to calculate silhouette similarity and emit `ai_vision_judge_report.json`.
5. **Stage and Serve**: Stage the compiled packages, spin up the HTTP serve daemon, and launch the E2E verification tests (`verify_mecha_pipeline.sh`).
6. **Chain Receipts**: The verifier seals the execution details, logs hashes, and outputs the signed `mecha-playwright-receipt.json`.
