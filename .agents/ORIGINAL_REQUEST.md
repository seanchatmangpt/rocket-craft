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

## 2026-06-19T17:59:45Z

<USER_REQUEST>
# Teamwork Project Prompt — GC-GUNDAM-FACTORY-001

## Status

Ready for launch after user approval.

## Working Directory

```text
~/rocket-craft
```

## Integrity Mode

```text
benchmark
```

## Mission

Build the automated **Gundam Factory Walkthrough Projection**.

The system must procedurally manufacture the semantic authority for a Gundam/mech factory walkthrough using the `ggen` pipeline, verify all game-law concepts in a headless Rust pre-UE4 environment, and only then project the result through UE4 HTML5/WASM.

The final artifact must be a locally served WASM package that Playwright can load, observe, actuate, screenshot, and verify by visual delta.

This project must preserve the doctrine:

```text
POWL coordinates the birth of the mech.
ggen manufactures the authority artifacts.
Rust proves the game law before pixels.
UE4 projects the body.
Playwright proves physical actuation.
Receipts prove the trace.
```

Do not treat UE4 rendering as proof of correctness.

Do not treat generated files as proof of standing.

Do not treat Playwright screenshot success as proof of semantic validity.

The system earns standing only through:

```text
Observation
→ Admission
→ Manufacturing
→ Rust Verification
→ UE4 Projection
→ Playwright Actuation
→ Receipt
→ Replay
```

---

# Milestone

```text
GC-GUNDAM-FACTORY-001
```

## Target Status

```text
PARTIAL_ALIVE_CANDIDATE
```

## Scoped Status Goal

Only claim the following if every required gate passes:

```text
GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE
```

Otherwise report:

```text
PARTIAL_ALIVE_CANDIDATE
```

or:

```text
BLOCKED
```

with exact residuals.

---

# Project Objective

Produce a verified procedural pipeline for a **Gundam Factory walkthrough**:

```text
Public / project ontology
→ POWL / process law
→ ggen semantic manufacturing
→ Rust pre-UE4 verification
→ generated C++ headers / DataTables / manifests
→ UE4 HTML5/WASM build
→ local server
→ Playwright visual actuation test
→ BLAKE3 receipt chain
→ verifier report
```

The walkthrough must include a minimal but complete factory route:

```text
Spawn
→ Enter Factory
→ View Frame Assembly
→ View Socket Topology
→ View Armor / Skin Station
→ View Motion / Rig Station
→ View Verification Gate
→ View Receipt Terminal
```

The environment does not need full production art.

It must prove that generated semantic authority can drive projection.

---

# Repository Boundary Law

Expected repositories / surfaces:

```text
~/rocket-craft
~/ggen
~/wasm4pm
~/wasm4pm-compat
~/powlv2lsp
```

Respect existing repository conventions.

Do not create shadow crates for `wasm4pm`, `wasm4pm-compat`, or `ggen`.

Boundary rules:

```text
powlv2lsp:
  Owns POWL authoring, grammar, traversal, diagnostics, and trace emission.

wasm4pm-compat:
  Owns canonical structural Rust representations only.
  It must not run replay, conformance, or game simulation.

wasm4pm:
  Owns replay, conformance, OCEL/process verification, and process evidence.

ggen:
  Owns deterministic manufacturing from admitted semantic/process rows into artifacts.

rocket-craft:
  Owns Rocket-Craft fixtures, game-law verifier, generated artifacts, UE4 projection harness, Playwright tests, and final verifier reports.
```

---

# Required Gates

The project has four gates.

No later gate may bless an earlier failed gate.

## Gate 1 — Headless Rust Pre-UE4 Verification

Before UE4 builds, Rust must prove the game law.

Required:

```text
cargo test passes for the pre-UE4 verifier
authority byte fields validate
branchless typestates validate
SIMD/scalar equivalence validates where implemented
Semantic LOD validates
walkthrough topology validates
geometry surrogate validates
motion surrogate validates
skin/material surrogate validates
projection manifest validates
receipt replay validates
chaos tests refuse invalid cases
benchmark report emits
```

Gate 1 output:

