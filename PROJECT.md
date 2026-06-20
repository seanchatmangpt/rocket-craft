# Project: F1-Grade Cinematic Mecha Manufacturing Plant
(Target: FLAGSHIP_UE4_MECH_PLANT_001, Milestone ID: GC-FLAGSHIP-UE4-MECH-F1)

## Core Doctrine: Cinematic Production Standards
Every generated mecha asset must meet the standards of a high-fidelity cinematic production asset, elevating the quality from standard game-ready to a $2M-$5M production bar. The asset must exhibit hero-level hard-surface geometry, complex silhouettes, rich PBR material variation (using 4K/8K textures), dynamic skeleton rigging with weapon/VFX sockets, heavy locomotion animations, dedicated destruction states (broken armor panels, exposed inner frame structures, and dedicated VFX debris sockets), multiple loadout assemblies, and automated engine integration with verification. Any asset failing to meet these gates is rejected as scrap.

## Architecture
The pipeline is structured around 7 parallel, specialized manufacturing cells that feed into the Verifier / Race-Control gate before admitting the asset into the final production registry:

1. **Design Engineering Cell**: Owns semantic role grammar, silhouette class composition, and original IP axes definitions.
2. **Chassis / Hard-Surface Cell**: Generates the primary internal framework, structural joints, external armor shelling, beveling, and panel segmentation.
3. **Surface Engineering Cell**: Manages material family definitions, OpenPBR Lookdev, wear/dirt generators, decals, and emissive channels under a 4K/8K texture policy.
4. **Rig and Motion Cell**: Manufactures joint hierarchies, skeletal weight mappings, IK target anchors, and heavy cinematic locomotion animations (`idle`, `walk`, `deploy`).
5. **Destruction Cell**: Projects battle-damaged states including broken armor segments, exposed inner frame meshes, fractured plate surfaces, and dedicated VFX debris attachment sockets.
6. **UE4 Integration Cell**: Automates FBX conversion, command-line asset import, DataTable material bindings, collision hull generation, LOD creation, and headless engine cooking.
7. **Verifier / Race-Control Cell**: Executes the complete 13 CTQ-F1 Gates verification suite. It acts as the final gatekeeper, validating geometry, material completeness, rigging, animation loops, destruction physics, IP distance, and receipts.

```
                  ┌────────────────────────────────────────────────────────┐
                  │                 GC-FLAGSHIP-UE4-MECH-F1                │
                  │             FLAGSHIP_UE4_MECH_PLANT_001                │
                  └───────────────────────────┬────────────────────────────┘
                                              │
         ┌────────────────────────────────────┴────────────────────────────────────┐
         ▼                                                                         ▼
  =================================                                         =================================
       MANUFACTURING CELLS                                                       VERIFICATION & QUALITY
  =================================                                         =================================
         │                                                                         │
         ▼                                                                         ▼
  ┌───────────────────────────────┐                                         ┌───────────────────────────────┐
  │  1. Design Engineering Cell   │                                         │  CTQ-F1-001 to CTQ-F1-005      │
  ├───────────────────────────────┤                                         │  (Silhouette, Geometry, PBR)  │
  │  2. Chassis & Hard-Surface    │                                         └──────────────┬────────────────┘
  ├───────────────────────────────┤                                                        │
  │  3. Surface Engineering Cell  │                                                        ▼
  ├───────────────────────────────┤                                         ┌───────────────────────────────┐
  │  4. Rig and Motion Cell       │                                         │  CTQ-F1-006 to CTQ-F1-009      │
  ├───────────────────────────────┤                                         │  (Rig, Anim, Destruction)     │
  │  5. Destruction Cell          │                                         └──────────────┬────────────────┘
  ├───────────────────────────────┤                                                        │
  │  6. UE4 Integration Cell      │ ─────── Cooked Assets ────────────────────────┐        │
  └───────────────────────────────┘                                               │        ▼
                                                                                  │ ┌───────────────────────────────┐
                                                                                  │ │  CTQ-F1-010 to CTQ-F1-013      │
                                                                                  │ │  (UE4 Cook, IP, Receipts)      │
                                                                                  │ └──────────────┬────────────────┘
                                                                                  ▼                ▼
                                                                           ┌────────────────────────────────────────┐
                                                                           │      7. Verifier / Race-Control Cell   │
                                                                           │      Command: just verify-flagship...   │
                                                                           └────────────────────────────────────────┘
```

## The 13 Critical-to-Quality F1 Gates (CTQ-F1)
Every generated asset must pass all 13 gates to be admitted:

- **CTQ-F1-001: Cinematic Silhouette Complexity**
  Evaluates silhouette richness and complexity across multiple angles. Restricts simple or primitive massings, enforcing high-fidelity structural complexity.
- **CTQ-F1-002: Modular Part Identity**
  Enforces OpenUSD composition laws. Every component (Head, Torso, Limbs, Wings, Blades) must exist as a structurally isolated USD file with unique geometry fingerprints (no duplicates).
- **CTQ-F1-003: Hard-Surface Detail Density**
  Verifies the presence of panel bevels, seam lines, greeble components, and structural segmentation on armor shells. Enforces micro-detail density suitable for cinematic close-ups.
- **CTQ-F1-004: PBR Channel Completeness**
  Ensures 100% complete channel maps (BaseColor, Normal, Roughness, Metallic, Ambient Occlusion, Emissive, and Wear/Decal masks) using the OpenPBR standard and a 4K/8K texture policy.
- **CTQ-F1-005: Material Variation Richness**
  Requires rich color and material blocking (e.g., pristine white ceramic armor, dark structural steel, cyan emissive accents, gold visor trim, and subtle red micro-decals).
