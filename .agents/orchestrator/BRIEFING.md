# BRIEFING — 2026-06-18T12:51:19-07:00

## Mission
Audit the rocket-craft project to scan, identify, and report all LLM-generated fake, cheat, or mock artifacts that falsely claim completion of the genuine Combinatorial Maximalist requirements.

## 🔒 My Identity
- Archetype: teamwork_preview_orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/rocket-craft/.agents/orchestrator
- Original parent: parent
- Original parent conversation ID: 773cb9ad-30c9-4762-b7a8-5055e76bc8de

## 🔒 My Workflow
- **Pattern**: Project Pattern
- **Scope document**: /Users/sac/rocket-craft/.agents/orchestrator/PROJECT.md
1. **Decompose**: Decompose the task into milestones.
2. **Dispatch & Execute**: Spawn subagents for exploration, implementation, review, and audit.
3. **On failure** (in this order):
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: self-succeed at spawn count >= 16.
- **Work items**:
  1. Exploration & Architecture [done]
  2. Implementation of GMF MUD walkthrough [done]
  3. Verification & Review [done]
  4. Forensic Audit [done]
  5. Counterfeit Scan [done]
  6. Counterfeit Report Generation [in-progress]
  7. Verification of Report [pending]
- **Current phase**: 2
- **Current focus**: Counterfeit Report Generation

## 🔒 Key Constraints
- NEVER write, modify, or create source code files directly.
- NEVER run build/test commands yourself — require workers to do so.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh
- Do not modify or delete any files during the scan.

## Current Parent
- Conversation ID: 773cb9ad-30c9-4762-b7a8-5055e76bc8de
- Updated: 2026-06-18T12:51:19-07:00

## Key Decisions Made
- Initialized plan and project milestones for the Counterfeit Artifact Audit task.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|---|---|---|---|---|
| 92d8abc1-17d9-40c5-950d-27c785306298 | teamwork_preview_explorer | Codebase Research | completed | 92d8abc1-17d9-40c5-950d-27c785306298 |
| dbd2936b-e79a-44df-8b60-a8a0571c551d | teamwork_preview_worker | GMF MUD Walkthrough Impl | completed | dbd2936b-e79a-44df-8b60-a8a0571c551d |
| d26a8719-4b93-4623-b06e-73fa68d8f1f4 | teamwork_preview_reviewer | Code Review | completed | d26a8719-4b93-4623-b06e-73fa68d8f1f4 |
| f280b41d-2262-442d-bf00-0bb109f3c3db | teamwork_preview_challenger | Adversarial Verification | completed | f280b41d-2262-442d-bf00-0bb109f3c3db |
| 023fb6c3-85a2-42af-80f5-cead4a2d50f2 | teamwork_preview_auditor | Forensic Audit | completed | 023fb6c3-85a2-42af-80f5-cead4a2d50f2 |
| a4729a91-7ef1-47b9-8782-2ee7ca0b5627 | teamwork_preview_explorer | Counterfeit Scan | completed | a4729a91-7ef1-47b9-8782-2ee7ca0b5627 |
| ffb3ced9-b151-4d3d-9c18-a2d42a0c5d5e | teamwork_preview_worker | Counterfeit Report Generation | pending | ffb3ced9-b151-4d3d-9c18-a2d42a0c5d5e |

## Succession Status
- Succession required: no
- Spawn count: 7 / 16
- Pending subagents: ffb3ced9-b151-4d3d-9c18-a2d42a0c5d5e
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: c1de2f14-f413-4e88-a05c-5dad1285c6e2/task-19
- Safety timer: none
- On succession: kill all timers before spawning successor
- On context truncation: run `manage_task(Action="list")` — re-create if missing

## Artifact Index
- /Users/sac/rocket-craft/.agents/orchestrator/progress.md — liveness heartbeat and state checkpoint
- /Users/sac/rocket-craft/.agents/orchestrator/plan.md — execution plan
- /Users/sac/rocket-craft/.agents/orchestrator/context.md — system context
- /Users/sac/rocket-craft/.agents/orchestrator/BRIEFING.md — briefing document
- /Users/sac/rocket-craft/.agents/orchestrator/PROJECT.md — global project index