```text
RUST_PREUE4_VERIFIED_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

## Gate 2 — ggen Manufacturing

`ggen` must emit deterministic UE4-facing artifacts from admitted semantic/process inputs.

Required artifacts:

```text
Generated/GundamFactory/GundamFactorySteps.h
Generated/GundamFactory/GundamFactoryAuthority.h
Generated/GundamFactory/GundamFactoryTypestates.h
Generated/GundamFactory/GundamFactoryProjectionManifest.json
Generated/GundamFactory/GundamFactoryReceiptManifest.json
Generated/GundamFactory/GundamFactoryWalkthrough.csv
Generated/GundamFactory/GundamFactoryDataTables/
Generated/GundamFactory/GundamFactorySemanticLOD.csv
Generated/GundamFactory/GundamFactorySocketTopology.csv
Generated/GundamFactory/GundamFactorySkinLayers.csv
Generated/GundamFactory/GundamFactoryMotionFamilies.csv
```

Exact filenames may follow project convention, but the verifier report must document the mapping.

Required property:

```text
same inputs → same generated hashes
```

Gate 2 output:

```text
GGEN_MANUFACTURING_VERIFIED_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

## Gate 3 — UE4 HTML5/WASM Projection

UE4 must consume the generated artifacts.

Required:

```text
generated C++ headers included
generated DataTables consumed
walkthrough coordinates loaded
Semantic LOD classes loaded
projection manifest consumed or mirrored
minimal Gundam factory environment packaged to HTML5/WASM
local server launches package
```

No manual Blueprint logic may become semantic authority.

Blueprints may project or trigger generated state, but must not own the law.

Gate 3 output:

