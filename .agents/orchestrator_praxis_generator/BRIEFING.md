# BRIEFING — 2026-06-22T05:28:00Z

## Mission
Catalog rocket-craft/lsp-max, upgrade praxis boilerplate generator, and verify compilation/structural conformance.

## 🔒 My Identity
- Archetype: teamwork_preview_orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/rocket-craft/.agents/orchestrator_praxis_generator
- Original parent: parent
- Original parent conversation ID: 5494bccd-8d38-4e24-835b-b69403255a0f

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /Users/sac/rocket-craft/.agents/orchestrator_praxis_generator/PROJECT.md
1. **Decompose**: Decomposed by modular tasks into 4 milestones: Exploration & Catalog, Generator Upgrade, Compilation Verification, and Programmatic Conformance verification.
2. **Dispatch & Execute**:
   - **Delegate (sub-orchestrator)**: When an item is too large, spawn a sub-orchestrator for it.
   - **Direct (iteration loop)**: Iterate Explorer -> Worker -> Reviewer -> Challenger -> Auditor for simple milestones.
3. **On failure** (in this order):
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: Self-succeed at 16 spawns, write handoff.md, spawn successor.
- **Work items**:
  1. M1: Ecosystem Catalog and Abstraction [pending]
  2. M2: Praxis Generator Upgrade [pending]
  3. M3: E2E Project Emission and Compilation [pending]
  4. M4: Programmatic Conformance Verification [pending]
- **Current phase**: 1
- **Current focus**: M1: Ecosystem Catalog and Abstraction

## 🔒 Key Constraints
- NEVER write, modify, or create source code files directly.
- NEVER run build/test commands yourself — require workers to do so.
- You MAY use file-editing tools ONLY for metadata/state files (.md) in your .agents/ folder.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh

## Current Parent
- Conversation ID: 5494bccd-8d38-4e24-835b-b69403255a0f
- Updated: not yet

## Key Decisions Made
- Chose Project Pattern with 4 milestones to map the exploration, upgrade, compilation, and programmatic verification phases.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| explorer_m1 | teamwork_preview_explorer | M1: Ecosystem Catalog and Abstraction | completed | 0233a26c-018d-4939-9fdf-b93c0296cf55 |
| worker_praxis_upgrade | teamwork_preview_worker | M2: Praxis Generator Upgrade | completed | 59e4ee64-7bce-45a9-acff-47ad84c719cb |
| reviewer_praxis_upgrade | teamwork_preview_reviewer | M3: Code correctness and design review | completed | 0e459506-04a2-4d83-9687-cbbdac277f0a |
| challenger_praxis_upgrade | teamwork_preview_challenger | M3/M4: Compilation, test run & conformance check | completed | 26e93652-f1ba-4de2-9f80-7d9d54d694f7 |
| worker_praxis_upgrade_remediation | teamwork_preview_worker | M2-M4: Praxis upgrades remediation | completed | a8bce0c1-c0cd-418f-a23e-c5e084ea8e97 |
| auditor_praxis_upgrade | teamwork_preview_auditor | M3/M4: Forensic integrity audit | completed | eaf66b91-b1f7-4751-be56-3c43809722f5 |

## Succession Status
- Succession required: no
- Spawn count: 6 / 16
- Pending subagents: none
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: none
- Safety timer: none
- On succession: kill all timers before spawning successor
- On context truncation: run manage_task(Action="list") — re-create if missing

## Artifact Index
- /Users/sac/rocket-craft/.agents/orchestrator_praxis_generator/BRIEFING.md — Persistent memory index
- /Users/sac/rocket-craft/.agents/orchestrator_praxis_generator/progress.md — Heartbeat and detailed steps progress
- /Users/sac/rocket-craft/.agents/orchestrator_praxis_generator/PROJECT.md — Global project plan and milestones
