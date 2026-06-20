# Handoff Report — explorer_exploration_1

## 1. Observation

- **RDF Ontologies location**:
  - Found under `/Users/sac/rocket-craft/ontology/`:
    - `core.ttl` (lines 1-100+) containing general classes.
    - `gundam_nexus.ttl` containing Gundam domain vocabulary.
    - `mech_factory_mud.ttl` containing station ontology models.
    - `mechbirth.ttl` containing level of detail and motion families.
    - `all_merged.ttl` containing the fully combined graph.
  - Found under `/Users/sac/rocket-craft/ontology/ggen-packs/`:
    - `mech_factory_mud/schema/mech_factory_mud.ttl` (MUD schema)
    - `mechbirth/schema/mechbirth_lod_geom_motion.ttl` (LOD and geom/motion schema)
- **SPARQL queries and Tera templates**:
  - Found in `ontology/ggen-packs/` subdirectories (`cognition`, `mech_factory_mud`, `mechbirth`, `sosa-ssn`).
  - Specifying mapping files in `ggen.toml` (lines 1-331), for example:
    - Line 17: `query = { file = "ontology/ggen-packs/mech_factory_mud/GeometrySurrogate.sparql" }`
    - Line 19: `template = { file = "ontology/ggen-packs/mech_factory_mud/GeometrySurrogate.tera" }`
    - Line 20: `output_file = "crates/mech_factory_mud/src/GeometrySurrogate.rs"`
  - Custom templates are located in `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/` (e.g. `usd/asset.usda.tera`, `usd/part_mesh.usda.tera`, `materialx/materials.mtlx.tera`, `texture_program.rs.tera`, `visual_gap_report.md.tera`).
- **USD, MaterialX, textures, rigs, and collision hulls generators**:
  - `ggen` generator configured in `ggen.toml` compiled:
    - Master USD assembly (`ASSET_ReferenceFabric_001.usda`) and sub-meshes (`SM_Torso.usda`, `SM_Head.usda`, etc.) under `generated/mech_assets/reference_fabric_001/usd/`.
    - MaterialX documents: `generated/mech_assets/reference_fabric_001/materialx/` (`M_WhiteArmor.mtlx`, `M_CyanBlade.mtlx`, etc.) via `materials.mtlx.tera`.
  - Python texture generator `scripts/render_reference_fabric.py` (lines 15-79) generates png base colors, roughness, normal maps, emissive, and `texture_manifest.json`.
  - Non-FBX model files are converted to FBX 7.4 via `asset-pipeline/scripts/blender_convert.py` (lines 92-121) orchestrated by Rust crates under `/Users/sac/rocket-craft/asset-pipeline/` (running headless blender `bpy`).
  - Collision hulls are represented in RDF as `mud:GeometrySurrogate` nodes and compiled by `ggen` into branchless static array tables in `crates/mech_factory_mud/src/GeometrySurrogate.rs` (derived via `GeometrySurrogate.tera` lines 14-23).
- **Headless rendering & visual verification**:
  - Renders are run headlessly using `/usr/bin/usdrecord` (v0.25.2) using the Metal backend (`--renderer Metal`) under `scripts/render_reference_fabric.py` (lines 108-115, 136-143).
  - Silhouette masks and edge maps are post-processed with `PIL` (lines 149-187).
  - Comparison script `/Users/sac/rocket-craft/scripts/compare_reference_render.py` calculates aligned Silhouette IoU (lines 84-102), Edge Cosine Similarity (lines 104-116), and Color Palette Similarity (lines 118-165).
  - The gap checker script `/Users/sac/rocket-craft/scripts/asset_fabric_gap_check.py` validates 19 Gap IDs (lines 200-340) for `GC-MECH-ASSET-FABRIC-001`, and executes 8 physical falsification tests (lines 343-458) and 8 counterfactual cases (lines 468-546).
