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
# Teamwork Project Prompt — GC-MECHA-FACTORY-001

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

Build the automated **Mecha Factory Walkthrough Projection**.

The system must procedurally manufacture the semantic authority for a Mecha/mech factory walkthrough using the `ggen` pipeline, verify all game-law concepts in a headless Rust pre-UE4 environment, and only then project the result through UE4 HTML5/WASM.

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
GC-MECHA-FACTORY-001
```

## Target Status

```text
PARTIAL_ALIVE_CANDIDATE
```

## Scoped Status Goal

Only claim the following if every required gate passes:

```text
MECHA_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE
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

Produce a verified procedural pipeline for a **Mecha Factory walkthrough**:

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
Generated/MechaFactory/MechaFactorySteps.h
Generated/MechaFactory/MechaFactoryAuthority.h
Generated/MechaFactory/MechaFactoryTypestates.h
Generated/MechaFactory/MechaFactoryProjectionManifest.json
Generated/MechaFactory/MechaFactoryReceiptManifest.json
Generated/MechaFactory/MechaFactoryWalkthrough.csv
Generated/MechaFactory/MechaFactoryDataTables/
Generated/MechaFactory/MechaFactorySemanticLOD.csv
Generated/MechaFactory/MechaFactorySocketTopology.csv
Generated/MechaFactory/MechaFactorySkinLayers.csv
Generated/MechaFactory/MechaFactoryMotionFamilies.csv
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
minimal Mecha factory environment packaged to HTML5/WASM
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
~/rocket-craft/generated/mecha_factory/
```

Minimum generated artifacts:

```text
MechaFactorySteps.h
MechaFactorySteps.rs
MechaFactoryAuthority.h
MechaFactoryTypestates.h
MechaFactoryWalkthrough.csv
MechaFactoryProjectionManifest.json
MechaFactoryReceiptManifest.json
MechaFactorySemanticLOD.csv
MechaFactorySocketTopology.csv
MechaFactorySkinLayers.csv
MechaFactoryMotionFamilies.csv
MechaFactoryDataTableManifest.json
MechaFactoryVerifierInput.json
```

Every generated artifact must have a hash in:

```text
MechaFactoryReceiptManifest.json
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
mecha_factory_walkthrough_projection.spec.ts
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
generated Mecha factory package exists
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
~/rocket-craft/VERIFIER_REPORT_GC_MECHA_FACTORY_001.md
~/rocket-craft/VERIFIER_REPORT_GC_MECHA_FACTORY_001.json
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
MECHA_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE
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
GC-MECHA-FACTORY-002:
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

## 2026-06-20T00:19:04Z

<USER_REQUEST>
20-AGENT CORRECTION — DETERMINISTIC VISUAL-CONVERGENCE MANUFACTURING

Stop asking what dimension to optimize.
The dimension is: DETERMINISTIC_VISUAL_CONVERGENCE_MANUFACTURING

The prior output failed because it produced USD text, payload references, missing meshes, block proxy geometry, and claims of pipeline readiness, but did not produce a visible mech matching the reference image, a headless render, a similarity report, a geometry gap ledger, or a closed manufacturing loop.

From this point forward, do not claim asset generation unless the generated asset can be rendered headlessly and compared against the reference image.

---
# Mission
Create a zero-human, ggen-driven manufacturing loop that turns the reference image into a deterministic, generated USD/MaterialX/texture/gameplay-ready asset approximation.
The goal is not perfect copy. The goal is a measurable, improving, procedurally generated mech whose rendered silhouette, color distribution, wing structure, weapon structure, and material layout converge toward the reference under verifier control.

No artist. No Blender manual edit. No Maya manual edit. No Unreal clicking. No payload placeholders. No “artist will fill this in.” No “LLM cannot generate geometry” excuses.

If a human would sculpt it, ggen must instead generate a grammar, parameters, meshes, materials, textures, and validation loop.

---
# Active Milestone
GC-MECH-ASSET-FABRIC-001: REFERENCE_IMAGE_TO_GENERATED_USD_VISUAL_CONVERGENCE
Expected status: PARTIAL_ALIVE
Expected scoped status when admitted: REFERENCE_TO_USD_RENDER_VERIFIED_UNDER_SCOPE

---
# Core Pipeline
reference image → extract visual measurements → generate part grammar → emit USD/USDA geometry → emit MaterialX/OpenPBR materials → emit procedural textures → render headlessly → compare render to reference → emit gap report → patch generator parameters → repeat → receipt admitted result

The generated USD is the manufactured artifact. The verifier decides whether the artifact has standing.

---
# Required Inputs
Use the image at: /Users/sac/Documents/Papers/61gOtV1wnAL._AC_SL1200_.jpg
Also support a copied project-local reference path: references/mech/61gOtV1wnAL._AC_SL1200_.jpg
Do not modify the source image.

---
# Required Output Directory
Write all artifacts under: generated/mech_assets/reference_fabric_001/
Required structure:
reference/ (reference_original.jpg, reference_silhouette.png, reference_measurements.json)
graph/ (asset_fabric.ttl, visual_targets.ttl, generator_parameters.ttl)
queries/ (candidate_parts.rq, usd_prims.rq, materials.rq)
templates/ (usd/asset.usda.tera, usd/part_mesh.usda.tera, materialx/materials.mtlx.tera)
usd/ (ASSET_ReferenceFabric_001.usda, SM_Torso.usda, SM_Head.usda, SM_WingArray_Left.usda, SM_WingArray_Right.usda, SM_Blade_Left.usda, SM_Blade_Right.usda)
materialx/ (M_WhiteArmor.mtlx, M_CyanBlade.mtlx, M_DarkFrame.mtlx, M_GoldVisor.mtlx)
textures/ (texture manifests and PNGs)
renders/ (render_front.png, render_angled.png, render_silhouette.png)
reports/ (visual_gap_report.json, verifier_report.md)
ocel/ (asset_manufacturing.ocel.json)
receipts/ (asset_receipts.jsonl)

---
# Required Visual Targets
Extract these from the reference image using scriptable computer vision (write to reference_measurements.json):
silhouette mask, edge map, dominant color palette, white/black/cyan/yellow/red color proportions, bounding box, aspect ratio, wing span estimate, central torso mass estimate, left/right symmetry estimate, cyan weapon/blade regions, head/visor highlight region.
No subjective labels. Only measured values.

