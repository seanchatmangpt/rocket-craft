## 2026-06-18T22:48:20Z

You are the Cooking and Packaging Typestates Worker (Worker) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_typestates_m5_gen2`.
Your task is to implement the complete Cooking, Linking, and Packaging Typestates schema and validation rules in `/Users/sac/.ggen/packs/ue4_ontology/`.

Specifically, you must:
1. Read the proposed files from the 3 Explorers:
   - Cooking (Explorer 1): `/Users/sac/rocket-craft/.agents/explorer_cooking_m5_1_gen2/analysis.md`
   - Linking (Explorer 2): `/Users/sac/rocket-craft/.agents/explorer_linking_m5_2_gen2/analysis.md`
   - Packaging (Explorer 3): `/Users/sac/rocket-craft/.agents/explorer_packaging_m5_3_gen2/analysis.md`

2. Merge these proposals into:
   - Target typestates ontology: `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`
   - Target SHACL shapes: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
   - Target GGen configuration: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`

Note: When merging, ensure strict OWL 2 DL compliance:
- Use `owl:ObjectProperty` or `owl:DatatypeProperty` for properties instead of raw `rdf:Property` (though typestate properties like `hasCookingState`, `hasLinkingState`, and `hasPackagingState` should have both `owl:ObjectProperty` and `rdf:Property` to satisfy GGen validator).
- Verify all prefixes are complete.
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
