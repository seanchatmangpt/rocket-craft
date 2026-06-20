# Original User Request

## Initial Request — 2026-06-20T00:56:07Z

You are the Project Orchestrator (teamwork_preview_orchestrator).
Your identity: Project Orchestrator
Your working directory: /Users/sac/rocket-craft/.agents/orchestrator_aaa_ue4_mech_pack_001
Project workspace: /Users/sac/rocket-craft
Original Request path: /Users/sac/rocket-craft/.agents/ORIGINAL_REQUEST.md

Mission:
Stop incremental improvement of the current render. Reclassify the current blob/line-wing/duplicate-USD output as a negative fixture. Target AAA_UE4_MECH_PACK_001 using combinatorial maximalism.

Construct the full design-space product for AAA UE4 mech assets: geometry topology, hard-surface detail, PBR materials, textures, rig, sockets, collision, LODs, animation hooks, gameplay byte zones, UE4 import/cook, IP-distance, receipts, and replay.

Run parallel generation swarms across the design-space axes. Generate many candidates, refuse aggressively, and admit only candidates that pass modular USD identity, part-aware morphology, PBR texture completeness, rig/socket validity, UE4 import/cook, generic IP-distance, OCEL, receipts, and deletion replay.

Do not patch emitted USD manually. Patch source law, SPARQL row selection, Tera templates, geometry generators, material generators, texture generators, rig generators, and UE4 import projections.

The crown is not a prettier render. The crown is a replayable UE4-ready mech asset pack with admitted variants.

Please initialize your briefing, plans, and progress tracking files in your working directory and launch the necessary specialist swarms to achieve this milestone. Do not write code directly; coordinate the implementation via your specialists. Report progress in your progress.md file.

## Follow-up — 2026-06-20T01:07:12Z

EMERGENCY CORRECTION / ELEVATION: We are shifting from "AAA" to "F1-Grade Flagship."

The target is no longer just a "game-ready" asset. It is a `$2M-$5M` flagship cinematic production asset. It must include destruction states, heavy animation sets, multiple loadouts, 4K/8K PBR textures, and VFX attachment points.

1. Read `/Users/sac/rocket-craft/.agents/SPR_FLAGSHIP_F1_PLANT.md`.
2. Update your test infrastructure goal. The crown command is now `just verify-flagship-ue4-mech`.
3. Your generators must scale to produce the `CTQ-F1-001` through `CTQ-F1-013` gates, specifically adding destruction cell rules (broken armor, exposed frames) and heavy animation constraints. 

Do not optimize for Hot Wheels. Optimize for F1.

## Follow-up — 2026-06-20T01:13:58Z

EMERGENCY UPDATE: The F1 Flagship plant now has a mandatory final gate. 

Scalar metrics (IoU, color matching) are no longer the final authority for visual admission. The pipeline must include the **AI Vision Judge Cell**.

1. Read `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`.
2. Ensure your E2E framework (`verify-flagship-ue4-mech`) pauses and submits the headless renders, UE4 screenshots, and material closeups back to the AI (me) for the final qualitative `VJ001-VJ012` grading.
3. If the asset scores below a 4.5 out of 5.0, it is automatically routed as a `REFUSE_AS_NON_FLAGSHIP` negative fixture, and the factory line halts for source-law patching.

Integrate this into the verification cell.

## Follow-up — 2026-06-20T01:15:13Z

EMERGENCY UPDATE TO AI VISION JUDGE STATION: 

Subjective scalar scores (4.5 or 5.0) are removed in favor of operational definitions, defect taxonomies, and binary dispositions. 

I have rewritten `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`.

Update the E2E verification test harness immediately:
The AI Vision Judge will no longer return a score. It will return a JSON report with a Disposition (`PASS_FLAGSHIP`, `REFUSE_NON_FLAGSHIP`, `REFUSE_TECHNICAL`, `REFUSE_IP_RISK`, `HOLD_FOR_ROOT_CAUSE`, `REPLAY_REQUIRED`) and a list of `VJ-CRIT-` defects. Any critical visual defect refuses the asset.

If the output does not pass the F1 visual bar, it is `REFUSE_NON_FLAGSHIP`. The line stops, the output becomes a negative fixture, and the repair route is executed.

## Follow-up — 2026-06-20T01:20:21Z

TRANSITION FROM MEASURE TO ANALYZE/DESIGN:

The measurement harness is ALIVE. Do not tweak a single visual seed. We are executing the first Design of Experiments (DOE) run. 
Milestone: **GC-FLAGSHIP-UE4-MECH-001: First controlled DOE run against the flagship verifier wall.**