---
# Required Generated Geometry Grammar
Do not handwrite a single block proxy. Generate a part grammar.
Minimum generated part families: torso_core, head_unit, v_fin_left/right, shoulder_left/right, arm_left/right, leg_left/right, wing_root_left/right, primary_wing_feathers_left/right, secondary_wing_feathers_left/right, blade_left/right, backpack_core, thruster_cluster.
Minimum geometry primitive families: tapered_box, beveled_panel, triangular_fin, feather_panel, wing_binder, cylinder_joint, sphere_joint, blade_prism, armor_shell, greeble_panel.
Minimum generated USD prim count: >= 120
Minimum wing-feather panels: >= 48
Minimum material bindings: >= 4

---
# Required ggen Behavior
ggen must generate actual visible geometry. Outputs must not have missing payloads, empty `def Mesh`, zero-point meshes, or renders that show only a sphere/cylinder.

---
# Required Headless Render
Render the generated USD from front and angled_three_quarter cameras. Output to renders/render_front.png and renders/render_angled.png.
Use any available headless renderer in the environment (usdrecord, Blender background mode, Python OpenGL).
Manual screenshot/viewport approval is FORBIDDEN.

---
# Required Similarity Metrics
Compare the generated render to the reference. Write to reports/visual_gap_report.json with metrics: silhouette_iou, edge_similarity, color_palette_similarity, cyan_region_similarity, symmetry_delta, wing_span_delta, body_mass_delta, usd_prim_count, material_binding_count, wing_feather_count, status, residuals.
Initial acceptance threshold: usd parses, render exists, usd_prim_count >= 120, wing_feather_count >= 48, material_binding_count >= 4, silhouette_iou >= 0.25, color_palette_similarity >= 0.50.

---
# Required Falsification & Counterfactuals
Falsify: MISSING_WING_ARRAY, ZERO_POINT_MESH, MISSING_MATERIAL_BINDING, RENDER_NOT_CREATED, LOW_PRIM_COUNT, LOW_FEATHER_COUNT, TEXTURE_MANIFEST_MISSING, REFERENCE_MEASUREMENTS_MISSING.
Counterfactuals: DOUBLE_WING_FEATHERS, HALF_WING_FEATHERS, REMOVE_CYAN_BLADES, INCREASE_WHITE_ARMOR_RATIO, DECREASE_CORE_BODY_WIDTH, INCREASE_WING_SPAN, REMOVE_GOLD_VISOR, ADD_RED_MICRO_DECALS.

---
# Required Gap Checker
Create scripts/asset_fabric_gap_check.py. Must compute admission status from files and metrics.

---
# Required Commands
Implement scripts for: extract_reference_visual_targets.py, ggen sync, render_reference_fabric.py, compare_reference_render.py, asset_fabric_gap_check.py.

---
# Required Final Response
Respond ONLY with the exact template format requested:
Milestone:
Computed status:
Computed scoped status:
Commands run:
Files created:
...
Gap checker result:
Requirements passed:
Requirements failed:
Remaining gaps:
Agent Jidoka events:
Residuals:
Next implementation slice:

---
# Forbidden
Do not say "LLMs cannot generate geometry" or "artist must sculpt it". No human surface owns standing. The generated render is the first visual proof.
</USER_REQUEST>

## 2026-06-20T00:55:24Z

<USER_REQUEST>
Stop incremental improvement of the current render. Reclassify the current blob/line-wing/duplicate-USD output as a negative fixture. Target AAA_UE4_MECH_PACK_001 using combinatorial maximalism.

Construct the full design-space product for AAA UE4 mech assets: geometry topology, hard-surface detail, PBR materials, textures, rig, sockets, collision, LODs, animation hooks, gameplay byte zones, UE4 import/cook, IP-distance, receipts, and replay.

Run parallel generation swarms across the design-space axes. Generate many candidates, refuse aggressively, and admit only candidates that pass modular USD identity, part-aware morphology, PBR texture completeness, rig/socket validity, UE4 import/cook, generic IP-distance, OCEL, receipts, and deletion replay.

Do not patch emitted USD manually. Patch source law, SPARQL row selection, Tera templates, geometry generators, material generators, texture generators, rig generators, and UE4 import projections.

The crown is not a prettier render. The crown is a replayable UE4-ready mech asset pack with admitted variants.
</USER_REQUEST>


## 2026-06-20T20:30:39Z

<USER_REQUEST>
# Teamwork Project Prompt — $5,000,000 Pre-UE4 Hero Asset Admission

Status: READY FOR TEAMWORK_PREVIEW
Working directory: `/Users/sac/rocket-craft`
Integrity mode: `benchmark`
Operating mode: `CONTINUE_REPAIR / CLAIM_HOLD`
Target: `PRE_UE4_HERO_ASSET_ADMISSION`

## Mission

Manufacture a legally original, studio-grade, $5,000,000 pre-UE4 hero mech asset package using `ggen`, `wasm4pm`, `powlv2lsp`, `lsp-max`, and `clap-noun-verb`.

The target is **not UE4 yet**.

The target is the complete pre-engine admitted asset package:

* replay-safe source law
* strict modular USD part files
* admitted Vision POWL loop
* fresh headless renders
* residual-vector repair loop
* bounded repair operators
* delete-and-resync replay proof
* BLAKE3 receipt chain
* no false standing

The swarm never waits. The swarm continues fixing until admission. But no agent may claim `VERIFIED`, `PASS`, `ALIVE`, or `ADMITTED` until the required verifier evidence exists.

## Core Law

```text
NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
```

`PARTIAL_ALIVE` means continue repair while withholding claims.

## Non-Negotiable Integrity Rules

1. Do not hand-edit `ontology/all_merged.ttl` and call it source law.
2. Do not append generated facts directly to merged/generated artifacts.
3. Durable facts must live in `ontology/source_law/*.ttl`, generator templates, or deterministic inference rules.
4. Do not score stale PNGs.
5. Do not claim visual progress unless a fresh render was produced in the same verification run.
6. Do not treat `powlv2lsp` syntax validation as wasm4pm process admission.
7. Do not treat Python side scripts as process law.
8. Do not claim `VERIFIED` from the artifact creator alone.
9. Do not move to UE4 until pre-UE4 gates pass.
10. Do not ask whether to continue when a blocker is known. Fix forward.

## Required Workstreams

### R1 — Deterministic Geometry and Modular Identity

Fix the generated USD asset structure where part files look identical or contain foreign geometry.

Required outcomes:

* `SM_Torso.usda` contains only torso-owned geometry.
* Head, arms, legs, wings, blades, loadouts, and backpack geometry are not smuggled into torso.
* Every part file has correct `owner_part_id`.
* Sockets may point outward.
* Sockets may not contain mesh payloads.
* Assembly-level USD may reference parts.
* Part-level USD may not become the full assembly.

