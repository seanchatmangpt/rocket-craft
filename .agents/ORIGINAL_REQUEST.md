# Original User Request

## 2026-06-19T00:00:02Z

<USER_REQUEST>
# Teamwork Project Prompt — Eden Manufacturing Server Ontology

Design and author the complete suite of RDF ontologies (.ttl) and SPARQL queries (.rq) for the Eden Manufacturing Server. This will formalize the architectural spine for the dimensional marketplace, mapping Industry 4.0 reliability engineering, sensor/fault loops, assembly-tree authority, and byte-class state deltas to the Combinatorial Maximalist platform.

Working directory: /Users/sac/.ggen/packs/eden_server
Integrity mode: benchmark

## Requirements

### R1. Public Ontology Integration
Create the core `pack.ttl` ontology that directly imports and maps Eden manufacturing concepts to public industry standards (e.g., FIBO for finance/markets, SOSA for telemetry/sensors, QUDT for quantities/physics, PROV-O for receipts).

### R2. Reliability & Assembly Topology
Define the ontological structure for a Combinatorial Assembly Tree (mech root, subassemblies, parts, sockets) and its reliability twin properties (damage class, stress class, heat class, fatigue class), mapped as byte-class authority types.

### R3. Delta Network Model
Formalize the 5 Delta families (`AuthorityDelta`, `AssemblyDelta`, `ProjectionDelta`, `InterestDelta`, `ReceiptDelta`) in the ontology to support the "replicate admitted deltas over object graphs" architecture.

### R4. SPARQL Query Suite
Author both the foundational `substrate.rq` (to extract the assembly root) and specific delta-mapping queries (e.g., `extract_authority_deltas.rq`, `extract_assembly_deltas.rq`) that the Java enterprise server will consume.

## Acceptance Criteria

### Syntactic and Structural Validity
- [ ] The team produces `ontology/pack.ttl` and `ontology/deltas.ttl` containing the full class hierarchies.
- [ ] The ontology explicitly declares imports (`owl:imports`) for FIBO, SOSA, QUDT, and PROV-O.
- [ ] The team produces a `queries/` directory containing at least `substrate.rq`, `extract_authority_deltas.rq`, and `extract_receipt_deltas.rq`.
- [ ] All `.ttl` files parse successfully as valid Turtle syntax (an independent agent or `rapper`/`riot` validation must confirm zero syntax errors).
- [ ] The queries are valid SPARQL 1.1 syntax.
</USER_REQUEST>

## 2026-06-19T00:00:38Z

Here are the direct official URLs for the public industry ontologies to download the `.ttl` files directly via `curl` for your mapping work:

**1. PROV-O (Provenance)**
W3C direct TTL: `http://www.w3.org/ns/prov.ttl`
(You can `curl -sH "Accept: text/turtle" -L http://www.w3.org/ns/prov.ttl`)

**2. SOSA / SSN (Sensors & Observations)**
Raw GitHub TTL: `https://raw.githubusercontent.com/w3c/sdw/gh-pages/ssn/integrated/sosa.ttl`

**3. QUDT (Quantities, Units, Dimensions)**
GitHub Releases (All-in-one TTLs): `https://github.com/qudt/qudt-public-repo/releases`
(Grab the latest `QUDT-all-in-one-OWL.ttl` or `QUDT-all-in-one-SHACL.ttl`)

**4. FIBO (Financial Industry Business Ontology)**
Production Zip: `https://spec.edmcouncil.org/fibo/ontology/prod.ttl.zip`
(Or use the GitHub repo: `https://github.com/edmcouncil/fibo`)

## 2026-06-19T00:32:24Z

<USER_REQUEST>
# Teamwork Project Prompt — UE4 Universal RDF Mapping

Design and author an exhaustive RDF ontology (in Turtle format) that represents the complete architecture and class hierarchy of Unreal Engine 4 (UE4). This includes modeling the `UObject` base, `AActor` lifecycle, `UActorComponent` system, Blueprint graphs, Materials, Levels, and reflection metadata, effectively demonstrating how Epic Games would represent the entire engine mathematically as a semantic graph.

Working directory: /Users/sac/.ggen/packs/ue4_ontology
Integrity mode: benchmark

## Requirements

### R1. Universal Class Inheritance (The C++ Core)
Map the exhaustive structural backbone of the engine: `UObject`, `AActor`, `APawn`, `ACharacter`, `UActorComponent`, `UWorld`, `ULevel`, and the deep inheritance networks that connect them.

### R2. Subsystem & Domain Topologies
Model the granular domain boundaries, including the Rendering Pipeline (Materials, Shaders, WebGL/RHI fallbacks), Physics layers (Collision Volumes, Kinematics), and Networking (Replication, RPCs).

### R3. Reflection & Blueprint Graph Modeling
Formally define the UE4 Reflection System (`UClass`, `UProperty`, `UFunction`) as semantic triples. Model how Blueprint nodes and visual execution paths exist purely as combinatorially valid RDF graphs.

### R4. Combinatorial Maximalist Typestates
Map how all of the above components compile, link, and exist as deterministic typestates. Include the definitions for how `rocket build` or standard cooking pipelines project these nodes into final WASM/HTML5 outputs.

## Acceptance Criteria

### Engine Mapping Integrity
- [ ] The team produces an exhaustive suite of `.ttl` files (e.g., `core.ttl`, `reflection.ttl`, `blueprints.ttl`, `physics.ttl`, `rendering.ttl`).
- [ ] The ontology successfully unifies the static C++ inheritance structure with the dynamic Reflection and Blueprint graph systems.
- [ ] An independent `ggen sync --validate-only` (or equivalent native `ggen` syntax check) confirms that all generated ontologies are 100% syntactically valid and structurally sound.
- [ ] The mapping is comprehensive enough to theoretically generate UE4 C++ headers and Blueprint structures entirely from the RDF graph.
</USER_REQUEST>
