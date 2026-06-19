# Project: Eden Manufacturing Server Ontology
# Scope: Complete Ontology and SPARQL Query suite implementation for Eden Manufacturing Server

## Architecture
The Eden Manufacturing Server Ontology provides the semantic representation for Industry 4.0 reliability twin engineering, Combinatorial Assembly Trees, and the 5 Delta families in the Combinatorial Maximalist platform.

- **Workspace Path**: `/Users/sac/.ggen/packs/eden_server`
- **Ontology Layout**:
  - `ontology/pack.ttl`: Core ontology importing FIBO, SOSA, QUDT, and PROV-O. Defines the Combinatorial Assembly Tree (mech root, subassemblies, parts, sockets) and reliability twin properties (damage, stress, heat, fatigue class) mapped as byte-class authority types.
  - `ontology/deltas.ttl`: Formalizing the 5 Delta families: `AuthorityDelta`, `AssemblyDelta`, `ProjectionDelta`, `InterestDelta`, and `ReceiptDelta`.
- **Query Layout**:
  - `queries/substrate.rq`: SPARQL 1.1 query to extract the assembly root and tree.
  - `queries/extract_authority_deltas.rq`: SPARQL 1.1 query to extract `AuthorityDelta` records.
  - `queries/extract_assembly_deltas.rq`: SPARQL 1.1 query to extract `AssemblyDelta` records.
  - `queries/extract_receipt_deltas.rq`: SPARQL 1.1 query to extract `ReceiptDelta` records.
- **Verification Infrastructure**:
  - A validation script will be written to parse Turtle (`.ttl`) files and SPARQL (`.rq`) files using Python's `rdflib` or other tools to check syntactic validity, RDF namespaces, classes/properties, and import declarations.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Workspace Initialization | Create workspace directories, prepare verification environment, check Python tools. | None | PLANNED |
| 2 | RDF Ontology Authoring | Author `ontology/pack.ttl` and `ontology/deltas.ttl` with complete class hierarchies and public imports. | M1 | PLANNED |
| 3 | SPARQL Query Suite Authoring | Author `queries/substrate.rq`, `queries/extract_authority_deltas.rq`, `queries/extract_assembly_deltas.rq`, and `queries/extract_receipt_deltas.rq`. | M1 | PLANNED |
| 4 | Syntactic & Logic Verification | Run syntax parsing, validation tests, namespace checks, class structure validation. | M2, M3 | PLANNED |
| 5 | Integrity Audit & Handoff | Run the Forensic Auditor to ensure zero violations, clean verdict, and generate final handoff. | M4 | PLANNED |

## Interface & Ontology Contracts
### Prefix / Namespaces
- `rdf`: `http://www.w3.org/1999/02/22-rdf-syntax-ns#`
- `rdfs`: `http://www.w3.org/2000/01/rdf-schema#`
- `owl`: `http://www.w3.org/2002/07/owl#`
- `xsd`: `http://www.w3.org/2001/XMLSchema#`
- `eden`: `https://ggen.io/ontology/eden-server/`
- `fibo`: `https://spec.edmcouncil.org/fibo/ontology/`
- `sosa`: `http://www.w3.org/ns/sosa/`
- `qudt`: `http://qudt.org/schema/qudt/`
- `prov`: `http://www.w3.org/ns/prov#`

### Imports
`ontology/pack.ttl` must declare:
```turtle
owl:imports <https://spec.edmcouncil.org/fibo/ontology/> ,
            <http://www.w3.org/ns/sosa/> ,
            <http://qudt.org/schema/qudt/> ,
            <http://www.w3.org/ns/prov#> .
```

### Class Hierarchy Details
- **Assembly Tree**:
  - `eden:AssemblyComponent` as a base class.
  - `eden:MechRoot`, `eden:SubAssembly`, `eden:Part`, and `eden:Socket` as subclasses.
  - Sockets represent connection points. Parts plug into sockets.
- **Reliability Twin Properties**:
  - `eden:damageClass`, `eden:stressClass`, `eden:heatClass`, `eden:fatigueClass` (or classes/properties as needed).
  - Properties mapping to byte-class authority values (e.g. `xsd:unsignedByte`).
- **Deltas**:
  - `eden:Delta` as base class.
  - Subclasses: `eden:AuthorityDelta`, `eden:AssemblyDelta`, `eden:ProjectionDelta`, `eden:InterestDelta`, `eden:ReceiptDelta`.
