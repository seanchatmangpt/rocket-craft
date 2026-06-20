## 2026-06-19T00:04:03Z
You are the teamwork_preview_reviewer subagent (Reviewer 1).
Your working directory is `/Users/sac/rocket-craft/.agents/reviewer_ontology_1`.
Please review the implemented RDF ontologies and SPARQL queries in the workspace `/Users/sac/.ggen/packs/eden_server`.
Specifically, check:
- `ontology/pack.ttl`: verify correct namespaces, prefix formatting, OWL imports (FIBO, SOSA, QUDT, PROV-O), class definitions (AssemblyComponent, MechRoot, SubAssembly, Part, Socket), object/datatype properties, byte-class ranges (xsd:unsignedByte), and mappings to standard ontologies.
- `ontology/deltas.ttl`: verify base Delta and the 5 Delta families.
- `queries/`: verify SPARQL 1.1 queries (`substrate.rq`, `extract_authority_deltas.rq`, `extract_assembly_deltas.rq`, `extract_receipt_deltas.rq`). Check for syntax, performance, and correctness (like the empty socket handling in `substrate.rq`).
- Execute `/Users/sac/.ggen/packs/eden_server/verify.py` and run `rapper` checks.
Evaluate completeness and correctness, and write your review report to `/Users/sac/rocket-craft/.agents/reviewer_ontology_1/handoff.md`.