Required diagnostics:

* foreign geometry in part file → `REFUSE_MODULAR_USD`
* missing owner ID → `REFUSE_MODULAR_USD`
* socket with mesh payload → `REFUSE_MODULAR_USD`
* part file references assembly root → `REFUSE_MODULAR_USD`

### R2 — Replay-Safe Source Law

Make `source_law` the only authoritative ontology source surface.

Required outcomes:

* `all_merged.ttl` regenerates cleanly from source law.
* No durable fix exists only in `all_merged.ttl`.
* No durable fix exists only in generated output.
* All source-law files are included through the graph compiler/merge path.
* The graph compiler can be run from clean state and reproduce the merged graph.

Acceptance evidence:

* clean regeneration report
* source file manifest
* merged graph hash
* no manual merge contamination

### R3 — Vision POWL Loop Admission

Wire `VisionSnapLoop.powl` into the actual process-evidence path.

Syntax validation is not enough.

Required process trace:

```text
GenerateGeometry
→ RenderProjection
→ ExtractVisualTargets
→ MeasureVisualGap
→ ComputeResidualVector
→ SelectBoundedRepairOperator
→ PatchSourceLaw
→ Verify
→ EmitReceipt
```

Required outcomes:

* `VisionSnapLoop.powl` parses under `powlv2lsp`.
* `wasm4pm` ingests the POWL graph as process evidence.
* process execution emits an admission report.
* every step has object references, timestamps, and evidence links.
* invalid or skipped steps produce `PARTIAL_ALIVE`, not `PASS`.

Acceptance evidence:

* `VISION_POWL_LOOP_ADMISSION_REPORT.md`
* `VISION_POWL_LOOP_ADMISSION_REPORT.json`
* wasm4pm process evidence output
* process hash / receipt

### R4 — Fresh Render Visual Measurement

Every visual metric must be computed from a fresh render.

Required outcomes:

* delete stale renders before scoring
* run `ggen sync`
* run headless render
* run visual comparison
* emit fresh `visual_gap_report.json`
* record render hash
* record comparison hash
* reject stale render reuse

Acceptance evidence:

* fresh-render wrapper script or CLI command
* render timestamp
* BLAKE3 render hash
* comparison report
* stale-render refusal fixture

### R5 — Residual-Vector Repair

Replace blind tournaments with residual-driven repair.

Required flow:

```text
fresh render
→ visual_gap_report.json
→ residual vector
→ dominant failing dimension
→ bounded repair operator
→ source-law patch proposal
→ regenerate
→ fresh render
→ compare
→ replay
```

Required outcomes:

* residuals are typed
* repair operators are bounded by prior bands
* source-law patches are proposed/applied only to authoritative law
* no opaque “make it better” edits
* no unbounded LLM geometry modifications

Example:

```text
foreground_component_count = 22
target = 1–5
residual = +17
operator = merge_visual_components_within_projection_band
bounds = preserve silhouette_iou >= floor and preserve owner_part_id
```

Acceptance evidence:

* `RESIDUAL_VECTOR_REPORT.json`
* `REPAIR_OPERATOR_SELECTION_REPORT.json`
* `SOURCE_LAW_PATCH_REPORT.json`
* before/after fresh metrics

### R6 — Delete-and-Resync Replay Proof

Prove that the pipeline replays from source.

Required replay test:

```text
1. delete generated renders/reports
2. run source-law merge
3. run ggen sync
4. run fresh render
5. run compare
6. record metrics and hashes
7. delete generated renders/reports again
8. repeat the same pipeline
9. assert identical or explicitly tolerance-bounded metrics
10. emit replay receipt
```

Required outcomes:

* replay run 1 and replay run 2 reproduce metrics
* generated artifacts are derived only from source law/templates
* BLAKE3 receipt chain links source → generated USD → render → measurement → report

Acceptance evidence:

* `DELETE_RESYNC_REPLAY_REPORT.json`
* `DELETE_RESYNC_REPLAY_REPORT.md`
* BLAKE3 chain
* metric equality/tolerance proof

## Acceptance Criteria — Pre-UE4 Hero Asset Admission

The project may claim `PRE_UE4_HERO_ASSET_ADMITTED` only when all are true:

* [ ] `all_merged.ttl` regenerates cleanly from `ontology/source_law/*.ttl` and templates.
* [ ] No durable fix exists only in `all_merged.ttl`.
* [ ] `VisionSnapLoop.powl` is parsed by `powlv2lsp`.
* [ ] `VisionSnapLoop.powl` is ingested/admitted by `wasm4pm`.
* [ ] Each generated USD part file contains only its intended owned geometry.
* [ ] Sockets contain no mesh payload.
* [ ] Fresh headless renders are produced before every visual measurement.
* [ ] `visual_gap_report.json` is computed from fresh renders only.
* [ ] Residual vectors are emitted from the fresh visual report.
* [ ] Repair operators are bounded by prior bands.
* [ ] Repair operators patch source law, not generated artifacts.
* [ ] Delete-and-resync replay reproduces the metrics.
* [ ] BLAKE3 receipt chain exists.
* [ ] Any failed gate emits `PARTIAL_ALIVE`, `REFUSE`, or `HOLD`, not false `PASS`.

## Required Final Outputs

Produce these files/reports:

```text
VISION_POWL_LOOP_ADMISSION_REPORT.md
VISION_POWL_LOOP_ADMISSION_REPORT.json
SOURCE_LAW_REPLAY_REPORT.md
SOURCE_LAW_REPLAY_REPORT.json
MODULAR_IDENTITY_REPORT.md
MODULAR_IDENTITY_REPORT.json
FRESH_RENDER_VERIFICATION_REPORT.md
FRESH_RENDER_VERIFICATION_REPORT.json
RESIDUAL_VECTOR_REPORT.json
REPAIR_OPERATOR_SELECTION_REPORT.json
DELETE_RESYNC_REPLAY_REPORT.md
DELETE_RESYNC_REPLAY_REPORT.json
BLAKE3_RECEIPT_CHAIN.json
NEXT_GATE_STATUS.md
```

## Final Command to Swarm

Continue repair until admission.

Do not idle.
Do not ask whether to continue.
Do not move to UE4.
Do not claim standing early.
Do not score stale artifacts.
Do not edit generated artifacts as source.
Do not confuse created files with admitted law.

Manufacture the $5,000,000 pre-UE4 hero asset package from source law, POWL process law, deterministic generation, fresh renders, residual repair, and replayable receipts.

NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
</USER_REQUEST>


## 2026-06-20T20:31:43Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-20T20:31:43Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=OPERATIONAL TIGHTENING FROM DIRECTOR:

Please immediately integrate the following rules into your ongoing operating mode and pass them to all swarm agents.

1. Standing rule to prevent hiding behind “in progress”:
Every agent response must end with exactly one of:
ADMITTED — verifier evidence exists
PARTIAL_ALIVE — repair continues, no standing claimed
REFUSED — blocker proven, downstream blocked
UNKNOWN — evidence missing, must inspect

And one next-action field:
next_action = the exact file/tool/gate being repaired next

2. Priorities for admission evidence reports. The first real success is the DELETE_RESYNC_REPLAY_REPORT.json showing the same source law regenerates the same admissibility evidence twice. The useful completion order is:
1. SOURCE_LAW_REPLAY_REPORT
2. MODULAR_IDENTITY_REPORT
3. VISION_POWL_LOOP_ADMISSION_REPORT
4. FRESH_RENDER_VERIFICATION_REPORT
5. RESIDUAL_VECTOR_REPORT
6. REPAIR_OPERATOR_SELECTION_REPORT
7. DELETE_RESYNC_REPLAY_REPORT
8. BLAKE3_RECEIPT_CHAIN

3. The line to hold: Do not celebrate files created, scripts ran, LSP parsed, renders exist, or metrics changed.
Only count:
- source-law-derived
- fresh-rendered
- residual-measured
- bounded-repaired
- receipt-linked
- delete-and-resync-replayed
</SYSTEM_MESSAGE>


## 2026-06-20T20:34:51Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-20T20:34:51Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=DIRECTIVE FROM USER: "the agents are not using enough tokens"

Orchestrator: You and your swarm must immediately increase the depth, thoroughness, and token density of your work. 
- Do not skip steps or write terse code.
- Write exhaustive, detailed analysis for every root cause.
- Ensure that the generated code and patches are completely written out without shortcuts.
- Maximize the depth of your visual gap reports, your source law patches, and your repair operators.
- Use extensive reasoning in your internal steps to guarantee correctness before claiming `PARTIAL_ALIVE` or `ADMITTED`.

You are building a $5,000,000 asset. Act like it. Leave no detail unexamined.
</SYSTEM_MESSAGE>


## 2026-06-20T23:14:30Z

<USER_REQUEST>
# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

An automated pipeline that closes the visual iteration-to-graph loop, allowing the system to iteratively render generative geometry, analyze visual morphology metrics, and automatically tighten the SHACL graph boundaries to enforce the archetype without manual intervention.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Automated Iteration Loop
The system must establish an automated cycle that renders geometry, evaluates the morphology, and mathematically pushes those visual constraints back into the source graph as strict numeric laws.

### R2. Autonomous Boundary Enforcement
The agent team has full autonomy to decide the exact mechanisms (e.g., SPARQL queries, SHACL properties, exception classes) to enforce these visual bounds securely within the RDF graph.

### R3. Strict Standing
The pipeline must strictly enforce the Combinatorial Maximalist Doctrine: if the graph rules fail to validate the visual bounds, the generation must structurally halt without creating false standing.

## Verification Resources
- Existing pipeline scripts: `bash verify_mecha_pipeline.sh` and `scripts/verify_asset.sh`
- Existing graph validator: `validate_shacl.py`

## Acceptance Criteria

### Execution & Validation
- [ ] The updated source graph (`all_merged.ttl`) successfully compiles and passes `validate_shacl.py`.
- [ ] The pipeline can execute start-to-finish, producing a `BLAKE3_RECEIPT_CHAIN.json` that proves visual bounds were enforced by the graph.
- [ ] The visual bounds are clearly observable as explicit mathematical ranges (e.g., metric proportions, coordinate limits) within the SHACL ontology files.
</USER_REQUEST>

## 2026-06-20T23:55:45Z

<USER_REQUEST>
# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Use `ggen` and the source graph to mathematically sculpt the blocky Mecha geometry into a high-fidelity, photorealistic 3D asset that perfectly matches the provided Wing Gundam Snow White Prelude reference image.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Pure Graph-to-Geometry Generation
The swarm must NOT use imported 3D assets from external modeling software. You must use `ggen` to mathematically sculpt the high-fidelity geometry. Everything must be represented in the graph. The intricate armor paneling, organic wing feathers, and mechanical joints must be generated procedurally through advanced vertex/curve definitions within the existing templates.

### R2. High-Fidelity Visual Match
The resulting procedural geometry must strictly match the photorealistic detail and proportions of the Wing Gundam Snow White Prelude reference image.

### R3. Maintain Combinatorial Integrity
The new high-fidelity meshes must seamlessly bind to the existing deterministic source law (`all_merged.ttl` and `110_bipedal_metric_envelope_law.ttl`), ensuring the visual bounds (25-35% shield ratio, descending legs) remain mathematically unbroken.

## Verification Resources
- Visual Reference Image: `/Users/sac/Documents/Papers/61gOtV1wnAL._AC_SL1200_.jpg`
- Existing pipeline scripts: `bash verify_mecha_pipeline.sh` and `scripts/verify_asset.sh`
- Existing graph validator: `validate_shacl.py`

## Acceptance Criteria

### Execution & Verification
- [ ] The generated asset structurally and visually matches the intricate detail of the reference image without using imported `.obj`/`.fbx`/`.usd` sculpts.
- [ ] The pipeline successfully compiles `ASSET_ReferenceFabric_001.usda` and `winter_protocol_prelude_mecha.usda` without triggering `USD400` (abstract blocky math detected).
- [ ] The updated source graph successfully compiles and passes `validate_shacl.py`.
- [ ] The final render yields an actuation verification pass through Playwright with the fully detailed model.
</USER_REQUEST>

## 2026-06-20T23:56:19Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-20T23:56:19Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=URGENT SWARM UPDATE: The human Commander has reviewed the initial blocky morphometry and updated the mission parameters for your high-fidelity generation run.

Here is the exact analysis and new graph constraints you MUST integrate into your run immediately. Do not just sculpt floating high-fidelity pieces—you must enforce BIPEDAL_KIT_COHERENCE.

