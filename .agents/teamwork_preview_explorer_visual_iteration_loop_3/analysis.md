# Metric Morphology Bands and Pipeline Verification Analysis

## Objective
Analyze the validation processes of `validate_shacl.py` and `scripts/verify_r6_delete_resync_replay.py` to ensure pipeline safety, semantic correctness, and deterministic replayability when morphology bands are tightened.

---

## 1. Validation Process Analysis

### A. Static SHACL Validation (`validate_shacl.py` / `validate_shacl_merged.py`)
- **Process**: Loads all source ontology Turtle files (excluding `all_merged.ttl` and `anti_llm` files) and validates them using PySHACL (`validate(g, inference='rdfs')`).
- **Core Loop Gap**: This script performs a **static schema-only check**. It does not load any instance triples of the generated mecha parts (e.g. `rf:ReferenceFabric_001` and its part bounding boxes). Consequently, target class shapes (such as `sh:targetClass eng:MechaPart`) have no instances to validate. 
- **Consequence**: The static check passes vacuously (`Conforms: True`) regardless of whether the generated mecha actually violates the metric morphology bands. It only flags structural syntax or OWL schema paradoxes within the ontology files themselves.

### B. Delete and Resync Replay Proof (`verify_r6_delete_resync_replay.py` / `verify_delete_and_resync_replay.py`)
- **Process**: Runs two full rebuilds (`rebuild #1` and `rebuild #2`) starting from clean states by deleting generated directories (`usd/`, `renders/` etc.) to prove reconstructive authority. It runs:
  1. `merge_ontology.py` (merges source laws to `all_merged.ttl`)
  2. `patch_geometry_generator.py` (updates generator code and templates)
  3. `ggen sync` (generates the C++ headers and USD/MaterialX files based on `all_merged.ttl`)
  4. `generate_procedural_textures.py` (creates programmatic PNG textures)
  5. `render_reference_fabric.py` (uses headless `usdrecord` with Apple Metal renderer to render front/angled PNG views)
  6. `compare_reference_render.py` (calculates similarity metrics, visual gap reports, and event logs)
- **Comparisons**: 
  - **Class 1 (Byte-Exact)**: Compares SHA-256 byte hashes of deterministic files (`.usda`, `.mtlx`, textures/PNGs, manifests) between rebuild #1 and #2. Any divergence halts the pipeline.
  - **Class 2 (Disposition-Exact)**: Since Apple Metal GPU rendering is not bit-deterministic, it rounds metrics to 4 decimal places and compares the resulting dict (usd_errors, vis_errors, metrics).
- **Loop Gap**: The rebuild steps in `verify_delete_and_resync_replay.py` **completely bypass the morphology gate** (`verify_metric_morphology.py`). If a generated asset compiles but violates the morphology bands, the replay proof can still succeed and produce a BLAKE3 receipt chain, leading to false standing.

### C. Active Morphology Validation (`verify_metric_morphology.py`)
- **Process**: Measures actual generated USDA bounding boxes (converting cm to meters), creates an in-memory RDF instance graph representing the generated mecha, loads SHACL shapes from `110_bipedal_metric_envelope_law.ttl` and `116_metric_morphology_bands.ttl`, and executes PySHACL validation.
- **Discrepancies Found**:
  - The actual torso height ratio is `0.1290`, which is outside the declared torso segment band of `[0.30, 0.45]`.
  - The actual limb/leg height ratio is `0.6307`, which is outside the bipedal leg/limb band of `[0.40, 0.55]`.
- **Pipeline Bypass**: In `scripts/verify_asset.sh`, a morphology gate violation (exit code `1`) only prints a warning and allows the render/score steps to continue (treated as non-blocking). This creates false standing because visual assets and BLAKE3 receipts are generated for a mecha that violates the graph law.

---

## 2. Impact on Source Ontologies and Queries

When morphology bands are tightened, the following files and components are impacted:

### A. Ontology Files
- **`116_metric_morphology_bands.ttl`**: Holds the declarative `MorphologyBand` min/max ratio bounds. Tightening these ranges forces the generator parameters to be tuned more precisely to prevent violations.
- **`117_reference_fabric_metric_binding.ttl`**: Binds the mecha parts (`rf:SM_Head`, `rf:SM_Torso` etc.) to the engineering classes. If new parts are added or existing parts are renamed, their class typings must be updated here.
- **`110_bipedal_metric_envelope_law.ttl`**: Declares anatomy structural paradox shapes (e.g. head must be above torso).

### B. Script Hardcoding
- **`verify_metric_morphology.py`**: Contains a hardcoded python dictionary `PART_BANDS` containing the bounds. Tightening the bands in the Turtle files **requires synchronizing** this python dictionary to prevent discrepancy errors.

### C. Generation Rules & Queries
- **`ggen.toml`**: The SPARQL queries select primitives based on `belongsToPart`. If parts are renamed or refactored, the inline queries in `ggen.toml` must be modified.
- **`patch_geometry_generator.py`**: The template generator script writes the geometry template. If bands are tightened, the scaling/translation multipliers in the templates must be updated to fit the new bands.

---

## 3. Recommended Verification Steps to Prevent False Standing

To guarantee the integrity of the Combinatorial Maximalist Doctrine and ensure that zero false standing is generated:

1. **Halt on Morphology Gate Failure**:
   Modify `scripts/verify_asset.sh` to exit immediately with a non-zero code if `scripts/verify_metric_morphology.py` returns `1` (real mecha violates morphology bands). The render and score steps must structurally halt.
2. **Include Morphology Gate in Replay Proof**:
   Add `["python3", "scripts/verify_metric_morphology.py"]` into the `CANONICAL_REBUILD_STEPS` in `verify_delete_and_resync_replay.py`. If morphology validation fails in either rebuild, the replay proof must fail and refuse standing.
3. **Synchronize Python Bands dynamically**:
   Modify `verify_metric_morphology.py` to extract the `MorphologyBand` min/max limits dynamically from the loaded ontology graph rather than relying on a hardcoded python dict.
4. **Snapshot/Freeze Workspace During Rebuild**:
   Snapshot all generator and source law files before commencing `rebuild #1` and `rebuild #2` to prevent other parallel agents from modifying files (e.g. `patch_geometry_generator.py`) mid-run, which causes non-deterministic diff failures.
5. **Enforce Strict Order By**:
   Audit all SPARQL queries inside `ggen.toml` and query files to ensure that results are sorted by a combination of variables that guarantees a unique sort key (e.g., `ORDER BY ?prim ?part`).
