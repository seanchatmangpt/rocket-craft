## 2026-06-18T17:02:59-07:00
You are the teamwork_preview_worker subagent.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_ontology_1`.
Please initialize the target project workspace at `/Users/sac/.ggen/packs/eden_server` and implement the complete suite of RDF ontologies and SPARQL queries.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Objectives:
1. Initialize the directory structure under `/Users/sac/.ggen/packs/eden_server`:
   - `ontology/`
   - `queries/`
2. Create `ontology/pack.ttl` by merging the rich public ontology mappings from `/Users/sac/rocket-craft/.agents/explorer_ontology_1/proposed_pack.ttl` and the validated structure of `/Users/sac/rocket-craft/.agents/explorer_ontology_3/pack.ttl`.
   Specifically, ensure the following alignments/mappings:
   - `eden:AssemblyComponent` rdfs:subClassOf fibo:Asset, fibo:Product, sosa:FeatureOfInterest, prov:Entity.
   - `eden:Socket` rdfs:subClassOf eden:AssemblyComponent, sosa:Platform.
   - Reliability Datatype properties (`eden:damageClass`, `eden:stressClass`, `eden:heatClass`, `eden:fatigueClass`) have `rdfs:range xsd:unsignedByte` and QUDT mappings:
     - `eden:damageClass` -> `qudt:hasQuantityKind qudt:DimensionlessRatio`.
     - `eden:stressClass` -> `qudt:hasQuantityKind qudt:Stress`.
     - `eden:heatClass` -> `qudt:hasQuantityKind qudt:Temperature`.
     - `eden:fatigueClass` -> `qudt:hasQuantityKind qudt:DimensionlessRatio`.
3. Create `ontology/deltas.ttl` based on `/Users/sac/rocket-craft/.agents/explorer_ontology_3/deltas.ttl`, ensuring the 5 Delta families are defined as subclasses of `eden:Delta` (which subclasses `prov:Entity`).
4. Create the SPARQL queries under `queries/`:
   - `queries/substrate.rq` (copied from `/Users/sac/rocket-craft/.agents/explorer_ontology_3/draft_queries/substrate.rq`)
   - `queries/extract_authority_deltas.rq` (copied from `/Users/sac/rocket-craft/.agents/explorer_ontology_3/draft_queries/extract_authority_deltas.rq`)
   - `queries/extract_assembly_deltas.rq` (copied from `/Users/sac/rocket-craft/.agents/explorer_ontology_3/draft_queries/extract_assembly_deltas.rq`)
   - `queries/extract_receipt_deltas.rq` (copied from `/Users/sac/rocket-craft/.agents/explorer_ontology_3/draft_queries/extract_receipt_deltas.rq`)
5. Setup a verification suite:
   - Write a verification script `/Users/sac/.ggen/packs/eden_server/verify.py` (you can adapt `/Users/sac/rocket-craft/.agents/explorer_ontology_3/verify.py` to target the local workspace files).
   - Execute the verification script to verify syntax and query execution against the mock dataset.
   - Run Raptor `rapper` tool (e.g. `rapper -i turtle -c ontology/pack.ttl`) to confirm syntactic turtle compliance of all ttl files.
6. Record all execution logs, validation output, and results in your handoff report at `/Users/sac/rocket-craft/.agents/worker_ontology_1/handoff.md`.