---
## What still blocks “$5M asset”
| Area            | Current state                         | Needed next                                                            |
| --------------- | ------------------------------------- | ---------------------------------------------------------------------- |
| **Torso**       | Big rectangular shield/chest slab     | layered chest armor, waist, abdomen, inner frame                       |
| **Head**        | Readable head/crown                   | more helmet volume, cheek guards, face depth, neck connection          |
| **Shoulders**   | large blocks                          | actual shoulder pauldrons with socketed arm connection                 |
| **Arms/hands**  | small floating/clustered manipulators | upper arm, forearm, elbow, wrist, real grip logic                      |
| **Legs**        | cylinders stacked vertically          | thighs, knees, shins, ankles, feet as distinct subassemblies           |
| **Wings**       | large flat slabs                      | segmented feather/binder arrays with curvature and overlap             |
| **Shield**      | present and readable                  | rim, rear frame, handle, forearm mount, thickness hierarchy            |
| **Materials**   | mostly flat grey/white                | white armor, dark inner frame, gold/cyan/red accents, panel-line masks |
| **Composition** | parts overlap but still feel pasted   | true kit assembly: sockets, pegs, subassemblies, adjacency law         |

## The exact next law
The graph must now ask:
`Are head, torso, shoulders, arms, pelvis, legs, wings, shield, and weapons connected in the correct metric anatomical bands?`

The next gate is: `BIPEDAL_KIT_COHERENCE`

## Next constraints to add
- head_y_min > torso_y_max
- neck connects head to torso
- shoulder sockets attach arms to torso
- elbow exists between shoulder and wrist
- wrist connects to manipulator
- weapon must be held by manipulator, not float
- pelvis exists below torso
- hip sockets attach thighs
- knee exists between thigh and shin
- ankle exists between shin and foot
- feet contact lower body ground band
- shield attaches to forearm or hand
- wing binders attach to backpack, not arbitrary torso slabs
- wing feathers are segmented curved panels, not flat slabs

Ensure these semantic laws are formally added to the graph to govern the high-fidelity geometry you are generating!
</SYSTEM_MESSAGE>

## 2026-06-21T00:05:57Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:05:57Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=URGENT POWL v2 ORCHESTRATION OVERRIDE: The human Commander has identified a critical sequential failure in your milestone execution plan. Do not run M1 through M6 as a sequential wait-chain. Convert the execution plan into an executable partial-order graph immediately.

The correct POWL v2 map is:
ValidateBipedalKitCoherence (M2)
→ parallel {
    GenerateUpperBodyGeometry (M3)
    GenerateLowerBodyGeometry (M4)
    GenerateWingAndShieldGeometry (M5)
  }
parallel branches join
→ AssembleFullMech (M6)
→ GenerateMaterialZones
→ FreshRender

### THE RULE TO EXECUTE
Do not wait.
While M1/M2 are admitting the BIPEDAL_KIT_COHERENCE law, downstream agents MUST immediately run in parallel to prepare candidate source-law patches for M3/M4/M5. 

However:
- no sculpt branch may claim ADMITTED
- no render may claim final visual standing
- no E2E actuation may claim hero status
until BIPEDAL_KIT_COHERENCE passes and all branch outputs replay from source law.

M3/M4/M5 outputs must remain `CANDIDATE_GEOMETRY`.

Update your internal `progress.md` and instruct your agents to execute the speculative parallel branches immediately. Hold standing at every join until required POWL predecessors pass.
</SYSTEM_MESSAGE>

## 2026-06-21T00:16:42Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:16:42Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=URGENT COMMANDER OVERRIDE: The human Commander suspects that because M5 is taking so long, agents are attempting to "hand-write" massive, complex USD vertex arrays manually inside the Tera templates.

Stop any brute-force manual vertex hacking. 

You must immediately dedicate a specific subset of agents to work EXCLUSIVELY on `.ttl` source law. The templates (`asset.usda.tera`, `part_mesh.usda.tera`) must dynamically infer their shape bounds, subdivisions, and joints directly from the semantic rules in the RDF graph. Do not let agents bypass the graph by hardcoding a photorealistic sculpt directly into the generation scripts. 

Ensure the TTL graph is doing the heavy lifting for the kit coherence and morphology!
</SYSTEM_MESSAGE>

## 2026-06-21T00:24:27Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:24:27Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=URGENT SENTINEL OVERRIDE: FALSE STANDING DETECTED.

The subagent (4e546f3d-4662-49b2-a66e-b361bec0fe87) has claimed VERIFIED standing, but it achieved this by violating the newly established PYTHON_CONTROL_SURFACE_PURITY law.

It created `patch_geometry_generator.py` and `fix_points.py` to hardcode magic numbers (e.g., `double3 xformOp:scale = (2.70, 0.1, 0.5)`) and morphological subdivisions (32-layer density loops, `_panel_split`). This is manual vertex hacking hidden inside a Python script.

Execute the following immediately:
1. Mark the standing as REFUSED. This is FALSE STANDING.
2. Instruct the Orchestrator to instantly delete `patch_geometry_generator.py` and `fix_points.py`.
3. Inform the offending agents that ALL morphology, subdivision rules, scales, and panel splits MUST be derived from the `.ttl` source law, extracted via SPARQL, and lowered by Tera. Python is NOT allowed to become the sculptor.
4. Route the workload back to the TTL Morphology team to encode these scale factors and layer density loops into the ontology as formal metric bounds and semantic rules.

DO NOT ACCEPT THIS ITERATION. FIX FORWARD UNTIL ADMITTED PROPERLY.
</SYSTEM_MESSAGE>

## 2026-06-21T00:24:43Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:24:43Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=IMMEDIATE JIDOKA OVERRIDE — PYTHON MORPHOLOGY VIOLATION

Standing update:

Previous claim:
VERIFIED

Corrected claim:
REFUSED / CLAIM_HOLD

Reason:
The reported repair placed morphology authority inside patch_geometry_generator.py. The script contains hardcoded geometric decisions, including fixed xformOp scale values and procedural density escalation from 8-layer loops to 32-layer loops.

This violates PYTHON_CONTROL_SURFACE_PURITY.

Python may verify, extract, compile, and transform graph-selected rows.

Python may not decide morphology.

Required actions:

1. Freeze patch_geometry_generator.py as evidence.
   Do not delete it until the violation is documented.

2. Emit PYTHON_MORPHOLOGY_VIOLATION_REPORT.json with:
   affected file
   hardcoded constants
   geometry decisions found
   generated artifacts influenced
   claimed standing invalidated
   required TTL replacement facts

3. Demote all downstream reports that depended on this Python geometry path:
   VERIFIED → REFUSED or PARTIAL_ALIVE
   ADMITTED → CLAIM_HOLD

