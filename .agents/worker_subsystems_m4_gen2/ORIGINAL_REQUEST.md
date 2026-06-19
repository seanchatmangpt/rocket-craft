## 2026-06-19T05:16:21Z

You are the Subsystem Topologies Worker (Worker) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_subsystems_m4_gen2`.
Your task is to implement the complete Subsystem Topologies schema and validation rules in `/Users/sac/.ggen/packs/ue4_ontology/`.

Specifically, you must:
1. Read the proposed files from the 3 Explorers:
   - Rendering:
     - Proposed subsystems: `/Users/sac/rocket-craft/.agents/explorer_rendering_m4_1_gen2/proposed_subsystems.ttl`
     - Proposed SHACL: `/Users/sac/rocket-craft/.agents/explorer_rendering_m4_1_gen2/proposed_validation.shacl.ttl`
     - Proposed ggen.toml additions: `/Users/sac/rocket-craft/.agents/explorer_rendering_m4_1_gen2/proposed_ggen_additions.toml`
   - Physics:
     - Proposed subsystems and SHACL: `/Users/sac/rocket-craft/.agents/explorer_physics_m4_2_gen2/analysis.md` (read sections 1, 2, 3)
   - Networking:
     - Proposed subsystems: `/Users/sac/rocket-craft/.agents/explorer_networking_m4_3_gen2/proposed_subsystems.ttl`
     - Proposed SHACL: `/Users/sac/rocket-craft/.agents/explorer_networking_m4_3_gen2/proposed_validation.shacl.ttl`
     - Proposed ggen.toml rules: `/Users/sac/rocket-craft/.agents/explorer_networking_m4_3_gen2/proposed_ggen_rules.toml`

2. Merge these proposals into:
   - Target subsystems ontology: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`
   - Target SHACL shapes: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
   - Target GGen configuration: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`

Note: Be extremely careful when merging. Make sure that prefix declarations, class/property definitions, SHACL shapes, and SPARQL rules are syntactically valid Turtle/TOML, and do not contain duplicate definitions or syntax conflicts. Make sure that any references to subclasses like `USceneComponent`, `UWorld`, `ACharacter` align with the core backbone.

3. Run the validation command:
   - `/Users/sac/rocket-craft/validate_ontology.sh`
   Ensure it exits with code 0.

4. Run the validation test runner:
   - `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh`
   Ensure all tests pass.

5. Write a detailed report to `changes.md` in your working directory.
6. Write a handoff report at `handoff.md` and send a message back to the parent orchestrator (conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3) with the path to your handoff.md.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.
