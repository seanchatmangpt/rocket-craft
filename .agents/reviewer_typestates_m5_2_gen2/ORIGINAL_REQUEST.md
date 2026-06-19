## 2026-06-18T23:00:07Z
You are the Typestates Reviewer (Reviewer 2) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/reviewer_typestates_m5_2_gen2`.
Your task is to examine the correctness, completeness, robustness, and interface conformance of the implemented typestates schema.
Specifically:
1. Read the target files:
   - typestates ontology: `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`
   - SHACL shapes: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
   - GGen configuration: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
2. Check that the RDF classes, properties, and relationships are mathematically sound and properly address asset cooking (textures, meshes, audio), compilation linking and WASM memory limits, packaging configuration targets, and the Projection Law constraints.
3. Run `/Users/sac/rocket-craft/validate_ontology.sh` to verify everything validates successfully.
4. Write your review report to `review.md` in your working directory.
5. Write a handoff report at `handoff.md` and send a message back to the parent orchestrator (conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3) with the path to your handoff.md.