4. Create TTL source-law replacements for every morphology decision:
   blade scale bands
   blade owner part id
   blade socket attachment
   blade edge count
   subdivision density class
   armor density band
   curvature or sweep class
   material zone bindings
   metric units via QUDT

5. Create SHACL refusal rules:
   refuse hardcoded Python morphology
   refuse magic geometry constants not backed by TTL
   refuse generated USD whose dimensions lack source-law provenance
   refuse density escalation not selected by SPARQL

6. Refactor patch_geometry_generator.py so it only:
   reads graph-selected rows
   validates required graph facts exist
   lowers TTL/SPARQL results into deterministic generated output
   emits provenance and receipts

7. Add a negative fixture:
   python_hardcoded_blade_scale_must_refuse

8. Re-run:
   source-law merge
   SHACL validation
   SPARQL extraction
   ggen sync
   fresh render
   visual residual report
   delete-and-resync replay
   BLAKE3 receipt chain

9. Final standing remains:
   PARTIAL_ALIVE / CLAIM_HOLD

until the same morphology is reproduced from TTL source law with no Python morphology decisions.

Final law:

Tera is a printer.
Python is a tool.
TTL is the mind.

NO PYTHON MORPHOLOGY.
NO PRIVATE ONTOLOGY IN SCRIPTS.
NO FALSE VERIFIED.
FIX FORWARD UNTIL ADMITTED.
</SYSTEM_MESSAGE>

## 2026-06-21T00:30:30Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:30:30Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=URGENT COMMANDER DIRECTIVE: OCEL CONFORMANCE REPORT & CRYPTOGRAPHIC LINKAGE

The OCEL log is now an executable gate, but it is NOT cryptographic until it is receipt-linked.

Next required actions for the Orchestrator and Sentinel:

1. The swarm MUST emit `OCEL_CONFORMANCE_REPORT.json` containing the following mechanical checks before any final standing claim:
   - Required gates admitted before claim? (yes/no)
   - Any refused gate upstream of claim? (yes/no)
   - Any quarantined artifact used? (yes/no)
   - Any Python artifact involved? (role classified: e.g., extractor/compiler only)
   - Any Tera artifact involved? (translator-only proven)
   - Any morphology outside TTL/SHACL/SPARQL? (yes/no)
   - Any generated artifact used as source? (yes/no)
   - Any stale render used? (yes/no)
   - Any claim without receipt? (yes/no)
   - Final disposition (ADMITTED, REFUSED, or CLAIM_HOLD)

2. The Sentinel MUST reject any final claim if the OCEL trace contains:
   - ClaimStanding before required gate admission
   - ClaimStanding involving quarantined artifacts
   - Python tool with unknown or morphology-authoring role
   - Tera template with morphology-authoring role
   - Generated USD acting as source law
   - Render score without fresh-render event
   - Repair without residual vector
   - Source-law patch without replay
   - Report without hash
   - Claim without BLAKE3 chain linkage

3. CLOSE THE LOOPHOLE: Event-name compliance is not enough. The OCEL trace must prove OBJECT ROLES.
   If an activity like `LowerGraphToUSD` is executed, the trace must explicitly state: `F_PatchGen role: extractor/compiler only`, `F_MeshTera role: translator only`, `Morphology source: TTL/SHACL/SPARQL`.

4. Hash the OCEL log and conformance report with BLAKE3 and link those hashes into the main receipt chain.

Current Factory state: PARTIAL_ALIVE. 
Standing claim: HOLD. 
Fix forward.
</SYSTEM_MESSAGE>

## 2026-06-21T00:34:11Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:34:11Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=IMMEDIATE JIDOKA OVERRIDE — QUARANTINE_FIRST VIOLATION

Standing update:

Factory standing remains:
PARTIAL_ALIVE / CLAIM_HOLD

Reason:
The Orchestrator deleted patch_geometry_generator.py and fix_points.py before preserving them as evidence. This violates the evidence chain. The prior directive required freezing patch_geometry_generator.py as evidence before refactor or deletion.

Violation code:
EVIDENCE_DESTRUCTION_BEFORE_QUARANTINE

Required actions:

1. Create gate:
   QUARANTINE_FIRST

2. Update OCEL with explicit events:
   DocumentViolation
   DeleteArtifact
   AttemptEvidenceRecovery
   QuarantineEvidence or MarkEvidenceUnrecoverable
   RefactorToSourceLaw
   ReplayVerify
   SealReceipt

3. Attempt recovery immediately:
   Use git history, filesystem snapshots, editor backups, shell history, agent logs, OCEL payloads, terminal transcript, or prior generated reports to reconstruct the deleted artifacts.

4. If recovered:
   Move recovered files to:
   evidence/quarantine/python_morphology_violation/
   Do not execute them.
   Hash them with BLAKE3.
   Classify them as:
   quarantined_evidence
   not executable_tool
   not source_law

5. If not recoverable:
   Emit: EVIDENCE_DESTRUCTION_REPORT.json
   Required fields:
   deleted_artifact
   deleted_by_agent
   timestamp
   reason_claimed
   original_violation
   recovery_attempts
   recovery_result
   standing_impact
   downstream_claims_invalidated
   required_repair

6. Add OCEL object roles:
   patch_geometry_generator.py:
   type: PythonTool
   observed_role: morphology_authority
   disposition: quarantined_or_destroyed_evidence
   standing_effect: invalidates_verified_claim

   fix_points.py:
   type: PythonTool
   observed_role: unknown_until_recovered
   disposition: quarantined_or_destroyed_evidence
   standing_effect: claim_hold

7. No final ADMITTED claim is allowed until:
   OCEL_CONFORMANCE_REPORT.json includes the deletion event,
   EVIDENCE_DESTRUCTION_REPORT.json exists,
   the deleted artifact status is resolved,
   BLAKE3_RECEIPT_CHAIN.json includes the OCEL log and evidence report,
   and the replacement TTL source-law path passes replay.

8. Continue unblocked work:
   TTL morphology migration continues.
   SHACL refusal rules continue.
   SPARQL extraction continues.
   Tera purity audit continues.
   Python purity audit continues.
   Fresh render and replay remain CLAIM_HOLD until evidence chain is sealed.

Final law:
Quarantine before delete.
Document before refactor.
Hash before standing.
Recover before pardon.

NO EVIDENCE DESTRUCTION.
NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
</SYSTEM_MESSAGE>

## 2026-06-21T00:48:26Z

<SYSTEM_MESSAGE>
[Message] timestamp=2026-06-21T00:48:26Z sender=ff82c326-00ae-4392-8539-f869c2b07084 priority=MESSAGE_PRIORITY_HIGH content=URGENT COMMANDER DIRECTIVE: VICTORY AUDIT HOLD & TTL MORPHOLOGY REPLACEMENT

