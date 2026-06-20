# Context - E2E Testing for UE4 Universal RDF Mapping

## System Information
- Project Root: `/Users/sac/rocket-craft`
- Target Pack Directory: `/Users/sac/.ggen/packs/ue4_ontology`
- Parent Conversation ID: `4f79cb22-2adb-466d-9e20-d8baef6e934d`
- Working Directory: `/Users/sac/rocket-craft/.agents/sub_orch_e2e_testing`

## Inputs
- `PROJECT.md` at project root
- `ORIGINAL_REQUEST.md` (original parent request)

## Outputs Required
- `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- `/Users/sac/rocket-craft/TEST_INFRA.md`
- `/Users/sac/rocket-craft/TEST_READY.md`
- Validation runner / verification execution
- Deliver handoff.md and completion report

## Rules & Constraints
- Do not modify or create project source files directly from the orchestrator.
- Do not run terminal validation commands directly from the orchestrator.
- Delegate all file writing and command executions to `teamwork_preview_worker`.