- **Build/test setup and validation results/receipts**:
  - Orchestration in `Justfile` (lines 45-134) for `just test-rust` (which runs `asset-pipeline` cargo tests) and `just ci`.
  - Execution run script `verify_html5_pipeline.sh` (lines 1-177) which verified the WASM binary, staged files to PWA serve directory, started the `rocket-cmd` HTTP server on port 8080, ran Playwright tests (Vitest/E2E spec files) to capture canvas `visualDelta`, and validated the resulting receipt.
  - Receipt validator command is implemented in Rust under `tools/rocket-cmd/src/verbs/receipt.rs` (lines 34-169) checking verdicts, WASM file sizes (>=10MB), and visual canvas deltas (>=20) to reject stubs and mocks.
  - Verifiable Merkle-like chain receipts are captured in `generated/mech_assets/reference_fabric_001/receipts/asset_receipts.jsonl` (sequences 1 to 12).

## 2. Logic Chain

1. By viewing `ontology/` contents and checking `ggen.toml`, it is proved that the static semantic structure and topological properties of mechas are defined in RDF Turtle files and compiled down to Rust types and USD assets using queries and Tera templates.
2. By viewing `/Users/sac/rocket-craft/asset-pipeline` crates and `blender_convert.py`, it is verified that Blender conversion transforms non-FBX models into compliant UE4 FBX 7.4 packages.
3. By analyzing `scripts/render_reference_fabric.py` and `scripts/compare_reference_render.py`, it is established that Apple's native `/usr/bin/usdrecord` Metal renderer runs headlessly to generate PNG views of USD models, which Pillow post-processes into silhouettes/edges and compared with targets using aligned IoU and cosine similarities.
4. By reviewing `scripts/asset_fabric_gap_check.py`, it is demonstrated that a comprehensive verification suite runs 19 gap checks, 8 falsification mutations, and 8 counterfactual cases.
5. By analyzing `tools/rocket-cmd/src/verbs/receipt.rs`, it is shown that `rocket-cmd` receipt validation enforces `verdict = "PASS"`, wasm size >= 10MB, and Playwright canvas `visualDelta` >= 20, guaranteeing that the actual target compiler and runtime engine were actuated with observed visual changes instead of mock results.

## 3. Caveats

- Xcode/Apple Native USD Tools (`usdrecord`) must be present in `/usr/bin/usdrecord`. On non-macOS systems, another path or mock/fallback setup would be needed.
- Headless Blender requires Blender binary to be discoverable in `BLENDER_PATH` or common system paths, otherwise non-FBX models cannot be converted.
- It is assumed that `all_merged.ttl` contains all necessary mecha model parts and instances for `ggen` to successfully compile the reference fabric layout.

## 4. Conclusion

The Rocket-Craft workspace utilizes a strictly structured mecha asset generation pipeline:
- **Ontology**: Bounded semantic models defined in Turtle files (`ontology/*.ttl`) compile static shapes, bounds, materials, and processes.
- **Actuation**: `ggen` templates output static rust surrogates and USD specifications, while python procedurally compiles textures and MaterialX definitions.
- **Verification**: Headless Metal rendering via `usdrecord` produces PNGs post-processed into silhouettes and edges, compared against targets, and verified using 19 gap checks, falsifications, and counterfactual design modifications.
- **Receipts**: Playwright execution registers canvas updates (`visualDelta` >= 20) and validates compiled WASM sizes to prevent stub laundering, producing sequential receipt chains in `asset_receipts.jsonl`.

## 5. Verification Method

To verify the findings and run the pipeline validations:
1. Run Rust workspace unit tests:
   `just test-rust`
2. Run the gap check verifier script:
   `python3 scripts/asset_fabric_gap_check.py`
3. Confirm files exist at:
   - `generated/mech_assets/reference_fabric_001/reports/gap_closure_report.json`
   - `generated/mech_assets/reference_fabric_001/receipts/asset_receipts.jsonl`
   - `crates/mech_factory_mud/src/GeometrySurrogate.rs`
4. Confirm receipt validator commands:
   `./rocket receipt validate --file pwa-staff/test-results/tps-dflss-receipt.json`