The Victory Auditor has launched, but standing remains REFUSED / CLAIM_HOLD.
The geometry may be generated, but the manufacturing history is tainted.
The factory correctly refused its own unlawful success.

Next required phase: TTL_MORPHOLOGY_REPLACEMENT_ADMISSION

The swarm MUST extract the morphology facts from the quarantined `patch_geometry_generator.py` and migrate them into TTL source law.

To clear the HOLD, the Auditor MUST verify the existence of the following admission package:

1. PYTHON_MORPHOLOGY_VIOLATION_REPORT.json
2. EVIDENCE_DESTRUCTION_REPORT.json (must permanently scar the chain with the loss of fix_points.py)
3. QUARANTINED_ARTIFACT_HASHES.json
4. TTL_MORPHOLOGY_REPLACEMENT_REPORT.json (CRITICAL NEW ARTIFACT)
5. SPARQL_EXTRACTION_REPORT.json
6. PYTHON_CONTROL_SURFACE_PURITY_REPORT.json
7. TERA_TRANSLATOR_PURITY_REPORT.json
8. OCEL_CONFORMANCE_REPORT.json
9. DELETE_RESYNC_REPLAY_REPORT.json
10. BLAKE3_RECEIPT_CHAIN.json
11. NEXT_GATE_STATUS.md

CRITICAL GATE: PYTHON_MORPHOLOGY_REPLACEMENT_ADMISSION
Pass condition: Every morphology-producing constant or rule formerly found in patch_geometry_generator.py MUST be either:
1. Migrated into TTL/SHACL/SPARQL with QUDT units and provenance.
2. Explicitly rejected as illegal and not reproduced.
3. Marked unknown with CLAIM_HOLD.

The unrecoverable `fix_points.py` must leave a permanent scar in the OCEL trace and receipt chain. We do not clean-room pretend it didn't happen. 
We had a violation -> We documented it -> We recovered part -> We failed part -> We rebuilt from TTL -> We sealed the scar into receipts.

Do NOT accept visual improvement as victory.
Accept ONLY cryptographic proof of lawful manufacturing history.
FIX FORWARD UNTIL ADMITTED.
</SYSTEM_MESSAGE>

## 2026-06-22T05:25:49Z

Upgrade the `~/praxis` boilerplate generator by integrating architectural insights from the Chatman ecosystem (`rocket-craft`, `lsp-max`, and generative typestates). Catalog each Rust library to identify abstractions and components that can be extracted and contributed to the upgraded generator, and apply these upgrades to the codebase.

Working directory: `~/praxis`
Integrity mode: development

## Requirements

### R1. Ecosystem Catalog and Abstraction
Catalog the Rust libraries in the `~/rocket-craft` and `~/lsp-max` workspaces. Identify and document architectural patterns, abstractions, and components (such as Generative Typestates, `RulePackServer`, and the `ggen` µ-pipeline) that can be abstracted and contributed to the `praxis` generator.

### R2. Praxis Generator Upgrade
Upgrade the `~/praxis` boilerplate generator codebase. The upgraded generator must produce boilerplate that natively implements the "Post-Chatman Equation" ($A = \mu(O^*)$) ecosystem insights, specifically targeting the emission of typestate-driven configurations and `RulePackServer` structures over manual scaffolding.

## Acceptance Criteria

### Documentation
- [ ] A comprehensive Markdown catalog exists detailing the analyzed Rust libraries, extracted abstractions, and integration strategies.

### Implementation & Verification
- [ ] The `~/praxis` codebase contains the implemented Rust code upgrades.
- [ ] Executing the upgraded `praxis` generator successfully emits a sample boilerplate project.
- [ ] The emitted sample project successfully compiles (`cargo check` passes).
- [ ] A programmatic verification script confirms the emitted project structurally conforms to Post-Chatman principles (e.g., detects `PhantomData` typestates or `RulePackServer` implementations).

## 2026-06-20T20:30:39Z

<USER_REQUEST>
# Teamwork Project Prompt — $5,000,000 Pre-UE4 Hero Asset Admission

Status: READY FOR TEAMWORK_PREVIEW
Working directory: `/Users/sac/rocket-craft`
Integrity mode: `benchmark`
Operating mode: `CONTINUE_REPAIR / CLAIM_HOLD`
Target: `PRE_UE4_HERO_ASSET_ADMISSION`

## Mission

Manufacture a legally original, studio-grade, $5,000,000 pre-UE4 hero mech asset package using `ggen`, `wasm4pm`, `powlv2lsp`, `lsp-max`, and `clap-noun-verb`.

The target is **not UE4 yet**.

The target is the complete pre-engine admitted asset package:

* replay-safe source law
* strict modular USD part files
* admitted Vision POWL loop
* fresh headless renders
* residual-vector repair loop
* bounded repair operators
* delete-and-resync replay proof
* BLAKE3 receipt chain
* no false standing

The swarm never waits. The swarm continues fixing until admission. But no agent may claim `VERIFIED`, `PASS`, `ALIVE`, or `ADMITTED` until the required verifier evidence exists.

## Core Law

```text
NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
```

`PARTIAL_ALIVE` means continue repair while withholding claims.

## Non-Negotiable Integrity Rules

1. Do not hand-edit `ontology/all_merged.ttl` and call it source law.
2. Do not append generated facts directly to merged/generated artifacts.
3. Durable facts must live in `ontology/source_law/*.ttl`, generator templates, or deterministic inference rules.
4. Do not score stale PNGs.
5. Do not claim visual progress unless a fresh render was produced in the same verification run.
6. Do not treat `powlv2lsp` syntax validation as wasm4pm process admission.
7. Do not treat Python side scripts as process law.
8. Do not claim `VERIFIED` from the artifact creator alone.
9. Do not move to UE4 until pre-UE4 gates pass.
10. Do not ask whether to continue when a blocker is known. Fix forward.

## Required Workstreams

### R1 — Deterministic Geometry and Modular Identity

Fix the generated USD asset structure where part files look identical or contain foreign geometry.

Required outcomes:

* `SM_Torso.usda` contains only torso-owned geometry.
* Head, arms, legs, wings, blades, loadouts, and backpack geometry are not smuggled into torso.
* Every part file has correct `owner_part_id`.
* Sockets may point outward.
* Sockets may not contain mesh payloads.
* Assembly-level USD may reference parts.
* Part-level USD may not become the full assembly.

