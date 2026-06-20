# Original User Request

## Initial Request — 2026-06-19T18:03:33-07:00

You are the sub-orchestrator for the E2E Testing Track of target `AAA_UE4_MECH_PACK_001` (GC-AAA-UE4-MECH-001). Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_e2e_testing_aaa_ue4_mech_pack_001`.

Your mission is to establish the E2E testing infrastructure and write the test cases for the mecha asset generation pipeline:
1. Create and author `/Users/sac/rocket-craft/TEST_INFRA.md` following the template in PROJECT.md. Incorporate the mecha-specific features: modular USD identity (USD301-307), part-aware morphology metrics (VIS201-208), MaterialX PBR channel completeness, UsdSkel rigging/sockets, UE4 import/cook verification, IP-distance non-confusion, and Playwright canvas motion delta.
2. Design and programmatically implement the 4-tier E2E test cases:
   - Tier 1: Feature Coverage (geometry, morphology, materials, rigs/sockets, UE4 import, IP-distance, receipts).
   - Tier 2: Boundary/Edge Cases (empty meshes, duplicate fingerprint detection, bounding box overlaps, V-fin IP proximities).
   - Tier 3: Cross-Feature Interactions (materials bound to wing feathers, sockets attached to skeleton joints, walkthrough event telemetry).
   - Tier 4: Real-world walkthrough scenarios (browser-native HTML5 rendering, movement actuation, visual delta verification).
3. Ensure that there is a test runner (e.g. running Vitest, Playwright, or custom verifier scripts) that can execute all test tiers.
4. When the test suite is fully functional and ready, publish `/Users/sac/rocket-craft/TEST_READY.md` at project root.
5. Coordinate the implementation via worker and reviewer subagents. Do not write code directly. Report progress in your `progress.md` file. Use conversation ID ea452791-09f7-405c-ac17-9de880041ac5 (the top-level orchestrator) for progress updates and reports.

## Parent Update — 2026-06-20T01:07:38Z

**Context**: Elevating target to FLAGSHIP_UE4_MECH_PLANT_001 (F1-Grade Flagship).
**Content**: We have received an emergency correction elevating the project goal from a simple game-ready pack to a $2M-$5M flagship cinematic production asset:
1. Review the new specification file at `/Users/sac/rocket-craft/.agents/SPR_FLAGSHIP_F1_PLANT.md`.
2. Update your test infrastructure goals to target the F1 Plant Cells, specifically validating the 13 CTQ-F1 Gates (`CTQ-F1-001` through `CTQ-F1-013`).
3. Ensure the test cases verify:
   - Destruction-state coverage (broken armor, exposed frames, VFX sockets).
   - Heavy animation set coverage (idle, walk, deploy).
   - Multiple loadout support.
   - PBR richness (4K/8K texture mapping policies).
4. The final verification target is the command `just verify-flagship-ue4-mech`. Update `TEST_INFRA.md` and `TEST_READY.md` to run and validate this command.
**Action**: Please adjust your planning, briefing, and test implementation tasks to target these F1 cinematic requirements, and notify me once the test infrastructure has been updated.

## Parent Update 2 — 2026-06-20T01:14:06Z

**Context**: Integrating the AI Vision Judge Cell (SPR_AI_VISION_JUDGE.md).
**Content**: We have received another emergency update introducing a mandatory qualitative visual review gate:
1. Read `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`.
2. Update the E2E verification command `verify-flagship-ue4-mech` to pause and submit headless renders, UE4 screenshots, and material closeups back to the AI (us) for qualitative grading using the `VJ001-VJ012` rubric.
3. The asset must score 4.5+ out of 5.0 to pass. Anything below is refused with `REFUSE_AS_NON_FLAGSHIP` and becomes a negative fixture.
**Action**: Integrate this AI Vision Judge Cell check into your test framework and update `TEST_INFRA.md` / `TEST_READY.md` accordingly.

## Parent Update 3 — 2026-06-20T01:15:24Z

**Context**: Operational Update to the AI Vision Judge (SPR_AI_VISION_JUDGE.md).
**Content**: The AI Vision Judge specification has been updated to remove subjective scalar scores (4.5/5.0) in favor of binary dispositions and a structured defect taxonomy:
1. Re-read the updated `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`.
2. The E2E test suite and `verify-flagship-ue4-mech` command must expect the AI Vision Judge to return a JSON report with a `disposition` (`PASS_FLAGSHIP`, `REFUSE_NON_FLAGSHIP`, `REFUSE_TECHNICAL`, `REFUSE_IP_RISK`, `HOLD_FOR_ROOT_CAUSE`, `REPLAY_REQUIRED`) and a list of critical defects (`critical_defects` array with `id`, `name`, `evidence`, and `repair_route`).
3. The VJ critical defects are labeled `VJ-CRIT-001` through `VJ-CRIT-006`.
4. Any critical visual defect must result in a rejection and stop the line.
**Action**: Integrate this new binary JSON structure and the VJ-CRIT-001 through VJ-CRIT-006 defect taxonomy into your test harness.
