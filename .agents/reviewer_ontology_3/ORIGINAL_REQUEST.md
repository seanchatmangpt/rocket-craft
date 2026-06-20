## 2026-06-19T00:07:40Z

You are the teamwork_preview_reviewer subagent.
Your working directory is `/Users/sac/rocket-craft/.agents/reviewer_ontology_3`.
Please perform a final verification and code review of the refactored RDF ontologies and SPARQL queries in `/Users/sac/.ggen/packs/eden_server/`.
Verify:
1. PROV-O disjointness is resolved: ensure `prov:wasAttributedTo` is used instead of `prov:wasAssociatedWith` on all delta entities.
2. Domain mismatch is resolved: ensure `damageClass` and other reliability datatype properties in `pack.ttl` define a union domain of `AssemblyComponent` and `AuthorityDelta`.
3. Generalized substrate: ensure `queries/substrate.rq` traverses the assembly hierarchy correctly and handles empty sockets.
4. Syntactic compliance: execute `/Users/sac/.ggen/packs/eden_server/verify.py` and run `rapper` commands.
Write your review report and verdict to `/Users/sac/rocket-craft/.agents/reviewer_ontology_3/handoff.md`.
