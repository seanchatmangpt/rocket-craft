# Original User Request

## 2026-06-19T01:55:50Z

Refactor the entire `eden_server` ontology registry (`pack.ttl`, `bandai_tps.ttl`, `egp_racing.ttl`, `mars_market.ttl`) from semantic first principles into true Level 5 Combinatorial Maximalist graphs, fully defining strict OWL 2 DL restrictions, metadata alignment, and native SHACL validation shapes.

Working directory: /Users/sac/.ggen/packs/eden_server/ontology/
Integrity mode: benchmark

## Requirements

### R1. Refactor the Core Ontology Graphs
Rewrite the entire ontology suite to hit Level 5 on the 7x5 maturity matrix. Implement deep `owl:equivalentProperty` mapping to public standards (FIBO, QUDT, PROV-O), enforce strict `owl:Restriction` cardinalities for all components, and bind all states to byte-class typestates.

### R2. Implement SHACL Validation Shapes
Write explicit SHACL `.ttl` shapes that mathematically enforce the bounds of the byte-class typestates (e.g., preventing `egp:heatClass` from exceeding unsigned byte limits) and verifying structural constraints (e.g., a chassis must have exactly 4 tires).

### R3. Wire the `ggen.toml` Validation Harness
Integrate the refactored graphs and SHACL validation paths into the master `ggen.toml` manifest. Configure the exact `SPARQL CONSTRUCT` inference rules to extract the typestates, ensuring compatibility with the recently patched `strict_mode=true` compiler harness.

## Acceptance Criteria

### Ontological & SHACL Integrity
- [ ] `rapper` or an equivalent RDF parser confirms zero syntax errors and valid import resolution across the entire registry.
- [ ] A negative test proves that the SHACL shapes correctly identify and reject a deliberately injected paradox (e.g., an asset with an out-of-bounds `riskClass` or a missing cryptographic receipt).
- [ ] The official `ggen` compiler successfully parses the manifest, triggers the SHACL validations, and processes the `SPARQL CONSTRUCT` extraction rules without an Agent Jidoka halt.
