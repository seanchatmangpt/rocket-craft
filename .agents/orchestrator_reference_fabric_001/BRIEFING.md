# BRIEFING — 2026-06-20T00:20:00Z

## Mission
Build a zero-human, ggen-driven manufacturing loop that turns the reference image into a deterministic, generated USD/MaterialX/texture/gameplay-ready asset approximation.

## 🔒 My Identity
- Archetype: teamwork_preview_orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/rocket-craft/.agents/orchestrator_reference_fabric_001
- Original parent: parent
- Original parent conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /Users/sac/rocket-craft/PROJECT.md
1. **Decompose**: Run the iteration loop (Explorer -> Worker -> Reviewer -> Challenger -> Auditor) to design, manufacture, and verify the reference fabric manufacturing loop.
2. **Dispatch & Execute**:
   - **Direct (iteration loop)**: Iterate with specialists to implement visual extraction, USD/MaterialX templates, headless rendering, comparison, and the gap checker.
3. **On failure** (in this order):
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: Self-succeed at 16 spawns.
- **Work items**:
  1. Define project layout and initialize PROJECT.md [pending]
  2. Spawn Explorer to analyze codebase and plan implementation [pending]
  3. Spawn Worker to implement visual extraction and comparison tools [pending]
  4. Spawn Worker to define ontology, SPARQL, templates and run ggen sync [pending]
  5. Spawn Worker to implement headless rendering and gap checking scripts [pending]
  6. Verify all deliverables using Reviewers, Challengers, and Forensic Auditor [pending]
- **Current phase**: 1
- **Current focus**: Define project layout and initialize PROJECT.md

## 🔒 Key Constraints
- All source files, USD geometry, and MaterialX materials must be generated from Turtle ontology files (.ttl), SPARQL queries (.rq), and Tera templates (.usd.tera, .mtlx.tera) via 'ggen sync'. No hand-written block proxies or code.
- Implement the commands: extract_reference_visual_targets.py, ggen sync, render_reference_fabric.py, compare_reference_render.py, asset_fabric_gap_check.py.
- Ensure that the generated assets can be rendered headlessly and compared against the reference image.
- Compare the generated render to the reference and emit a visual gap report.
- Follow the TAI Status Reporting Format and update 'progress.md' at least once every 8 minutes.
- Maintain 'plan.md' and 'context.md' in your working directory.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh.

## Current Parent
- Conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d
- Updated: not yet

## Key Decisions Made
- Decompose the GC-MECH-ASSET-FABRIC-001 milestone into a sequence of worker tasks to ensure logical progression.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| explorer_001 | teamwork_preview_explorer | Explore codebase, reference images, and render capabilities | completed | d42f41fd-1487-41b5-9cf2-9ec3459f4012 |
| worker_setup_cv | teamwork_preview_worker | Set up directories, copy reference image, and implement CV targets extraction | completed | 23f56ed3-583d-4d22-bc8b-0692d1b2590c |
| worker_gen | teamwork_preview_worker | Implement ontology, queries, templates, update ggen.toml, and run ggen sync | completed | b3ad57b6-7480-42b4-ae32-ff6894fd558a |
| worker_render | teamwork_preview_worker | Implement textures generation, rendering, comparison, OCEL, and receipts | completed | e18e5ba1-c131-4f32-92f0-1707c968590a |
| worker_gapcheck | teamwork_preview_worker | Implement asset_fabric_gap_check.py, run validations, and compile gap reports | completed | ffa4a985-ba62-45d9-9a00-f98f3c46264a |
| auditor_001 | teamwork_preview_auditor | Perform forensic audit of GC-MECH-ASSET-FABRIC-001 | completed | fa4f26cd-6858-4c10-85d6-4f645738bca6 |
| worker_morphology | teamwork_preview_worker | Implement GC-MECH-ASSET-FABRIC-001B visual morphology convergence loop | in-progress | 57843ca4-33c2-4499-af1c-ee6ea5418afa |

## Succession Status
- Succession required: no
- Spawn count: 7 / 16
- Pending subagents: [57843ca4-33c2-4499-af1c-ee6ea5418afa]
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: not started
- Safety timer: none

## Artifact Index
- /Users/sac/rocket-craft/.agents/orchestrator_reference_fabric_001/ORIGINAL_REQUEST.md — Verbatim copy of user request
- /Users/sac/rocket-craft/.agents/orchestrator_reference_fabric_001/BRIEFING.md — Persistent memory index