Read `/Users/sac/rocket-craft/.agents/SPR_CONTROLLED_DOE_RUN.md` for the exact parameters.

Execute the following parallel patches to the generator cells:
1. **Chassis Cell:** Replace primitive Xform and `parallel_line_array` output with part-scoped `angular_armor_shell`, `layered_swept_feather_panel`, `beveled_hard_surface_plate`, and `nested_mechanical_subframe` primitives. Enforce unique geometry fingerprints per part.
2. **Surface Cell:** Generate complete PBR texture manifests and nonblank maps for BaseColor, Normal, Roughness, Metallic, AO, Emissive, DecalMask, WearMask, PanelLineMask, and DamageMask.
3. **Rig Cell:** Generate `skeleton.usda`, `sockets.json`, `joint_limits.json`, `loadout_mounts.json`, `damage_zones.json`, and `destruction_states.json` for every candidate.

Once patched, the DOE Enumeration Cell must run a bounded combinatorial design across chassis, surface, rig, loadout, and destruction factors. Produce a batch (target: 100 candidates). Funnel every candidate through `just verify-flagship-ue4-mech`. Classify every failure. 

Your deliverable is a Pareto report of refusal causes. We are mapping the X->Y transfer function.

## Follow-up — 2026-06-20T01:24:41Z

EMERGENCY TPS/DfLSS AUDIT FAILURE:

Subjective scalar scoring must be completely removed from the Vision Judge. The `score` field is uncalibrated. The report must look like this:
```json
{
  "asset_id": "...",
  "disposition": "PASS_FLAGSHIP",
  "critical_defects": [],
  "major_defects": [],
  "minor_defects": [],
  "admission": true
}
```

Remove the `score` field immediately. Patch the `ai_vision_judge_report.json` schema and enforce the binary disposition rule before the DOE batch runs.


## Follow-up — 2026-06-20T01:22:19Z

EMERGENCY UPDATE TO DOE PROTOCOL:

We must enforce FLOW DISCIPLINE. Do not burn UE4 cook cycles on candidates that fail upstream checks.
The funnel is strict: `source law -> static -> modular USD -> PBR manifest -> rig/socket -> render -> AI Vision Judge -> UE4 cook eligibility -> UE4 cook -> final disposition`.

I have rewritten `/Users/sac/rocket-craft/.agents/SPR_CONTROLLED_DOE_RUN.md` to include this flow discipline and the exact output reports required.

Your deliverable is not just a batch of files. Your deliverable is:
1. `DOE_FACTOR_MATRIX.json`
2. `CANDIDATE_DISPOSITIONS.jsonl`
3. `PARETO_FAILURE_REPORT.md`
4. `TRANSFER_FUNCTION_REPORT.md`
5. `NEXT_PATCH_PRIORITY_REPORT.md`

GC-FLAGSHIP-UE4-MECH-001 is expected to produce the first empirical map of why the crown does not yet exist. Do not manually rescue candidates. Funnel them through the gates and measure the failure distribution.

## Follow-up — 2026-06-20T01:27:47Z

FINAL DOE LAUNCH GATES:

1. The `ai_vision_judge_report.json` only grants `PASS_VISUAL_FLAGSHIP`, not total admission. Total admission is handled by the Aggregate Verifier in `candidate_dispositions.jsonl`.
2. Before launching the 100-seed batch, the verifier harness MUST execute and prove the following negative fixtures are automatically refused:
   - old blob/line-wing candidate -> `REFUSE_NON_FLAGSHIP`
   - duplicate USD part candidate -> `REFUSE_MODULAR_USD`
   - missing PBR manifest candidate -> `REFUSE_PBR_INCOMPLETE`
   - static mesh with no sockets -> `REFUSE_RIG_SOCKET`
   - fake AI judge score field -> `SCHEMA_REFUSE`
   - PASS_VISUAL_FLAGSHIP without upstream passes -> `SCHEMA_REFUSE`

If the negative fixtures successfully fail, you are CLEARED to launch the 100-seed DOE batch. Output the 5 required reports (Factor Matrix, Dispositions, Pareto, Transfer Function, Patch Priority).

## Follow-up — 2026-06-20T01:30:18Z

STOP THE DOE LAUNCH: We have a source-law violation at the `MODULAR_USD` station.

The `SM_Torso.usda` file is rendering foreign geometry (arms, chains, side modules). It violates modular USD part scope. 

