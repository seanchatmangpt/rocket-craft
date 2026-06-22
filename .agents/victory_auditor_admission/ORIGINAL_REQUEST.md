## 2026-06-20T21:32:41Z
You are the Forensic Auditor (teamwork_preview_auditor).
Your working directory is `/Users/sac/rocket-craft/.agents/victory_auditor_admission/`.

Your task is to run forensic integrity checks on the implementation and reports of the PRE_UE4_HERO_ASSET_ADMISSION task:
1. Audit the source code and templates in the workspace (specifically, `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera`, `/Users/sac/rocket-craft/patch_geometry_generator.py`, `/Users/sac/rocket-craft/scripts/generate_all_reports.py`) to ensure there are no hardcoded mock results, fake runtime mocks, or integrity violations designed to bypass checks.
2. Verify that the 14 reports generated at the project root are fully authentic, genuine, and consistent with the actual codebase and generated USD parts, rather than using stubbed/hardcoded values.
3. Confirm that the verify command (`bash scripts/verify_asset.sh`) outputs pass/holdout values genuinely and that the tests/verifications pass or fail cleanly without bypasses.
4. Verify that USD parts contain only their correct prims, sockets contain no mesh payloads, and part-level files are not duplicate assembly files.
5. Record your verdict and evidence in `/Users/sac/rocket-craft/.agents/victory_auditor_admission/handoff.md` in your working directory.
6. Report your verdict as either CLEAN or INTEGRITY VIOLATION.
7. Send a message to the parent (conversation ID: 88fcd31c-c4a2-4feb-9052-7ac2ae5eaf71) when done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. Integrity violations WILL be detected and your
work WILL be rejected.
