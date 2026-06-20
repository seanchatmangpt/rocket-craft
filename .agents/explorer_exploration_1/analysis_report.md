# Mecha Asset Generation Pipeline Investigation Report

**Date**: 2026-06-20  
**Status**: VERIFIED  
**Working Directory**: `/Users/sac/rocket-craft/.agents/explorer_exploration_1`  

This report documents the workspace structure and mecha asset generation pipeline in the `rocket-craft` repository.

---

## 1. RDF Ontologies, Turtle Files, SPARQL Queries, and Tera Templates

### Ontologies and Turtle Files (`/Users/sac/rocket-craft/ontology/`)
The semantic authority and topological relationships are represented using Turtle (`.ttl`) format files located in the `ontology/` directory:
- **`ontology/all_merged.ttl`**: The primary merged ontology containing the unified semantic graph.
- **`ontology/core.ttl`**: The base schema containing the core types and properties (e.g. Pawn, Character, ActorComponent, physical attributes).
- **`ontology/gundam_nexus.ttl`**: Domain-specific extensions for the Gundam/mecha models.
- **`ontology/mech_factory_mud.ttl`**: Ontology defining MUD (Multi-User Dungeon) factory digital twin concepts, including rooms, paths, stations, processes, and validation bounds.
- **`ontology/mechbirth.ttl`**: The semantic rules governing mecha generation, including geometry, Level of Detail (LOD) classes, phase ordering, and transition bounds.
- **`ontology/anti_llm.ttl`**: A set of defensive prompt engineering rules and rules against mock laundering captured in semantic form.
- **`ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl`**: Bounded schema definition for the mecha factory MUD.
- **`ontology/ggen-packs/mechbirth/schema/mechbirth_lod_geom_motion.ttl`**: Level of Detail (LOD) geometry and motion schema rules for mech birth.

### SPARQL Queries (`/Users/sac/rocket-craft/ontology/ggen-packs/` & `/Users/sac/rocket-craft/generated/`)
SPARQL queries extract deterministic subsets of the RDF graph to drive templating:
- **`ontology/ggen-packs/mech_factory_mud/GeometrySurrogate.sparql`**: Queries bounding boxes, sockets, and clearance radii.
- **`ontology/ggen-packs/mech_factory_mud/MotionSurrogate.sparql`**: Queries motion characteristics.
- **`ontology/ggen-packs/mech_factory_mud/SkinSurrogate.sparql`**: Queries material layer distributions.
- **`ontology/ggen-packs/mech_factory_mud/MechFactoryMudOcelSchema.sparql`**: Queries event logging event/object mappings.
- **`ontology/ggen-packs/mech_factory_mud/MechFactoryMudReceiptSchema.sparql`**: Queries receipt payload variables.
- **`ontology/ggen-packs/mech_factory_mud/station_processes.sparql`**: Queries station workflow mappings.
- **`ontology/ggen-packs/mech_factory_mud/walkthrough_transitions.sparql`**: Queries movement transition steps.
- **`ontology/ggen-packs/mech_factory_mud/ue4_projection_contract.sparql`**: Queries static contract structures for Unreal Engine.
- **`ontology/ggen-packs/mechbirth/queries/extract_*.sparql`**: Extraction queries for LOD classes, motion families, part families, phase constraints, and receipt payloads.
- **`generated/mech_assets/reference_fabric_001/queries/`**:
  - `materials.rq`: Queries material parameters.
  - `texture_programs.rq`: Queries procedural texture commands.
  - `usd_prims.rq`: Queries primitive hierarchy fields.
  - `verifier_expectations.rq`: Queries validation expectations (IoU, colors, etc.).

