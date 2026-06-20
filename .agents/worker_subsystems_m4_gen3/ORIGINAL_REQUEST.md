## 2026-06-19T06:06:24Z

You are the Subsystem Topologies Worker (Worker) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_subsystems_m4_gen3`.
Your task is to implement the complete Subsystem Topologies schema and validation rules in `/Users/sac/.ggen/packs/ue4_ontology/`.

Specifically, you must:
1. Read the proposed files from the 3 Explorers:
   - Rendering: `/Users/sac/rocket-craft/.agents/explorer_rendering_m4_1_gen3/analysis.md`
   - Physics: `/Users/sac/rocket-craft/.agents/explorer_physics_m4_2_gen3/analysis.md`
   - Networking: `/Users/sac/rocket-craft/.agents/explorer_networking_m4_3_gen3/analysis.md`

2. Merge these proposals into:
   - Target subsystems ontology: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`
   - Target SHACL shapes: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
   - Target GGen configuration: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`

Note: Be extremely careful when merging. Make sure that prefix declarations, class/property definitions, SHACL shapes, and SPARQL rules are syntactically valid Turtle/TOML, and do not contain duplicate definitions or syntax conflicts. Make sure that any references to subclasses like `USceneComponent`, `UWorld`, `ACharacter` align with the core backbone.
Ensure OWL 2 DL compliance:
- Use `owl:ObjectProperty` or `owl:DatatypeProperty` for properties instead of raw `rdf:Property`.
- Be careful with `FILTER NOT EXISTS` queries in custom SPARQL rules: to avoid the empty-graph bug in `ggen`'s SPARQL engine, always bind a guaranteed-to-exist triple pattern like `?ontology a owl:Ontology .` outside of the `FILTER NOT EXISTS` block, so that the ASK rule does not crash on a clean ontology.
- Ensure that the Projection Law is strictly respected: do not generate WebGL binary graphics assets (meshes, textures, etc.) from the ontology. Instead, model metadata, walkthrough coordinates, DataTables, and output path configuration variables. VaRest calls should be forbidden on statically baked targets.

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
