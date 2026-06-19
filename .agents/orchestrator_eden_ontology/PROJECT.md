# Project: Eden Server Ontology Refactor

## Architecture
The `eden_server` ontology registry defines the formal architecture of the dimensional marketplace. It includes:
- `pack.ttl`: Core ontology importing/mapping public standards (FIBO, SOSA, QUDT, PROV-O).
- `bandai_tps.ttl`: Toyota Production System & Reliability Engineering.
- `egp_racing.ttl`: Racing & Vehicles (e.g. chassis, tires, heatClass, stressClass).
- `mars_market.ttl`: Asset marketplace, trading, risk classes.
- SHACL shapes: Validation shapes to enforce byte-class limits and structural constraints.
- `ggen.toml`: Configures the manifest, SPARQL CONSTRUCT rules, SHACL validation files, and strict mode.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Explore | Explore current ontologies, structure, and ggen.toml | None | DONE |
| 2 | Core Graph Refactor | Refactor core ontology graphs to Level 5 maturity with OWL 2 DL restrictions, public mappings, and byte-class typestates | M1 | DONE |
| 3 | SHACL Validation | Implement SHACL validation shapes for byte-class bounds and structural constraints | M2 | DONE |
| 4 | Harness Integration | Wire validation harness in ggen.toml, configure SPARQL CONSTRUCT, set strict_mode=true | M3 | DONE |
| 5 | Verification | Run validation parser tests (rapper), negative test (paradox check), and ggen compiler run | M4 | DONE |

## Interface Contracts
- RDF/XML or Turtle imports of FIBO, SOSA, QUDT, PROV-O.
- Strict OWL 2 DL compliance: all classes/properties fully typed, restrictions on cardinalities.
- Byte-class limits: unsigned 8-bit integer boundaries [0-255] expressed in SHACL.
- `ggen.toml` configuration: imports must resolve locally/remotely, validations must fail build if violated when strict_mode=true.

## Code Layout
- Ontology Registry: `/Users/sac/.ggen/packs/eden_server/ontology/`
  - `pack.ttl`
  - `bandai_tps.ttl`
  - `egp_racing.ttl`
  - `mars_market.ttl`
  - `validation_shapes.ttl`
- Manifest: `/Users/sac/.ggen/packs/eden_server/ggen.toml`