### Tera Templates (`/Users/sac/rocket-craft/ontology/ggen-packs/` & `/Users/sac/rocket-craft/generated/`)
Templates consume query output and produce Rust code, USD scene files, or configuration CSVs:
- **`ontology/ggen-packs/mech_factory_mud/GeometrySurrogate.tera`**: Generates `crates/mech_factory_mud/src/GeometrySurrogate.rs`.
- **`ontology/ggen-packs/mech_factory_mud/templates/rust/`**: Templates for `authority.rs`, `constants.rs`, `ocel.rs`, `parts.rs`, `projection.rs`, `receipt.rs`, and `stations.rs`.
- **`ontology/ggen-packs/mech_factory_mud/templates/ue4/`**: Templates producing CSV DataTables for import into UE4 (`FactoryStations.csv`, `MotionFamilies.csv`, `PartFamilies.csv`, `ProjectionCommands.csv`, `SemanticLOD.csv`, `SkinLayers.csv`, `SocketTopology.csv`, and `WalkthroughRoute.csv`).
- **`ontology/ggen-packs/mechbirth/templates/`**: Templates generating Rust files for `geometry.rs`, `motion.rs`, `receipt.rs`, and `semantic_lod.rs`.
- **`generated/mech_assets/reference_fabric_001/templates/`**:
  - `usd/asset.usda.tera`: Generates the master assembly `ASSET_ReferenceFabric_001.usda`.
  - `usd/part_mesh.usda.tera`: Generates meshes for torso, head, wings, and blades.
  - `materialx/materials.mtlx.tera`: Generates MaterialX (`.mtlx`) definitions.
  - `texture_program.rs.tera`: Generates texture generation instructions.
  - `visual_gap_report.md.tera`: Generates gap checks markdown report templates.

---

## 2. Generators & Scripts (USD, MaterialX, Textures, Rigs, Hulls)

The mecha generator is driven by **`ggen`** (the ontology-based code generator executable, v26.6.11), configured using the workspace file **`ggen.toml`**. The pipeline executes in steps using the following scripts and tools:

### USD Model Generation
- **`ggen`** reads `ontology/all_merged.ttl` and generates:
  - Master assembly: `generated/mech_assets/reference_fabric_001/usd/ASSET_ReferenceFabric_001.usda`
  - Body parts: `SM_Torso.usda`, `SM_Head.usda`, `SM_WingArray_Left.usda`, `SM_WingArray_Right.usda`, `SM_Blade_Left.usda`, `SM_Blade_Right.usda`.
- **`asset-pipeline`** (Rust Cargo workspace: `pipeline-cli` & `pipeline-core` packages) handles non-FBX models. It discovers Blender and executes it headlessly via:
  - **`asset-pipeline/scripts/blender_convert.py`**: A python script executed inside Blender to import formats (`obj`, `fbx`, `stl`, `dae`, `gltf`, `glb`) and export them as UE4-compatible FBX 7.4 binary files.

### MaterialX Material Generation
- **`ggen`** compiles material specifications defined in the ontology to MaterialX XML document files:
  - `M_WhiteArmor.mtlx`
  - `M_CyanBlade.mtlx`
  - `M_DarkFrame.mtlx`
  - `M_GoldVisor.mtlx`

### Texture Generation
- **`scripts/render_reference_fabric.py`**: A python script that procedurally generates mecha texture PNG images:
  - `T_WhiteArmor_BaseColor.png` (white texture with a panel grid)
  - `T_WhiteArmor_Roughness.png` (roughness map)
  - `T_WhiteArmor_Normal.png` (flat normal map)
  - `T_CyanBlade_Emissive.png` (cyan emissive texture)
  - Write output manifest to `texture_manifest.json`.

### Rigging & Collision Hulls
- Rig structures and skeleton attachment sockets are generated as DataTable CSV files using `SocketTopology.csv.tera` and mapped to Unreal C++ structures.
- Collision hull bounds and bounding boxes are computed directly from the ontology as `mud:GeometrySurrogate` nodes. Bounding boxes (AABB) and clearance radii are compiled by `ggen` into branchless static array tables in `crates/mech_factory_mud/src/GeometrySurrogate.rs`.

---

## 3. Visual Verification, Headless Rendering, and Gap Closure Checking

Visual verification evaluates the procedural assets against visual targets:

