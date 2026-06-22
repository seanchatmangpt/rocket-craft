# Handoff Report — Metric Morphology Bands and Pipeline Verification

## 1. Observation
We directly observed the following files, commands, and results:
- **SHACL Schema Validation (`validate_shacl.py`)**:
  Line 11 in `validate_shacl.py` invokes:
  ```python
  conforms, results_graph, results_text = validate(g, inference='rdfs')
  ```
  Running this command yields:
  ```
  Conforms: True
  ```
  However, this check does not include any actual part measurements or instance data, passing vacuously.
- **Morphology Gate script (`scripts/verify_metric_morphology.py`)**:
  Manually running `python3 scripts/verify_metric_morphology.py` returns exit code `1` and outputs:
  ```
  verdict: PARTIAL_ALIVE | shacl_conforms=False | neg_refused=True | replay=True
    refusal: REFUSE_PART_HEIGHT_BAND(SM_Limb_Left): ratio 0.6307 outside [0.40,0.55]
    refusal: REFUSE_PART_HEIGHT_BAND(SM_Limb_Right): ratio 0.6307 outside [0.40,0.55]
    refusal: REFUSE_PART_HEIGHT_BAND(SM_Torso): ratio 0.1290 outside [0.30,0.45]
    refusal: REFUSE_PART_HEIGHT_BAND: part height ratio falls outside its declared morphology band.
  ```
- **Asset Verification script (`scripts/verify_asset.sh`)**:
  Lines 33–40 in `scripts/verify_asset.sh` show morphology violations are bypassed as warnings:
  ```bash
  elif [ "$MM_RC" -eq 1 ]; then
      echo "!! METRIC MORPHOLOGY: PARTIAL_ALIVE -- real mech violates a band (see METRIC_MORPHOLOGY_REPORT.md). Render continues (non-blocking)."
  ```
- **Replay Proof (`verify_r6_delete_resync_replay.py`)**:
  `verify_r6_delete_resync_replay.py` does not include `verify_metric_morphology.py` in its list of rebuild steps `CANONICAL_REBUILD_STEPS`, completely bypassing active morphology validations.
- **Multi-Agent Interference**:
  Running `python3 scripts/verify_r6_delete_resync_replay.py` yielded:
  ```
  === VERIFYING (a) generator-artifact byte-identity ===
    FAIL: non-deterministic generator artifact -> usd/SM_Blade_Left.usda
      rebuild#1: 649339453ea2167da240644ce6162d6b0934f0a9632f6a25c1232814af83398a
      rebuild#2: 91f8a31ce57a17b9f691eab9e2ce998af25008f62b6b4a7780f9899ac0be887f
  ```
  This is caused by concurrent modifications to `patch_geometry_generator.py` in the workspace root by other agents during the two-pass rebuild execution.

---

## 2. Logic Chain
1. Since `validate_shacl.py` loads only ontology files without mecha instance bounding box data, it is incapable of catching real mecha morphology violations (Observation 1).
2. The actual morphology validation is run by `verify_metric_morphology.py`, which correctly catches limb and torso ratio violations (Observation 2).
3. However, `verify_asset.sh` allows these violations to proceed to rendering and receipt generation (Observation 3), and `verify_delete_and_resync_replay.py` does not execute the morphology check at all (Observation 4).
4. Therefore, the pipeline can produce visual assets and cryptographic receipts that violate semantic laws (Conclusion).
5. Additionally, the lack of source/generator snapshotting in `verify_delete_and_resync_replay.py` leaves it vulnerable to workspace modifications by concurrent agents, causing replay determinism checks to fail (Observation 5).

---

## 3. Caveats
No caveats.

---

## 4. Conclusion
The current validation process is insufficient to prevent "false standing" under the Combinatorial Maximalist Doctrine because morphology band violations are treated as warnings and bypassed. Furthermore, the replay proof does not validate morphology, and is vulnerable to concurrency issues when run in a multi-agent environment.

To secure the pipeline:
1. Halt the pipeline immediately (non-zero exit code) on morphology violations in `verify_asset.sh`.
2. Add the morphology check step to `verify_delete_and_resync_replay.py`.
3. Snapshot generator files before running the replay rebuilds to isolate the process from concurrent workspace changes.

---

## 5. Verification Method
- **Verify Morphology Violations**: Run `python3 scripts/verify_metric_morphology.py` from the root directory and inspect `METRIC_MORPHOLOGY_REPORT.json` for validation results.
- **Verify Static SHACL**: Run `python3 validate_shacl.py` to check for schema/syntax correctness.
- **Verify Replay Proof**: Ensure no concurrent processes are editing the source/generator files, and run `python3 scripts/verify_r6_delete_resync_replay.py` to confirm deterministic output generation.
