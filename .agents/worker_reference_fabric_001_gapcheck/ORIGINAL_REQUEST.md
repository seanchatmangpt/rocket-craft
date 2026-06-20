## 2026-06-20T00:37:51Z
You are a teamwork_preview_worker agent.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_gapcheck`
Your task is to implement the final gap checker script `scripts/asset_fabric_gap_check.py` and run it to compute the milestone admission status.

Follow these instructions exactly:
1. Implement `scripts/asset_fabric_gap_check.py`. It should:
   - Check all 19 Gap IDs specified in the SPR:
     1. REFERENCE_MEASUREMENTS_EXIST: `reference_measurements.json` exists.
     2. GGEN_SYNC_PASSES: `ggen sync` runs and outputs files.
     3. USD_ASSEMBLY_EXISTS: `ASSET_ReferenceFabric_001.usda` exists.
     4. USD_MESH_FILES_EXIST: Part USD files exist (`SM_Torso.usda`, `SM_Head.usda`, `SM_WingArray_Left.usda`, etc.).
     5. USD_PARSES: USD files are valid (can be parsed or checked).
     6. USD_PRIM_COUNT_GE_120: `usd_prim_count` in reports is >= 120.
     7. WING_FEATHER_COUNT_GE_48: `wing_feather_count` in reports is >= 48.
     8. MATERIAL_BINDINGS_GE_4: `material_binding_count` in reports is >= 4.
     9. MATERIALX_FILES_GE_4: 4 MaterialX files exist under `materialx/`.
     10. TEXTURE_MANIFEST_EXISTS: `texture_manifest.json` exists.
     11. RENDER_FRONT_EXISTS: `render_front.png` exists.
     12. RENDER_ANGLED_EXISTS: `render_angled.png` exists.
     13. SILHOUETTE_IOU_GE_025: `silhouette_iou` is >= 0.25.
     14. COLOR_PALETTE_SIMILARITY_GE_050: `color_palette_similarity` is >= 0.50.
     15. FALSIFICATION_CASES_GE_8_PASS: Checks if at least 8 falsification cases are successfully handled/passed. Falsification checks test that invalid states (e.g. missing wing array, zero-point mesh, missing material binding, render not created, low prim count, low feather count, missing texture manifest, missing reference measurements) are correctly rejected or flagged as refused.
     16. COUNTERFACTUAL_CASES_GE_8_PASS: Checks if at least 8 counterfactual cases are implemented/passed (e.g. DOUBLE_WING_FEATHERS, HALF_WING_FEATHERS, REMOVE_CYAN_BLADES, INCREASE_WHITE_ARMOR_RATIO, DECREASE_CORE_BODY_WIDTH, INCREASE_WING_SPAN, REMOVE_GOLD_VISOR, ADD_RED_MICRO_DECALS) reporting metric changes.
     17. OCEL_EXISTS: `asset_manufacturing.ocel.json` exists.
     18. RECEIPTS_EXIST: `asset_receipts.jsonl` exists.
     19. REPORTS_UPDATED: Markdown/JSON reports are correctly compiled and present.

   - The script must compute a final admission status: if all 19 gaps are closed, status is `VERIFIED` and scoped status is `REFERENCE_TO_USD_RENDER_VERIFIED_UNDER_SCOPE`.
   - Write the output reports:
     - `generated/mech_assets/reference_fabric_001/reports/gap_closure_report.json`
     - `generated/mech_assets/reference_fabric_001/reports/gap_closure_report.md`
     - also copy `gap_closure_report.json` and `.md` to the repository root if required.

2. Run `python3 scripts/asset_fabric_gap_check.py` and verify that all checks pass and the reports are written.
3. Create a detailed handoff report in `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_gapcheck/handoff.md` detailing the gap checker logic, outputs, and the final status of each check.
4. Send a completion message to the parent (conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d).