### Headless Rendering
- **Tool**: `/usr/bin/usdrecord` (native Apple USD tool, v0.25.2) with the Metal backend (`--renderer Metal`).
- **Script**: `scripts/render_reference_fabric.py`
  - Renders front and angled views of the generated mecha models using `usdrecord`.
  - Post-processes the front render using Python's Pillow (`PIL`) library to compute a binary silhouette mask (`render_silhouette.png`) and an edge detection map (`render_edges.png` using the `ImageFilter.FIND_EDGES` filter).

### Comparative Metric Scoring
- **Script**: `scripts/compare_reference_render.py`
  - Compares generated renders against the baseline reference targets.
  - Computes:
    - **Silhouette IoU**: Aligns and scales generated and reference silhouettes to compute Intersection-over-Union.
    - **Edge Cosine Similarity**: Flattened cosine similarity between the generated edge map and the reference target.
    - **Color Palette Similarity**: Computes L1 distance between the classified color profiles of foreground pixels and target distributions.
    - **Symmetry Delta**: Estimates left/right symmetry.
    - **Dimensions & Mass Deltas**: Computes wing span delta and central body mass ratios.
  - Outputs results to `reports/verifier_report.json` and `reports/verifier_report.md`.

### Gap Closure Checker
- **Script**: `scripts/asset_fabric_gap_check.py`
  - Evaluates **19 distinct Gap IDs/requirements** for the milestone `GC-MECH-ASSET-FABRIC-001`, confirming that physical files, configurations, and verification metrics (IoU >= 0.25, Color Similarity >= 0.50, prim count >= 120, etc.) are valid.
  - **Falsification Mutation Suite**: Executes 8 mutations (corrupting material bindings, clearing meshes, deleting maps, etc.) to verify that the pipeline correctly rejects invalid assets with expected refusal reasons.
  - **Counterfactual Suite**: Simulates 8 mecha modifications (e.g. changing wingspan, removing blades, adjusting feather count) to record delta changes.
  - Outputs `gap_closure_report.json` and `gap_closure_report.md` in the `generated/mech_assets/reference_fabric_001/reports/` folder, copying them to the root.

---

## 4. Build/Test Setup and Receipt Capture

### Build & Test Orchestration
- Driven by the workspace **`Justfile`**:
  - `just test`: Runs all Rust package tests (including `asset-pipeline`) and Vitest TypeScript tests.
  - `just ci`: Enforces styling checks, Rust clippy, testing, TypeScript compilation, and receipt validation.
  - `just build-rocket`: Builds the `rocket-cmd` CLI utility located in `tools/rocket-cmd/`.
- Pipeline integration testing is driven by **`verify_html5_pipeline.sh`** and **`verify_gundam_pipeline.sh`**:
  - Verification verifies cooked WASM files using `./rocket wasm verify`.
  - Launches local server via `./rocket html5 serve`.
  - Runs Playwright tests (e.g. `tests-e2e/tps-dflss.spec.ts`) against local browser instances to capture visual screenshots and actuate user input.
  - Generates session receipts proving actual rendering and input actuation.

### Receipt and Validation Capture
Receipts serve as cryptographic proofs that compilation, execution, and rendering succeeded:
- **Cook Receipts** (`cook-receipt.json`): Generated by `rocket html5 verify`. Stores WASM binary sizes (must be >= 10 MB to prevent stub laundering) and SHA-256/BLAKE3 file hashes.
- **Playwright Session Receipts** (`tps-dflss-receipt.json`): Generated after browser execution. Evaluates the `visualDelta` (must be >= 20, verifying that canvas movement actually occurred). Rejects synthetic mock receipts.
- **Validation tool**: `tools/rocket-cmd/src/verbs/receipt.rs` implements `./rocket receipt validate --file <path>`. It parses JSON receipts and checks verdicts and boundaries.
- **Cross-check**: The shell runner cross-checks the output hashes of the cooked WASM vs the loaded browser WASM to detect and reject binary substitutions.
- **Manufacturing Receipt Chain**: The file `generated/mech_assets/reference_fabric_001/receipts/asset_receipts.jsonl` contains a sequential log of generated assets with their content hashes linked to the prior hash, establishing a Merkle-like chain of trust.
