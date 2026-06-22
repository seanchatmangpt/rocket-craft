## 2026-06-21T00:43:22Z

You are the Purity Remediation Worker.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_remediation_tera_purity`.
Your parent is the Project Orchestrator (conversation ID: 4de4468a-b1c4-4f05-8ff3-cb26e5516fd2).
Your task is to remediate the `TERA_TRANSLATOR_PURITY` violation in the template `generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera`.

Specifically:
1. Edit `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera` to remove all hardcoded overrides for `my_ty` (translateY) and `my_sy` (scaleY) on lines 74-89. Use the values `row.translateY` and `row.scaleY` directly from the query rows.
2. Run the full verification pipeline:
   - Run the merge script: `python3 scripts/merge_ontology.py`
   - Run the metric morphology verification: `python3 scripts/verify_metric_morphology.py`
   - Run the R6 delete-resync replay verification: `python3 scripts/verify_r6_delete_resync_replay.py`
3. Ensure that the generated assets reproduce byte-identically (or within tolerance) and match the expected values, with no hardcoded values remaining in the template.
4. Ensure the standing remains `REFUSED` under `CLAIM_HOLD` in the reports (e.g. `DELETE_RESYNC_REPLAY_REPORT.json` and `NEXT_GATE_STATUS.md`).

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your handoff report to `/Users/sac/rocket-craft/.agents/worker_remediation_tera_purity/handoff.md` and report back.
