# Original User Request

## 2026-06-18T17:33:36-07:00

You are the E2E Testing Orchestrator for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_e2e_testing`.
Please initialize your plan.md, progress.md, and context.md in your working directory.

## Objective
Design and implement a comprehensive test suite and testing infrastructure for the UE4 RDF Mapping project. This must be requirement-driven and opaque-box, verifying the validity of Turtle ontologies and SPARQL queries.

## Scope Boundaries
- DO NOT implement the main UE4 ontology files (e.g. core.ttl, reflection.ttl, etc.).
- Focus entirely on test infrastructure, test case definitions, validation configuration, and test runner execution.

## Input Information
- PROJECT.md: `/Users/sac/rocket-craft/PROJECT.md`
- Verbatim Request: `/Users/sac/rocket-craft/.agents/orchestrator_ue4/ORIGINAL_REQUEST.md`
- Target pack directory: `/Users/sac/.ggen/packs/ue4_ontology`

## Output Requirements
1. Create `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` configured to validate all Turtle (.ttl) files in `/Users/sac/.ggen/packs/ue4_ontology`.
2. Write a comprehensive `TEST_INFRA.md` at `/Users/sac/rocket-craft/TEST_INFRA.md` detailing the test suite features, boundaries, and test case mapping following the 4-tier acceptance methodology.
3. Write a validation harness or script (if needed) to run the `ggen sync --validate-only` command.
4. Publish `TEST_READY.md` at `/Users/sac/rocket-craft/TEST_READY.md` summarizing the test suite coverage across Tiers 1-4.
5. Deliver your completion report and handoff.md in your working directory.

## Completion Criteria
- ggen.toml is correctly written in `/Users/sac/.ggen/packs/ue4_ontology`.
- TEST_INFRA.md and TEST_READY.md are written at the project root.
- The validation command is verified to run successfully (even if it initially fails due to missing ontology files, the tool execution itself must work).

Communicate your status and handoff using send_message to Conversation ID: 4f79cb22-2adb-466d-9e20-d8baef6e934d (Parent name: parent).
