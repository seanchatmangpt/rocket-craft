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

## 2026-06-18T17:45:57-07:00

<USER_REQUEST>
# Teamwork Project Prompt — Ggen Pack Specification

Research the `~/ggen/` repository (specifically the configuration schema found in `ggen.toml`) and author the canonical formal specification for building a validated `ggen` ontology pack. This specification must document the required TOML metadata, the ontology import structures, the SPARQL inference rules (`[inference]`), and the generation pipeline rules (`[[generation.rules]]`) to standardize all future ontology manufacturing packs.

Working directory: /Users/sac/.ggen/specs/
Integrity mode: benchmark

## Requirements

### R1. Document `ggen.toml` Configuration Schema
Create an exhaustive Markdown document detailing the required structure of a `ggen` pack manifest. Break down the `[project]` block, the `[ontology]` graph sources, the SPARQL `[inference]` rules (using `CONSTRUCT`), and the `[[generation.rules]]` (using `SELECT` queries mapped to `.tera` templates).

### R2. Author a Quick-Start Boilerplate
Include a comprehensive boilerplate section within the specification that provides a copy-pasteable minimal `ggen.toml` and reference `.ttl` structure so future teams can instantly bootstrap a validated pack.

## Acceptance Criteria

### Documentation Integrity
- [ ] The team produces `GGEN_PACK_SPEC.md` in the target directory.
- [ ] The specification clearly differentiates between `[inference]` (modifying the graph via SPARQL CONSTRUCT) and `[[generation.rules]]` (projecting the graph to files via SPARQL SELECT + Tera templates).
- [ ] The specification includes the "BIG BANG 80/20" criteria found in the reference `ggen.toml`.
- [ ] The boilerplate example provides a syntactically valid `ggen.toml` snippet that matches the engine's expected schema.
</USER_REQUEST>

## 2026-06-19T01:55:50Z

<USER_REQUEST>
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
</USER_REQUEST>

## 2026-06-19T04:28:44Z

<USER_REQUEST>
Deploy a 20-agent multi-disciplinary swarm to aggressively audit and close all semantic gaps in the Rocket-Craft pipeline. The core objective is not ontology expansion, but strict manufacturability: proving that the admitted graph can physically manufacture a working, multi-resolution Eden/GMF world with valid walkthroughs, byte-class typestates, and unforgeable receipts. 

Working directory: /Users/sac/.ggen/
Integrity mode: benchmark

## Requirements

### R1. Complete the Manufacturable Ontology Surface
Fill out the remaining `.ttl` gaps across the `eden_server` and `ue4_ontology` packs. **Only admit ontology that is consumed by manufacturing.** Do not exhaustively map public interfaces for the sake of completeness. Treat the ontology as inventory, templates as machines, and generated artifacts as finished goods.

### R2. Author Exhaustive SPARQL Inference Subsets
Write the bounded, deterministic `SPARQL SELECT` and `SPARQL CONSTRUCT` queries required by `ggen.toml` to extract exact, compile-time typestates. Every query must use an `ORDER BY` clause to guarantee deterministic assembly.

### R3. Hard-Gate with SHACL
Ensure every single semantic constraint introduced by the swarm is accompanied by a native SHACL shape file that aggressively prevents illogical combinations before they ever reach the C++ compiler.

### R4. Manufacturability Audit
For every newly introduced ontology concept, the swarm must identify:
- the `ggen` template family that consumes it
- the generated artifact type
- the runtime surface it affects
- the walkthrough proof that exercises it
*Constraint: No ontology node may exist without a manufacturing consumer.*

### R5. Walkthrough Closure
The swarm must prove that the ontology contains sufficient information to generate locations, exits, routes, zones, interactables, manufacturing stations, repair stations, race facilities, and market facilities. Every generated space must be reachable through a deterministic walkthrough.

### R6. Renderability Audit
Every ontology class that may become a visual artifact must define its LOD class, material class, instancing class, semantic importance class, silhouette importance class, and interaction distance class. The graph must support the generation of deterministic Render BOMs.

### R7. Semantic Importance Modeling
Every visual ontology artifact must be classified as: CROWN, PRIMARY, SECONDARY, TERTIARY, or BACKGROUND. This classification must natively support generated LOD culling and strict rendering budgets.

### R8. Gameplay Cell Coverage
The swarm must identify all gameplay production cells and ensure ontology support exists for: Manufacturing, Repair, Race, Trade, Insurance, Prediction, Resource Collection, Infrastructure, Defense, Exploration, Discovery, and Research.

### R9. Missing Surface Discovery
Produce a residual gap report identifying:
- concepts required by gameplay but absent from the ontology
- concepts present in the ontology but unused by manufacturing
- concepts present in templates but unsupported by the ontology
- concepts present in runtime but unsupported by templates

### R10. Authority Surface Coverage
The swarm must identify and model every authoritative state dimension required by the world (Damage, Heat, Stress, Fatigue, Grip, Energy, Resource, Market Condition, Risk, Provenance, Conformance, Standing). For every authority dimension, define: ontology representation, SHACL validation, SPARQL extraction path, generated typestate, Render BOM impact, gameplay consequence, and receipt consequence. *Authority dimensions must support byte-class representation.*

### R11. Resolution Closure
Every generated world artifact must support multiple states of resolution (Global, Regional, Zone, Facility, Assembly, Subassembly, Part, Socket). The swarm must prove that ontology, templates, and manifests support deterministic projection between resolutions. No artifact may exist only at maximum resolution.

### R12. Manufacturing Flow Coverage
Every ontology concept must participate in at least one complete flow:
`Ontology → SHACL → SPARQL → Typestate → Template → Generated Artifact → Runtime Surface → Walkthrough Proof → Receipt`
Concepts without a complete flow must be reported as residual inventory.

## Acceptance Criteria

### Combinatorial & Compiler Integrity
- [ ] `rapper` confirms all newly authored `.ttl` files have zero syntax errors and valid import resolutions.
- [ ] Programmatic scan proves 100% of the new `.rq` or inline SPARQL queries contain an explicit `ORDER BY` clause.
- [ ] The `verify_all_rules.sh` test harness yields a 100% pass rate against negative SHACL permutations.

### The ALIVE Proof
- [ ] Using ONLY the generated ontology, SPARQL, SHACL, and `ggen` manifests, the system must be capable of generating:
  1. A walkable GMF factory
  2. A complete mech assembly line
  3. A race facility
  4. A market facility
  5. A deterministic MUD walkthrough
  6. Renderable artifacts with valid Render BOMs
  7. Semantic LOD classifications
  8. Authority typestates
  9. Receipt paths
  10. States-of-resolution projections

*No manual code additions. No mock runtime substitutions. No placeholder artifacts.*
</USER_REQUEST>