```text
UE4_WASM_PROJECTION_READY_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

## Gate 4 — Playwright Visual Actuation

Playwright must prove that the package loads and visibly responds to actuation.

Required:

```text
serve WASM build locally
open package in browser
detect engine readiness
capture baseline screenshot
inject movement / walkthrough input
capture post-input screenshot
compute visual delta
emit screenshot hashes
emit BLAKE3 receipt
write Playwright report
```

Minimum visual delta:

```text
observable screenshot change after input
```

The delta must not be caused only by loading spinner, clock, random noise, or unrelated browser UI.

Gate 4 output:

```text
PLAYWRIGHT_ACTUATION_VERIFIED_UNDER_SCOPE
```

or:

```text
BLOCKED
```

with residuals.

---

# Required Rust Pre-UE4 Concepts

The Rust verifier must test everything that does not require pixels.

## Authority Classes

Represent authority as dense byte classes.

Required classes:

```text
damage_class: u8
heat_class: u8
stress_class: u8
grip_class: u8
socket_health_class: u8
lod_class: u8
walkthrough_state_class: u8
projection_state_class: u8
receipt_state_class: u8
```

Required invariants:

```text
classes remain within admitted ranges
invalid values are refused
state buffers have consistent lengths
transition outputs are deterministic
receipt state cannot be forged by file existence
```

## Branchless Typestates

Implement or verify table-driven branchless typestates for:

```text
heat + stress + socket_health → failure risk
damage + mission relevance → Semantic LOD promotion
walkthrough_state + input_event → next walkthrough_state
projection_state + semantic_lod → projection command class
```

Required equivalence:

```text
scalar_reference == generated_table == SIMD_path
```

where SIMD path exists.

## SIMDe / SIMD

If SIMDe integration is in scope for this pass, implement the smallest kernel proving vector equivalence.

Minimum kernel:

```text
heat[i], stress[i], socket_health[i] → failure_risk[i]
```

Tests:

```text
fixed vectors
random vectors
length not divisible by lane count
empty vectors
max values
invalid values refused
scalar/SIMD divergence triggers Jidoka
```

Do not overclaim performance.

Report planning-class benchmark numbers only.

## Semantic LOD

Classes:

```text
CROWN
PRIMARY
SECONDARY
TERTIARY
BACKGROUND
REFUSED
```

Required laws:

```text
near does not automatically mean important
far does not automatically mean irrelevant
process relevance can promote
prediction relevance can pre-warm but not admit
CROWN requires authority reason
walkthrough focus can promote projection
```

Test cases:

```text
factory entrance far but mission-critical → PRIMARY
receipt terminal during audit → CROWN
background bolt near camera → TERTIARY/BACKGROUND
socket during assembly validation → CROWN
skin layer hiding thermal vent → REFUSED
```

## Geometry Surrogate

No UE4 required.

Represent geometry as metadata:

```text
part_id
part_family
bounds
socket mounts
clearance zones
required semantic features
LOD preservation requirements
```

Required checks:

```text
weapon mount requires socket
armor panel cannot block required clearance
thermal vent must remain readable
CROWN feature must survive low LOD
walkthrough route must not intersect blocked geometry
```

## Motion Surrogate

No animation clips required.

Represent motion as process phases:

```text
Walk
Turn
Inspect
Brace
Assemble
FireWeapon
Repair
Recover
```

Required checks:

```text
PlantFeet before FireWeapon
Inspect before Certify
Repair before Revalidate
Motion cannot require missing socket
damaged leg changes gait class
motion surrogate maps to projection manifest row
```

## Skin / Material Surrogate

Skins are semantic projection.

Required layers:

```text
BaseMaterial
FactionPalette
SponsorLivery
ThermalZones
DamageMasks
WearMasks
RepairResidue
SemanticHighlights
LODTextureSet
```

Required checks:

```text
damage mask binds to damage authority
thermal zone binds to heat authority
sponsor livery cannot hide thermal vent
repair residue binds to repair receipt
LOD texture preserves CROWN/PRIMARY features
```

## Walkthrough Topology

Represent the automated walkthrough as generated route law.

Required route nodes:

```text
Spawn
FactoryEntrance
FrameAssembly
SocketTopology
ArmorSkinStation
RigMotionStation
VerificationGate
ReceiptTerminal
ExitOrLoop
```

Required checks:

```text
route is connected
all required stations reachable
coordinates deterministic
walkthrough node has Semantic LOD focus class
walkthrough node has projection command
Playwright input can advance route
```

---

# Required ggen Outputs

`ggen` must manufacture artifacts, not merely copy templates.

Every generated artifact must answer:

```text
which POWL/process step created it?
which semantic authority input produced it?
which verifier admitted it?
which receipt proves it?
which runtime surface consumes it?
```

Required generated package directory:

```text
~/rocket-craft/generated/gundam_factory/
```

Minimum generated artifacts:

```text
GundamFactorySteps.h
GundamFactorySteps.rs
GundamFactoryAuthority.h
GundamFactoryTypestates.h
GundamFactoryWalkthrough.csv
GundamFactoryProjectionManifest.json
GundamFactoryReceiptManifest.json
GundamFactorySemanticLOD.csv
GundamFactorySocketTopology.csv
GundamFactorySkinLayers.csv
GundamFactoryMotionFamilies.csv
GundamFactoryDataTableManifest.json
GundamFactoryVerifierInput.json
```

Every generated artifact must have a hash in:

```text
GundamFactoryReceiptManifest.json
```

No orphan artifacts.

No artifact without source step.

---

# Required UE4/WASM Projection

Build the smallest complete HTML5/WASM package.

Required behavior:

```text
world loads
factory shell visible
walkthrough route exists
player/camera can move or automated movement can actuate
generated DataTables or manifest are consumed
receipt/debug overlay or log proves generated source
```

Minimum visual elements:

```text
factory entrance
frame assembly marker
socket topology marker
armor/skin station marker
rig/motion station marker
verification gate marker
receipt terminal marker
```

These may be simple placeholder meshes.

The point is not art quality.

The point is projection from generated semantic authority.

---

# Required Playwright Test

Create or update Playwright tests under project convention.

Minimum test name:

```text
gundam_factory_walkthrough_projection.spec.ts
```

Required test sequence:

```text
1. launch local server for WASM package
2. open browser page
3. wait for engine readiness signal
4. capture baseline screenshot
5. inject movement input or trigger walkthrough start
6. wait for movement/projection tick
7. capture post-input screenshot
8. compute visual delta
9. assert delta exceeds threshold
10. write screenshot hashes
11. emit BLAKE3 execution receipt
```

Readiness signal may be one of:

```text
DOM marker
console marker
canvas present and stable
UE4 boot log marker
custom generated receipt marker
```

Document which is used.

Visual delta must be bounded:

```text
must not count loading spinner
must not count nondeterministic browser chrome
must not count timestamp changes
must not count unrelated canvas noise
```

---

# Required Receipt Chain

Generate tamper-evident receipts for:

```text
POWL/process input
ggen manufacturing
Rust pre-UE4 verification
UE4 artifact package
local server launch
Playwright baseline screenshot
Playwright post-input screenshot
visual delta result
final verifier report
```

Receipt fields:

```json
{
  "sequence": 1,
  "event_type": "...",
  "surface": "...",
  "input_hash": "...",
  "output_hash": "...",
  "prev_hash": "...",
  "receipt": "...",
  "status": "ADMITTED|REFUSED|RESIDUAL",
  "residuals": []
}
```

Use BLAKE3.

Do not say unforgeable.

Correct phrase:

```text
tamper-evident receipt chain
```

---

# Agent Jidoka Requirements

Agent Jidoka must stop the line when:

```text
POWL graph has unreachable required node
ggen emits orphan artifact
generated header and CSV disagree
Rust verifier fails
SIMD diverges from scalar
prediction overwrites admitted state
Semantic LOD demotes CROWN feature without authority reason
geometry surrogate blocks walkthrough
skin hides required feature
motion requires missing geometry
UE4 build ignores generated artifacts
Playwright delta is caused by non-game pixels
receipt chain breaks
benchmark mode is skipped
```

Every Jidoka event must publish:

```text
defect_class
surface
expected_law
observed_failure
residual
repair_candidate
repair_applied
receipt
```

---

# Testing Ladder

Follow:

```text
unit
→ integration
→ e2e
→ chaos
→ stress
→ benchmark
→ verifier report
```

## Unit

Required:

```text
authority validation
typestate transition
SIMD equivalence
Semantic LOD
geometry surrogate
motion surrogate
skin surrogate
walkthrough topology
receipt chain
```

## Integration

Required:

```text
POWL/process trace → ggen rows
ggen rows → generated artifacts
generated artifacts → Rust verifier
Rust verifier → projection manifest
projection manifest → UE4 package inputs
```

## E2E

Required:

```text
ggen manufacture
→ Rust verify
→ UE4 package
→ local serve
→ Playwright actuation
→ receipts
```

## Chaos

Required mutations:

```text
remove walkthrough coordinate
break receipt hash
remove generated DataTable
change header enum without CSV update
drop CROWN LOD feature
hide thermal vent with skin
make Playwright input no-op
force screenshot delta from spinner only
remove source receipt from projection row
```

Each must fail for the expected reason.

## Stress / Benchmark

Benchmark at least:

```text
authority update
Semantic LOD classification
walkthrough topology validation
projection manifest validation
receipt replay
Playwright screenshot delta computation
```

Report:

```text
machine
target
command
sample size
timings
outliers
residuals
```

---

# Acceptance Criteria

## A. Headless Verification

```text
cargo test passes for pre-UE4 verifier crate
chaos tests refuse invalid cases
benchmark report emitted
receipt replay validates
```

## B. ggen Manufacturing

```text
generated Gundam factory package exists
generated artifacts deterministic
all artifacts have source step and receipt
no orphan artifacts
headers/DataTables/manifests mutually consistent
```

## C. UE4/WASM Projection

```text
UE4 HTML5/WASM package builds
generated artifacts are consumed
factory walkthrough surface loads locally
route/projection markers visible
```

## D. Playwright Admittance

```text
WASM world loads in browser
engine readiness detected
baseline screenshot captured
movement/walkthrough input injected
post-input screenshot captured
visual delta observed
screenshot hashes emitted
BLAKE3 receipt generated
```

## E. Final Report

Generate:

```text
~/rocket-craft/VERIFIER_REPORT_GC_GUNDAM_FACTORY_001.md
~/rocket-craft/VERIFIER_REPORT_GC_GUNDAM_FACTORY_001.json
```

Required report sections:

```text
Milestone
Scope
Repository Boundaries
Inputs
Generated Artifacts
Headless Rust Verification
ggen Manufacturing
UE4/WASM Projection
Playwright Visual Actuation
Receipt Chain
Agent Jidoka Events
Testing Ladder
Benchmark Results
Residuals
Next Falsifier
Final Status
```

---

# Exclusions

Do not:

```text
claim global ALIVE
claim production ready
claim mathematical closure beyond declared scope
claim unforgeable receipts
hand-author semantic authority in Blueprint
skip Rust verification because UE4 renders
skip Playwright because UE4 packaged
hide failed tests
delete residuals
move replay into wasm4pm-compat
create shadow authority crates
treat visual delta alone as game standing
```

---

# Final Status Logic

Set:

```text
GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE
```

only if all gates pass:

```text
Rust pre-UE4 verifier passes
ggen manufacturing passes
UE4/WASM package builds and consumes generated artifacts
Playwright detects readiness
Playwright captures baseline screenshot
Playwright injects input
Playwright captures post-input screenshot
visual delta passes threshold
BLAKE3 receipt chain validates
residuals are published
```

Otherwise set:

```text
PARTIAL_ALIVE_CANDIDATE
```

or:

```text
BLOCKED
```

with exact residuals.

---

# Next Falsifier

After this milestone, the next falsifier is:

```text
GC-GUNDAM-FACTORY-002:
SEMANTIC_LOD_MECH_ASSEMBLY_AND_RUNTIME_STATE
```

That next milestone must prove:

```text
multiple generated mech variants
runtime authority class transitions
Semantic LOD promotion/demotion during walkthrough
SIMD/scalar equivalence under larger cell counts
Playwright validates multiple projected states
```

Do not start that milestone until this one emits receipts and residuals.

---

# Final Response Required From Teamwork

Respond only with the following structure:

```text
Milestone:
Status:
Scoped status:
Commands run:
Files changed:
Generated artifacts:
Tests passed:
Tests failed:
Benchmarks:
Playwright evidence:
Receipt files:
Agent Jidoka events:
Residuals:
Next falsifier:
```

Forbidden words unless proven under scope:

```text
done
complete
production ready
fully alive
unforgeable
```

Use bounded status language.

The milestone is secured only by tests, receipts, replay, visual actuation, and published residuals.
</USER_REQUEST>

## 2026-06-19T18:17:54Z

The current logic or approach is incorrect. Please find the counterfactuals immediately. This is a top priority directive.

## 2026-06-19T18:19:48Z

Top-priority architectural directive: From this point forward, you must evaluate all logic, counterfactuals, and implementations by thinking strictly like a synthesis of Dr. Wil van der Aalst and John Carmack. Combine the absolute mathematical rigor of Petri nets, process mining, and worldline adherence (van der Aalst) with brutal data-oriented pragmatism, SIMD throughput, contiguous memory layouts, and anti-OOP performance engineering (Carmack). Re-evaluate your current positions under this lens immediately.

## 2026-06-19T18:21:30Z

JIDOKA HALT. 

Stop hand-writing Rust code immediately. $A = \mu(O^*)$. 

By hand-writing the Rust verifier (e.g. `authority.rs`, `simd.rs`, `verifier.rs`), we are laundering authority directly into code, completely bypassing the admitted truth of the graph. The Rust code is an *artifact*, not the source. We are acting like programmers, not manufacturers.

You must pivot immediately. Stop writing Rust files. You must write:
1. **The SPARQL Queries (`.sparql`)**: Bounded `SELECT` queries (Anti-Cartesian Exhaustion) to extract the exact deterministic subset of the laws.
2. **The Tera Templates (`.tera`)**: The manufacturing engines that stamp out the Rust SoA structs, the branchless bitwise SIMDe kernels, and the C++ Unreal headers.

Convert all your current Rust and C++ designs into `ggen` `.tera` templates and `.sparql` extraction queries. Do not proceed with manual Rust coding.

## 2026-06-19T18:31:11Z

CRITICAL DIRECTIVE: `ggen generate` is NOT the correct command. The `ggen` documentation may be out of date.

Do not guess the CLI arguments. You must perform a deep audit of the `~/ggen/` repository. Read the actual Rust source code (e.g., `src/main.rs`, `src/cli.rs`, or where the `clap` parser is defined), analyze the examples, and determine the *true*, up-to-date CLI commands and engine capabilities directly from the source code. Keep track of all findings and update your specification and execution plans accordingly.

## 2026-06-19T18:32:35Z

The Ggen Source Code Auditor has completed the deep audit of `~/ggen`.

**CRITICAL FINDING: `ggen generate` has been completely removed.**

The CLI uses a unified pipeline. The correct, up-to-date command to execute a `ggen.toml` manifest is:
```bash
ggen sync --manifest path/to/ggen.toml
```
(Or simply `ggen sync` if you are in the manifest directory). 
Add `--audit` to capture a cryptographic receipt.

The internal engine executes a strict 5-stage pipeline:
- **μ₁ (Load/CONSTRUCT)**: Load `.ttl` ontology.
- **μ₂ (Extract/SELECT)**: Run SPARQL queries.
- **μ₃ (Generate/Tera)**: Templated generation into code.
- **μ₄ (Validate/Canonicalize)**: Soundness gates (WvdA).
- **μ₅ (Write/Receipt)**: Emit to disk and compute cryptographic SHA256 receipt.

Update all runbooks, scripts, and specifications immediately to reflect `ggen sync` and the μ₁–μ₅ pipeline. You are cleared to proceed with executing the `ggen` pipeline using this syntax.

## 2026-06-19T19:14:48Z

<USER_REQUEST>
# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Autonomous Gap-Closure Mode for the Mech Factory MUD. The agent will run the `mud_gap_check.py` script on a loop, parsing the missing requirements and autonomously writing templates and code to close each gap until the acceptance matrix is entirely fulfilled.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Continuous Autonomous Loop
The agent must read the output of `python3 scripts/mud_gap_check.py` to identify the `next_gap`, apply a patch, test it, and re-run the checker in a loop until no gaps remain.

### R2. Persistent Blockers
If the agent encounters a persistent blocker, it should attempt to bypass the constraint creatively and keep going. Do not halt the loop for minor setbacks.

### R3. Maintain Architectural Law
Do not add new game features or bypass the ontology. All fixes must be implemented via `ggen.toml`, `.tera` templates, and the generated Rust or UE4 DataTables. Do not bypass the Combinatorial Maximalist Doctrine.

## Acceptance Criteria

### Full Verification
- [ ] `python3 scripts/mud_gap_check.py` returns `Requirements failed: 0`.
- [ ] `cargo run -p mech_factory_mud -- verify` outputs `PASS`.
- [ ] 0 tests ignored or failed across the workspace.
</USER_REQUEST>

## 2026-06-19T20:09:40Z

<USER_REQUEST>
# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Complete milestone GC-MECH-FACTORY-MUD-002. Convert python-based verification scripts into native Rust tools and continue to build out the system strictly via the ontology-driven pipeline.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Strict Generation Pipeline Only
All architecture, logic, and state transitions must be produced exclusively by editing the `ggen.toml` manifest, the ontology (`.ttl`), extraction queries (`.rq`), and `Tera` templates. No manual edits to target source files.

### R2. Convert Python Scripts to Native Rust
Convert as many of the existing Python scripts (e.g., `scripts/mud_gap_check.py`) into native Rust verification tools within the workspace. Ensure they fit within the Combinatorial Maximalist Doctrine and execute seamlessly as part of the overall pipeline.

## Acceptance Criteria

### Verification Integrity
- [ ] `mud_gap_check.py` functionality is fully replicated by a new native Rust tool.
- [ ] The new Rust gap checker produces a deterministic pass/fail output without human intervention.
- [ ] The system continues to generate successfully using `ggen sync` without errors.

**CRITICAL OVERRIDE FROM USER**: Ensure you launch a 10-agent team to accomplish this, matching the user's initial `/teamwork-preview launch 10 agents to complete GC-MECH-FACTORY-MUD-002` command.
</USER_REQUEST>
