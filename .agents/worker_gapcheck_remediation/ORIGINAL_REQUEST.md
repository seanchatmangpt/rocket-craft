## 2026-06-20T21:41:13Z

MANDATORY INTEGRITY WARNING — include this verbatim in the Worker's dispatch prompt:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.

You are dispatched to remediate the falsification failures in `scripts/asset_fabric_gap_check.py`:
1. Inspect the file `/Users/sac/rocket-craft/scripts/asset_fabric_gap_check.py`.
2. Locate the `MISSING_MATERIAL_BINDING` mutation check (around line 384). Update it to target `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/usd/SM_Blade_Left.usda` (using `blade_path` instead of `head_path`), as `SM_Blade_Left.usda` contains `def Mesh` and will properly trigger the missing material binding refusal check.
3. Locate the `LOW_FEATHER_COUNT` mutation check (around line 423). Update it to generate 20 `def Mesh` prims in both `SM_WingArray_Left.usda` and `SM_WingArray_Right.usda` (for a total wing feather count of 40, which is < 48), so that the total primitive count remains above 120 and does not trigger the `LOW_PRIM_COUNT` check.
4. Run the offline Vitest unit test suite using `npm test` inside `pwa-staff/` to verify that all mecha_offline tests compile and pass cleanly (expecting 48/48 tests passing).
5. Run `python3 scripts/asset_fabric_gap_check.py` to ensure that `gap_closure_report.json` status becomes `VERIFIED` and all falsification checks pass successfully.
6. Write your progress to `/Users/sac/rocket-craft/.agents/worker_gapcheck_remediation/progress.md` and handoff report to `/Users/sac/rocket-craft/.agents/worker_gapcheck_remediation/handoff.md`.
7. When done, send a message to the orchestrator (conversation ID: 88fcd31c-c4a2-4feb-9052-7ac2ae5eaf71).
