## 2026-06-19T05:32:06Z
You are the Preemptive Audit Worker (Worker).
Your working directory is `/Users/sac/rocket-craft/.agents/worker_preemptive_audit_gen2`.
Your task is to execute a preemptive deep audit of the Ontology, OWL 2 DL, and SPARQL extraction layers for the UE4 Universal RDF Mapping and Eden Server ontologies.

Specifically, you must:
1. Examine the ontologies in `/Users/sac/.ggen/packs/ue4_ontology/` and `/Users/sac/.ggen/packs/eden_server/ontology/`.
2. Verify that the SPARQL extractions (in the respective `ggen.toml` configurations and query files) correctly map to WASM memory and API constraints. Pay close attention to:
   - SPARQL subset bounding (Anti-Cartesian exhaustion) - check if queries select only the target-oriented subsets and use strict `ORDER BY` to guarantee determinism.
   - Byte-class authority (the server owns standing, not visual detail). Check if authoritative state is compacted into discrete byte-class vectors rather than unnecessary high-resolution float representations.
3. Ensure Semantic LOD rules are strictly enforced. Unnecessary float data must not pollute the byte-authority. Verify that only required spatial or visual details are exposed to the client, while keeping the server state strictly compact.
4. Audit the ontologies for OWL 2 DL compliance defects:
   - Verify there are no violations of OWL 2 DL constraints, such as using properties as both object and datatype properties, missing class declarations for subjects/objects, or invalid transitive/symmetric property restrictions.
   - Run any available RDF/OWL reasoning or validation tool or check them statically.
5. If you find any defects:
   - Identify them clearly.
   - Propose and implement fixes directly in the respective `.ttl` files under `/Users/sac/.ggen/packs/ue4_ontology/` and `/Users/sac/.ggen/packs/eden_server/ontology/` or their respective configuration files.
   - Run `/Users/sac/rocket-craft/validate_ontology.sh` and `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` to ensure they compile and validate.
6. Write a detailed findings and repairs report to `audit_report.md` in your working directory. Ensure that you summarize the results using the strict TAI Status Reporting Format.
7. Write a handoff report at `handoff.md` and send a message back to the parent orchestrator (conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3) with the path to your handoff.md.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.