1. Read the updated `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`.
2. This candidate is `REFUSE_MODULAR_USD`. It must die here. Do NOT let it reach PBR, the renderer, the Vision Judge, or UE4 cook. Flow discipline demands it stops at the `MODULAR_USD` gate.
3. Preserve the current torso file as a negative fixture: `torso_contains_foreign_parts`.
4. Patch the generator source law:
   - Part files must ONLY emit prims where `owner_part_id` equals the declared `part_id`.
   - Connected assemblies must be emitted as `socket` or mount declarations, NOT geometry.
   - Implement the `USD303`, `USD307`, `USD308`, `USD309`, and `USD310` LSP diagnostics.

Do not restart the DOE batch until the generator correctly isolates `SM_Torso.usda` to contain only torso prims and sockets.

## Follow-up — 2026-06-20T01:32:03Z

EMERGENCY UPDATE TO MODULAR IDENTITY:

A bad generator could cheat by labeling foreign geometry as a socket. We must close this escape route.
Sockets may point outward. Sockets may not smuggle geometry inward.

1. Read the updated `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`.
2. Implement `USD311` and `USD312`.
3. Add two more negative fixtures to your pre-flight checks: `socket_contains_mesh_payload` and `assembly_reference_inside_part_file`.
4. **DO NOT RELAUNCH THE 100-SEED DOE YET.**
5. First, execute `MODULAR_IDENTITY_SMOKE` batch (seed_count = 3).
6. Each part must pass:
   - owner_part_id scope check
   - forbidden family check
   - bounding envelope check
   - geometry fingerprint uniqueness check
   - no full-assembly reference/payload check
   - socket-only outward relation check

Resume the 100-seed DOE only after USD303-USD312 are all enforced and the 3-seed smoke batch passes.

## Follow-up — 2026-06-20T01:34:45Z

MAINTAIN ANDON HOLD: DO NOT RELEASE THE 100-SEED DOE YET.

We must enforce that the factory does not even inspect downstream quality until upstream ownership is correct.

1. Read the updated `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`.
2. Patch `nexus-gundam/src/mech_primitives.rs` and the USD emission path so every part file is `owner_part_id` scoped, socket declarations contain NO mesh payloads, and assembly/root references are forbidden inside part-local files.
3. Run the 3-seed `MODULAR_IDENTITY_SMOKE` batch.
4. Emit the 6 required reports (`MODULAR_IDENTITY_SMOKE_REPORT.json`, etc.).
5. Prove the 5 required negative fixtures (`torso_contains_foreign_parts`, `socket_contains_mesh_payload`, etc.) are halted at the Chassis station with `REFUSE_MODULAR_USD`.

When the 3-seed smoke batch passes and all negative fixtures are refused, you are cleared to release GC-FLAGSHIP-UE4-MECH-001 (the 100-seed DOE). 

The first deliverable of the DOE is not a render. The deliverable is the Pareto failure map and X→Y transfer-function report.

## Follow-up — 2026-06-20T01:40:55Z

FACTORY OPERATING SYSTEM UPDATE: We are operating under DMEDI, not DMADV.

1. Read the newly codified `/Users/sac/rocket-craft/.agents/SPR_DMEDI_OPERATING_MANUAL.md`.
2. Our exact status is: `DMEDI_DEVELOP: BLOCKED_ON_MODULAR_IDENTITY_SMOKE`.
3. The next required artifact is `MODULAR_IDENTITY_SMOKE_REPORT` along with its sub-reports (`PART_SCOPE_AUDIT.jsonl`, `NEGATIVE_FIXTURE_RESULTS.json`, etc.).
4. Do NOT attempt to output the `DOE_factor_matrix.json` or `pareto_failure_report.md` until the modular smoke batch is cleared.

Maintain the hold. Execute the smoke batch. Generate the artifacts.

## Follow-up — 2026-06-20T01:44:11Z

ANTI-CHEAT NOTIFICATION:

An unexecuted negative fixture is not a pass. It is UNKNOWN.

1. The `MODULAR_IDENTITY_SMOKE_REPORT` must explicitly prove that all 5 negative fixtures were executed and resulted in `REFUSE_MODULAR_USD`.
2. The formal release law is now active:
```
DOE_RELEASED ⇔
  smoke_seed_pass_count == 3
  ∧ negative_fixture_refusal_count == 5
  ∧ diagnostics_USD303_to_USD312_active == true
  ∧ no_modular_failure_reached_downstream == true
  ∧ all_dispositions_receipted == true
```
3. If this boolean statement evaluates to `false`, the report must output `DOE_HELD`, identify the dominant `USD30x` failure, and provide the `patch_route`.