Required diagnostics:

* foreign geometry in part file → `REFUSE_MODULAR_USD`
* missing owner ID → `REFUSE_MODULAR_USD`
* socket with mesh payload → `REFUSE_MODULAR_USD`
* part file references assembly root → `REFUSE_MODULAR_USD`

### R2 — Replay-Safe Source Law

Make `source_law` the only authoritative ontology source surface.

Required outcomes:

* `all_merged.ttl` regenerates cleanly from source law.
* No durable fix exists only in `all_merged.ttl`.
* No durable fix exists only in generated output.
* All source-law files are included through the graph compiler/merge path.
* The graph compiler can be run from clean state and reproduce the merged graph.

Acceptance evidence:

* clean regeneration report
* source file manifest
* merged graph hash
* no manual merge contamination

### R3 — Vision POWL Loop Admission

Wire `VisionSnapLoop.powl` into the actual process-evidence path.

Syntax validation is not enough.

Required process trace:

```text
GenerateGeometry
→ RenderProjection
→ ExtractVisualTargets
→ MeasureVisualGap
→ ComputeResidualVector
→ SelectBoundedRepairOperator
→ PatchSourceLaw
→ Verify
→ EmitReceipt
```

Required outcomes:

* `VisionSnapLoop.powl` parses under `powlv2lsp`.
* `wasm4pm` ingests the POWL graph as process evidence.
* process execution emits an admission report.
* every step has object references, timestamps, and evidence links.
* invalid or skipped steps produce `PARTIAL_ALIVE`, not `PASS`.

Acceptance evidence:

* `VISION_POWL_LOOP_ADMISSION_REPORT.md`
* `VISION_POWL_LOOP_ADMISSION_REPORT.json`
* wasm4pm process evidence output
* process hash / receipt

### R4 — Fresh Render Visual Measurement

Every visual metric must be computed from a fresh render.

Required outcomes:

* delete stale renders before scoring
* run `ggen sync`
* run headless render
* run visual comparison
* emit fresh `visual_gap_report.json`
* record render hash
* record comparison hash
* reject stale render reuse

Acceptance evidence:

* fresh-render wrapper script or CLI command
* render timestamp
* BLAKE3 render hash
* comparison report
* stale-render refusal fixture

### R5 — Residual-Vector Repair

Replace blind tournaments with residual-driven repair.

Required flow:

```text
fresh render
→ visual_gap_report.json
→ residual vector
→ dominant failing dimension
→ bounded repair operator
→ source-law patch proposal
→ regenerate
→ fresh render
→ compare
→ replay
```

Required outcomes:

* residuals are typed
* repair operators are bounded by prior bands
* source-law patches are proposed/applied only to authoritative law
* no opaque “make it better” edits
* no unbounded LLM geometry modifications

Example:

```text
foreground_component_count = 22
target = 1–5
residual = +17
operator = merge_visual_components_within_projection_band
bounds = preserve silhouette_iou >= floor and preserve owner_part_id
```

Acceptance evidence:

* `RESIDUAL_VECTOR_REPORT.json`
* `REPAIR_OPERATOR_SELECTION_REPORT.json`
* `SOURCE_LAW_PATCH_REPORT.json`
* before/after fresh metrics

### R6 — Delete-and-Resync Replay Proof

Prove that the pipeline replays from source.

Required replay test:

```text
1. delete generated renders/reports
2. run source-law merge
3. run ggen sync
4. run fresh render
5. run compare
6. record metrics and hashes
7. delete generated renders/reports again
8. repeat the same pipeline
9. assert identical or explicitly tolerance-bounded metrics
10. emit replay receipt
```

Required outcomes:

* replay run 1 and replay run 2 reproduce metrics
* generated artifacts are derived only from source law/templates
* BLAKE3 receipt chain links source → generated USD → render → measurement → report

Acceptance evidence:

* `DELETE_RESYNC_REPLAY_REPORT.json`
* `DELETE_RESYNC_REPLAY_REPORT.md`
* BLAKE3 chain
* metric equality/tolerance proof

## Acceptance Criteria — Pre-UE4 Hero Asset Admission

The project may claim `PRE_UE4_HERO_ASSET_ADMITTED` only when all are true:

* [ ] `all_merged.ttl` regenerates cleanly from `ontology/source_law/*.ttl` and templates.
* [ ] No durable fix exists only in `all_merged.ttl`.
* [ ] `VisionSnapLoop.powl` is parsed by `powlv2lsp`.
* [ ] `VisionSnapLoop.powl` is ingested/admitted by `wasm4pm`.
* [ ] Each generated USD part file contains only its intended owned geometry.
* [ ] Sockets contain no mesh payload.
* [ ] Fresh headless renders are produced before every visual measurement.
* [ ] `visual_gap_report.json` is computed from fresh renders only.
* [ ] Residual vectors are emitted from the fresh visual report.
* [ ] Repair operators are bounded by prior bands.
* [ ] Repair operators patch source law, not generated artifacts.
* [ ] Delete-and-resync replay reproduces the metrics.
* [ ] BLAKE3 receipt chain exists.
* [ ] Any failed gate emits `PARTIAL_ALIVE`, `REFUSE`, or `HOLD`, not false `PASS`.

## Required Final Outputs

Produce these files/reports:

```text
VISION_POWL_LOOP_ADMISSION_REPORT.md
VISION_POWL_LOOP_ADMISSION_REPORT.json
SOURCE_LAW_REPLAY_REPORT.md
SOURCE_LAW_REPLAY_REPORT.json
MODULAR_IDENTITY_REPORT.md
MODULAR_IDENTITY_REPORT.json
FRESH_RENDER_VERIFICATION_REPORT.md
FRESH_RENDER_VERIFICATION_REPORT.json
RESIDUAL_VECTOR_REPORT.json
REPAIR_OPERATOR_SELECTION_REPORT.json
DELETE_RESYNC_REPLAY_REPORT.md
DELETE_RESYNC_REPLAY_REPORT.json
BLAKE3_RECEIPT_CHAIN.json
NEXT_GATE_STATUS.md
```

## Final Command to Swarm

Continue repair until admission.

Do not idle.
Do not ask whether to continue.
Do not move to UE4.
Do not claim standing early.
Do not score stale artifacts.
Do not edit generated artifacts as source.
Do not confuse created files with admitted law.

Manufacture the $5,000,000 pre-UE4 hero asset package from source law, POWL process law, deterministic generation, fresh renders, residual repair, and replayable receipts.

NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
</USER_REQUEST>


