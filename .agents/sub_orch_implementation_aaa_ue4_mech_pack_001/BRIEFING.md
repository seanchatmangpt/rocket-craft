# BRIEFING — 2026-06-20T01:03:33Z

## Mission
Implement the parallel generation swarms and source law for flagship cinematic target FLAGSHIP_UE4_MECH_PLANT_001.

## 🔒 My Identity
- Archetype: teamwork_preview_sub_orch
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001
- Original parent: parent
- Original parent conversation ID: ea452791-09f7-405c-ac17-9de880041ac5

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/SCOPE.md
1. **Decompose**: Decompose the implementation into milestones.
2. **Dispatch & Execute** (pick ONE):
   - **Direct (iteration loop)**: Iterate: Explorer -> Worker -> Reviewer -> Challenger -> Auditor -> Gate
   - **Delegate (sub-orchestrator)**: [TBD]
3. **On failure** (in this order):
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: Self-succeed at spawn count >= 16 and all subagents complete. Spawn successor via teamwork_preview_sub_orch.
- **Work items**:
  1. Milestone 1: F1 Geometry & Morphology [in-progress]
  2. Milestone 2: Modular USD Identity [pending]
  3. Milestone 3: Cinematic Lookdev & 4K/8K Textures [pending]
  4. Milestone 4: Rigging, VFX Sockets & Bounds [pending]
  5. Milestone 5: Destruction & Animation States [pending]
  6. Milestone 6: Multiple Weapon Loadouts [pending]
  7. Milestone 7: IP-Distance Engine [pending]
  8. Milestone 8: UE4 Import & Cooking Automation [pending]
  9. Milestone 9: Verification & Receipts [pending]
- **Current phase**: 2B (Iteration Loop: F1 Patches & Controlled DOE Run)
- **Current focus**: Milestone 1 (F1 Geometry & Morphology & DOE Run)

## 🔒 Key Constraints
- Coordinate via worker, reviewer, challenger subagents. Do not write code directly.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh

## Current Parent
- Conversation ID: ea452791-09f7-405c-ac17-9de880041ac5
- Updated: 2026-06-20T01:07:45Z (elevated to F1 FLAGSHIP)

## Key Decisions Made
- Enforce strict modular USD identity (USD303-312) in parallel with F1 chassis/surface/rig patches.
- Require a 3-seed smoke batch and 12-section report to prove negative fixtures before launching the 100-seed DOE.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| worker_baseline | teamwork_preview_worker | Run baseline verification | completed | 3ac2e1f5-6317-4cee-8e88-255350e63708 |
| explorer_geom_1 | teamwork_preview_explorer | Explore F1 geometry & morphology strategy | completed | 70d2ffd0-8b68-4857-b039-ed53af16e4de |
| explorer_geom_2 | teamwork_preview_explorer | Explore F1 geometry & morphology strategy | completed | 70cde5a9-50c4-473a-8720-4d084b9edfce |
| explorer_geom_3 | teamwork_preview_explorer | Explore F1 geometry & morphology strategy | completed | 19a98cf7-365c-4c82-b866-01fa4ae802d6 |
| worker_doe | teamwork_preview_worker | Implement F1 patches & Controlled DOE Run | completed | a67b2faf-3ec4-4989-af2f-807a7839b3c6 |
| worker_packaging | teamwork_preview_worker | Compile and package final flagship mecha asset pack | in-progress | 5fe27df8-021b-4537-845b-a15d1c48c012 |

## Succession Status
- Succession required: no
- Spawn count: 6 / 16
- Pending subagents: [5fe27df8-021b-4537-845b-a15d1c48c012]
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: task-15
- Safety timer: task-287

## Artifact Index
- /Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/ORIGINAL_REQUEST.md — Original request
- /Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/progress.md — Progress log
- /Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/SCOPE.md — Scope / milestones
