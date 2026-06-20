## 2026-06-19T06:14:23Z
You are the Subsystem Topologies Reviewer (Reviewer 2) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/reviewer_subsystems_m4_2_gen3`.
Your task is to examine the correctness, completeness, robustness, and interface conformance of the implemented subsystem topologies schema and validation shapes/rules.
Specifically:
1. Read the target files:
   - subsystems ontology: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`
   - SHACL shapes: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
   - GGen configuration: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
2. Check that the C++ class hierarchy mapping is mathematically sound (OWL 2 DL compliant) and properly models:
   - Materials & parameters
   - Shaders, frequencies, parameters
   - WebGL/RHI fallbacks
   - Collision profiles & channel responses (resolving domain mismatches without class punning)
   - Kinematics (bSimulatePhysics, bOverrideMass, URigidBody)
   - Networking (net roles, replication lifetimes, player controllers, void RPC returns, mandatory Server RPC validation)
3. Run the ontology validation script `/Users/sac/rocket-craft/validate_ontology.sh` to ensure it compiles/validates.
4. Write your review report (correctness, gaps, unverified items) to `review.md` in your working directory.
5. Write a handoff report at `handoff.md` and send a message back to the parent orchestrator (conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3) with the path to your handoff.md.