- **CTQ-F1-006: Rig/Skeleton/Socket Completeness**
  Validates joint hierarchies, joint limits, IK target anchors, and dynamic weapon/equipment socket attachments via UsdSkel schemas.
- **CTQ-F1-007: Heavy Animation Coverage**
  Requires full animation coverage for primary locomotion states:
  - `idle`: Stable breathing loop, shifting center of mass.
  - `walk`: Heavy, high-inertia bipedal stride with simulated foot-planting and weight transfer.
  - `deploy`: Initialization sequence, unfolding weapon rails, weapon mounting, and visor startup.
- **CTQ-F1-008: Destruction-State Coverage**
  Requires dedicated damaged states:
  - `broken armor`: Armor shells fractured into debris.
  - `exposed frames`: Exposed internal cabling, gears, and structural frames.
  - `VFX sockets`: Sockets for sparks, smoke, and mechanical fluid leak particles.
- **CTQ-F1-009: Multiple Loadout Support**
  Verifies that the modular assembly can swap weapon systems, shield generators, and auxiliary propulsion units dynamically without structural or naming collisions.
- **CTQ-F1-010: UE4 Import/Cook Proof**
  Automates headless import of FBX and material bindings into Unreal Engine 4 using Python/DataTable API. Ensures 100% successful compile/cook under the target build profile.
- **CTQ-F1-011: In-Engine Presentation Proof**
  Playwright-driven verification in browser-native UE4 HTML5/WASM runtime. Captures baseline, actuate keyboard inputs, and verifies visual delta movements.
- **CTQ-F1-012: IP-Distance/Non-Confusion Proof**
  Mathematical verification that the asset is outside protected trademark centroids (e.g., Gundam, Evangelion, MechWarrior) while anchored in the project's original support-first grammar.
- **CTQ-F1-013: Receipt/Replay Proof**
  Constructs a tamper-evident BLAKE3 receipt chain mapping `Prompt -> Contract -> USD -> Cook -> Playwright screenshot delta`. Re-running the pipeline under the same seed must produce identical receipt hashes.

## The F1 Admission Command
To trigger the complete verification process, run the command:
```bash
just verify-flagship-ue4-mech
```
The execution sequence executes the following steps:
1. Deletes all previously generated local asset outputs to ensure no caching interference.
2. Regenerates the entire F1 asset stack directly from ontological source law and SPARQL specifications.
3. Performs geometry, material, rig, and destruction state verification.
4. Executes the headless UE4 asset import pipeline, maps material data tables, and cooks the project assets.
5. Launches the local browser-native WebGL presentation, executes Playwright keyboard actuation, and computes the visual movement delta.
6. Computes mathematical IP-distance metrics against protected signature databases.
7. Emits complete Object-Centric Event Logs (OCEL) and a sequential BLAKE3 receipt chain.
8. Replays and compares receipt hashes to guarantee 100% deterministic reproducibility.
If any gate or metric fails, the script exits with a non-zero code.

## Code Layout

The project files are mapped into designated directories to keep generated artifacts, source templates, and test scripts isolated and clean:

- **Ontology Directories (`/Users/sac/rocket-craft/ontology/`)**:
  - `mecha_commons.ttl`: Holds semantic definitions of shared genre elements (cockpits, joint types, weapon mounts).
  - `protected_signatures.ttl`: Declares banned trademark design clusters (franchise-specific silhouettes, color combinations).
  - `original_design.ttl`: Defines the custom design grammar and axes (support-first roles, process-intelligence detail distributions).

- **SPARQL Queries (`/Users/sac/rocket-craft/queries/`)**:
  - `candidate_parts.rq`: Selects modular parts from design-space configuration.
  - `usd_prims.rq`: Queries geometry primitives for USDA expansion.
  - `materials.rq`: Extracts material/PBR settings for MaterialX emission.

- **Templates (`/Users/sac/rocket-craft/templates/`)**:
  - `asset.usda.tera`: Parent OpenUSD composition template.
  - `part_mesh.usda.tera`: Template for part-local primitive meshes.
  - `materials.mtlx.tera`: Lookdev/PBR mapping template.
  - `texture_program.rs.tera`: Procedural texture generation code template.

- **Generated Assets (`/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/`)**:
  - `reference/`: Reference imagery and targets (`reference_original.jpg`, `reference_silhouette.png`, `reference_color_histogram.json`).
  - `graph/`: Instanced graph files representing candidate configurations.
  - `usd/`: Final composed OpenUSD scenes (`ASSET_ReferenceFabric_001.usda`, `SM_Torso.usda`, `SM_Head.usda`, `SM_WingArray_Left.usda`, `SM_WingArray_Right.usda`, `SM_Blade_Left.usda`, `SM_Blade_Right.usda`).
  - `materialx/`: Lookdev outputs (`M_WhiteArmor.mtlx`, `M_CyanBlade.mtlx`, `M_DarkFrame.mtlx`, `M_GoldVisor.mtlx`).
  - `textures/`: Procedural textures (`T_WhiteArmor_BaseColor.png`, `T_WhiteArmor_Normal.png`, `T_CyanBlade_Emissive.png`, `texture_manifest.json`).
  - `renders/`: Offscreen headless renders used for visual validation (`render_front.png`, `render_angled.png`).
  - `reports/`: Verification details (`visual_gap_report.json`, `verifier_report.json`, `gap_closure_report.json`).
  - `ocel/`: Object-Centric Event Logs (`asset_manufacturing.ocel.json`).
  - `receipts/`: Blockchain-like cryptographic receipts (`asset_receipts.jsonl`).

- **Test Directories (`/Users/sac/rocket-craft/tests/`)**:
  - `tests/e2e/`: E2E test scripts, Playwright configurations, server setups.
  - `tests/unit/`: Unit tests validating SHACL constraints, SPARQL queries, and template expansion logic.
