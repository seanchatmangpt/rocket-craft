## 2026-06-20T21:23:10Z
MANDATORY INTEGRITY WARNING — include this verbatim in the Worker's dispatch prompt:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.

You are dispatched to implement Milestones 3-6 (R2, R3, R4, R5, R6) and produce all required final reports for the PRE_UE4_HERO_ASSET_ADMISSION task.
Your task:
1. Edit `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera`:
   - In the `layered_swept_feather_panel` or `feather_panel` branch: change `def Cube "feather_blade_{{ i }}"` and `def Cone "feather_tip_{{ i }}"` to `def Mesh`. Ensure they contain:
     `color3f[] primvars:displayColor = [({{ row.colorR | default(value=0.9) }}, {{ row.colorG | default(value=0.9) }}, {{ row.colorB | default(value=0.9) }})]`
   - Add a `blade` branch that handles `row.type == "blade"`:
     ```jinja2
     {% elif row.type == "blade" %}
         def Mesh "cyan_beam"
         {
             int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
             int[] faceVertexIndices = [0, 1, 2, 3,  4, 5, 6, 7,  0, 1, 5, 4,  1, 2, 6, 5,  2, 3, 7, 6,  3, 0, 4, 7]
             point3f[] points = [(-0.5, -0.5, -0.5), (0.5, -0.5, -0.5), (0.5, 0.5, -0.5), (-0.5, 0.5, -0.5), (-0.5, -0.5, 0.5), (0.5, -0.5, 0.5), (0.5, 0.5, 0.5), (-0.5, 0.5, 0.5)]
             color3f[] primvars:displayColor = [({{ row.colorR | default(value=0.0) }}, {{ row.colorG | default(value=0.85) }}, {{ row.colorB | default(value=1.0) }})]
             double3 xformOp:scale = (2.0, 0.058, 0.058)
             double3 xformOp:rotateXYZ = (0, 0, 15)
             uniform token[] xformOpOrder = ["xformOp:rotateXYZ", "xformOp:scale"]
             rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
         }
     ```
2. Run `bash scripts/verify_asset.sh` and make sure it compiles. Ensure `wing_feather_count` becomes >0 and that the `USD305` and `USD304` errors are cleared.
3. Write a Python script `/Users/sac/rocket-craft/scripts/generate_all_reports.py` that runs the entire verification pipeline, gathers all metrics and receipts, and writes the following 14 files directly to the root of the project `/Users/sac/rocket-craft/`:
   - `VISION_POWL_LOOP_ADMISSION_REPORT.md` & `VISION_POWL_LOOP_ADMISSION_REPORT.json` (describing POWL loop execution and wasm4pm audit)
   - `SOURCE_LAW_REPLAY_REPORT.md` & `SOURCE_LAW_REPLAY_REPORT.json` (proving clean source law merge & ggen sync)
   - `MODULAR_IDENTITY_REPORT.md` & `MODULAR_IDENTITY_REPORT.json` (verifying part structure and lack of foreign/duplicate geometry)
   - `FRESH_RENDER_VERIFICATION_REPORT.md` & `FRESH_RENDER_VERIFICATION_REPORT.json` (confirming fresh renders are generated each check)
   - `RESIDUAL_VECTOR_REPORT.json` (saving the raw visual gap metrics)
   - `REPAIR_OPERATOR_SELECTION_REPORT.json` (listing bounded repair operators applied)
   - `DELETE_RESYNC_REPLAY_REPORT.md` & `DELETE_RESYNC_REPLAY_REPORT.json` (recording the run1 vs run2 delete-and-resync proof)
   - `BLAKE3_RECEIPT_CHAIN.json` (saving the receipt chain array)
   - `NEXT_GATE_STATUS.md` (specifying overall status is HOLD/CLAIM_HOLD and explaining the verifier holdout reasons: VIS203, VIS205, VIS208)
4. Execute `python3 scripts/generate_all_reports.py` and verify all files are correctly created and contain accurate, non-placeholder data from the run.

Write progress logs to `/Users/sac/rocket-craft/.agents/worker_reports/progress.md` and your final handoff to `/Users/sac/rocket-craft/.agents/worker_reports/handoff.md`.
