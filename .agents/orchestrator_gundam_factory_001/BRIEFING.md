# BRIEFING — 2026-06-19T11:05:00-07:00

## Mission
Guide the team to secure all gates for the GC-GUNDAM-FACTORY-001 milestone and achieve the target status: GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE.

## 🔒 My Identity
- Archetype: teamwork_preview_orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/rocket-craft/.agents/orchestrator_gundam_factory_001/
- Original parent: top-level
- Original parent conversation ID: f4d8ab35-42a8-4a47-97c7-a81a9c870304

## 🔒 My Workflow
- **Pattern**: Project Pattern
- **Scope document**: /Users/sac/rocket-craft/PROJECT.md
1. **Decompose**: Breaking the milestone into four sequential gates corresponding to the requirements:
   - Milestone 1: Headless Rust Pre-UE4 Verification
   - Milestone 2: ggen Manufacturing
   - Milestone 3: UE4 HTML5/WASM Projection
   - Milestone 4: Playwright Visual Actuation
   - Milestone 5: Final Report & Verification
2. **Dispatch & Execute**:
   - For exploration of existing codebase: dispatch to Explorer.
   - For implementation and builds: dispatch to Worker.
   - For verification of each gate: dispatch to Reviewer and Challenger.
   - For integrity checks: dispatch to Forensic Auditor.
3. **On failure**:
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: Self-succeed when spawn count >= 16.
- **Work items**:
  1. Explore codebase & gaps (pending)
  2. Implement & run Gate 1 pre-UE4 verifier (pending)
  3. Run Gate 2 ggen manufacturing (pending)
  4. Build & package Gate 3 UE4 WASM projection (pending)
  5. Run Gate 4 Playwright visual delta verifications (pending)
  6. Emit final report and verify all receipts (pending)
- **Current phase**: 1
- **Current focus**: Explore codebase & gaps

## 🔒 Key Constraints
- Never write, modify, or create source code files directly.
- Never run build/test commands directly — use subagents.
- Distrust first responses; perform rigorous cross-checking.
- If Forensic Auditor reports INTEGRITY VIOLATION, halt the pipeline.
- Never reuse a subagent after it has delivered its handoff.

## Current Parent
- Conversation ID: f4d8ab35-42a8-4a47-97c7-a81a9c870304
- Updated: not yet

## Key Decisions Made
- Decomposed the project into 6 milestones corresponding to the required gates and final report.
- Propagated top-priority architectural directive (Wil van der Aalst + John Carmack synthesis) to the worker subagent to enforce process rigor (Petri net trace conformance) and data-oriented/SoA performance (SIMD chunked execution, zero-stub packaging).
- Propagated command update to worker subagent: use `ggen sync --manifest <path> --audit` as `ggen generate` is obsolete.


## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| explorer_gundam_factory_001 | teamwork_preview_explorer | Explore codebase & gaps | completed | 77d0472c-6fd7-431c-94dd-c29f6a117683 |
| worker_gundam_factory_001 | teamwork_preview_worker | Implement walkthrough changes | completed | 729c34be-d2ab-4775-ae0b-33f764b673db |
| worker_gundam_factory_002 | teamwork_preview_worker | Packaging and E2E visual verification | in-progress | b1b88d54-dc49-4a82-8228-6467b612c390 |

## Succession Status
- Succession required: no
- Spawn count: 3 / 16
- Pending subagents: b1b88d54-dc49-4a82-8228-6467b612c390
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: task-103
- Safety timer: none

## Artifact Index
- /Users/sac/rocket-craft/.agents/orchestrator_gundam_factory_001/plan.md — Project plan
- /Users/sac/rocket-craft/.agents/orchestrator_gundam_factory_001/progress.md — Execution progress
- /Users/sac/rocket-craft/.agents/orchestrator_gundam_factory_001/context.md — Context and environment info
