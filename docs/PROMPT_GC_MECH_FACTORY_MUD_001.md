# GEMINI / CODE SWARM PROMPT — GC-MECH-FACTORY-MUD-001

## Mission

Finish the **Mech Factory MUD** end to end.

MUD means:

```text
Manufacturing Underlay Digital-twin
```

This is not a text adventure.

This is not a UE4 level.

This is not a visual demo.

This is the **headless Rust digital twin** of the mech factory walkthrough that must align perfectly with the future UE4 projection surface.

The Mech Factory MUD must prove:

```text
POWL / process law
→ ggen-manufactured Rust/C++ artifacts
→ Rust digital twin factory simulation
→ deterministic walkthrough route
→ station-by-station mech manufacturing
→ authority byte-state transitions
→ geometry/motion/skin/projection surrogates
→ OCEL-style event log
→ tamper-evident receipt chain
→ UE4-aligned projection package
→ verifier report
```

before UE4 is allowed to claim anything.

The current focus is the **C++ and Rust side of ggen**.

Do not stop at C++ headers.

Do not stop at generated Rust files.

Do not stop at passing unit tests.

Do not stop at “ggen sync succeeded.”

Do not stop at “digital twin scaffold exists.”

Finish the whole Rust factory-walkthrough MUD under declared scope.

---

# Milestone

```text
GC-MECH-FACTORY-MUD-001
```

Target status:

```text
PARTIAL_ALIVE_CANDIDATE
```

Scoped status goal:

```text
MECH_FACTORY_MUD_ALIVE_UNDER_SCOPE
```

Only claim the scoped status if every required gate passes.

---

# Core Definition

The Mech Factory MUD is the headless Rust world that answers:

```text
Can the factory walkthrough exist, operate, manufacture, validate, receipt, and project its state
before UE4 renders it?
```

It must model:

```text
factory topology
walkthrough route
stations
mech parts
authority byte fields
station processes
generated artifacts
motion/geometry/skin surrogates
projection commands
receipt chains
OCEL-style event logs
UE4-facing DataTable/header contracts
```

The MUD is the **truth rehearsal** for UE4.

UE4 later receives:

```text
projection rows
coordinates
station IDs
part IDs
LOD classes
motion family rows
skin/material rows
authority state classes
receipt/debug overlay data
```

from this MUD/ggen pipeline.

---

# Architectural Law

Preserve:

```text
Nuxt owns the application shell.
Supabase owns backend evidence substrate.
Rust/ggen/wasm4pm own authority and verification.
The Mech Factory MUD owns the headless factory-world digital twin.
UE4 owns visual embodiment later.
Playwright owns future visual actuation proof.
Receipts decide standing.
```

Do not collapse:

```text
generated file exists
≠ law admitted

Rust crate builds
≠ factory MUD alive

C++ header emitted
≠ UE4 alignment proven

unit tests pass
≠ end-to-end walkthrough proven

digital twin simulated
≠ visual projection admitted

UE4 later renders
≠ process conformance proven
```

---

# Critical ggen Command Discipline

Do not guess ggen commands.

Do not use outdated docs as authority.

Before invoking ggen, confirm the true current command surface by inspecting:

```text
~/ggen source
~/ggen CLI parser
~/ggen examples
~/ggen tests
~/ggen docs
```

If prior audit already proved `ggen sync` is the current command, verify it against source before relying on it.

Create or update:

```text
docs/GGEN_SOURCE_CAPABILITY_AUDIT.md
```

Required audit sections:

```text
CLI commands discovered from source
Examples discovered
Pack format discovered
Manifest format discovered
SPARQL support discovered
Tera support discovered
Output path behavior
Validation behavior
Known stale docs
Residual unknowns
Exact command used
```

No generation command may be run until this audit exists or is updated.

---

# Manufacturing Discipline

Final Rust/C++ artifacts must be manufactured through ggen.

Hand-written code is allowed only for:

```text
test harnesses
thin adapters
CLI wrapper
fixtures
report writers
documentation
```

Hand-written code is not allowed for:

```text
authority byte fields
transition tables
typestates
MUD station law
walkthrough topology law
projection command schema
refusal reason enums
semantic LOD law
motion surrogate law
geometry surrogate law
skin surrogate law
receipt schemas
UE4-facing enums/headers
```

If an agent hand-writes an algorithm that belongs to the model, stop the line:

```text
JIDOKA: MANUAL_AUTHORITY_CODE
```

Then route the algorithm back into:

```text
ontology
→ SPARQL query
→ Tera template
→ ggen output
→ tests
```

---

# Required Output Package

Produce a deterministic generated package at:

```text
~/rocket-craft/generated/mech_factory_mud/
```

Required generated artifacts:

```text
MechFactoryMudSteps.rs
MechFactoryMudSteps.h
MechFactoryMudAuthority.rs
MechFactoryMudAuthority.h
MechFactoryMudTypestates.rs
MechFactoryMudTypestates.h
MechFactoryMudFactoryTopology.rs
MechFactoryMudWalkthrough.rs
MechFactoryMudStations.rs
MechFactoryMudParts.rs
MechFactoryMudGeometrySurrogates.rs
MechFactoryMudMotionSurrogates.rs
MechFactoryMudSkinSurrogates.rs
MechFactoryMudProjectionCommands.rs
MechFactoryMudProjectionManifest.json
MechFactoryMudDataTableManifest.json
MechFactoryMudReceiptSchema.json
MechFactoryMudOcelSchema.json
MechFactoryMudVerifierInput.json
MechFactoryMudHashes.json
```

Exact filenames may follow repo convention, but the verifier report must map them.

Every generated artifact must include provenance:

```text
generated_by
ggen_pack
source_ontology
source_sparql
source_template
generated_at_commit
content_hash
```

No orphan artifact is allowed.

---

# Required Rust Crate

Create or complete a Rust crate:

```text
crates/mech_factory_mud
```

or, if workspace convention demands another path, use that and document it.

This crate must compile and run as a headless factory digital twin.

Required modules:

```text
src/lib.rs
src/main.rs or src/bin/mech-factory-mud.rs
src/world.rs
src/factory.rs
src/walkthrough.rs
src/stations.rs
src/parts.rs
src/authority.rs
src/transitions.rs
src/geometry.rs
src/motion.rs
src/skin.rs
src/projection.rs
src/receipt.rs
src/ocel.rs
src/replay.rs
src/verifier.rs
src/report.rs
```

But generated law modules must be included from ggen output where possible.

The crate must expose a CLI:

```bash
cargo run -p mech_factory_mud -- verify
cargo run -p mech_factory_mud -- simulate
cargo run -p mech_factory_mud -- replay
cargo run -p mech_factory_mud -- export-ue4
cargo run -p mech_factory_mud -- report
```

If CLI names differ, document exact equivalents.

---

# MUD World Model

The MUD must instantiate a complete factory walkthrough.

## Required Route

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

Required law:

```text
route is connected
all nodes reachable
coordinates deterministic
each route node maps to a station or projection marker
each route node has Semantic LOD focus
each route node can emit projection commands
each route transition emits receipt event
```

## Required Stations

```text
FrameAssembly
SocketTopology
ArmorSkinStation
RigMotionStation
VerificationGate
ReceiptTerminal
```

Each station must have:

```text
station_id
station_kind
coordinates
entry_event
exit_event
required_inputs
produced_outputs
authority_effects
projection_rows
receipt_events
refusal_conditions
```

## Required Part Families

```text
Frame
Shoulder
Arm
Leg
Socket
ArmorPanel
WeaponMount
CoolingVent
SkinLayer
MotionFamily
LODVariant
ReceiptTerminal
```

Each generated part must answer:

```text
why it exists
which station created it
which process step created it
which authority state drives it
which projection row consumes it
which receipt proves it
```

---

# Authority Byte Fields

The digital twin must use dense byte authority state.

Required fields:

```text
damage_class: u8
heat_class: u8
stress_class: u8
grip_class: u8
socket_health_class: u8
lod_class: u8
walkthrough_state_class: u8
station_state_class: u8
projection_state_class: u8
receipt_state_class: u8
```

Required layout:

```text
SoA preferred.
Flat arrays preferred.
Contiguous buffers preferred.
Branchless LUTs/bitmasks preferred.
No vtable dispatch in hot path.
No String-heavy authority state in hot path.
No iter().position() in hot path for precedence checks.
```

Required generated kernels:

```text
walkthrough_state + input_event → next_walkthrough_state
station_state + part_state → next_station_state
heat + stress + socket_health → failure_risk
damage + process_relevance → semantic_lod
station + walkthrough_focus → projection_priority
```

Required equivalence:

```text
scalar_reference == generated_table == SIMD/SIMDe path
```

where SIMD/SIMDe path exists.

---

# Branchless / SIMD Requirements

If SIMDe or Rust SIMD exists in current workspace, integrate the smallest verified path.

Minimum required SIMD-equivalence proof:

```text
heat[i], stress[i], socket_health[i] → failure_risk[i]
```

Tests must cover:

```text
empty buffers
single element
length smaller than lane width
length equal to lane width
length not divisible by lane width
large buffer
max class values
invalid class refusal
scalar/SIMD divergence
```

If SIMD is not feasible in this milestone, publish residual:

```text
RESIDUAL: SIMD_PATH_NOT_ADMITTED
```

But the scalar and generated table paths must still pass.

---

# Walkthrough Simulation

The MUD must simulate the full walkthrough end to end.

Required command:

```bash
cargo run -p mech_factory_mud -- simulate --scenario factory_walkthrough
```

Required output:

```text
factory_walkthrough.trace.json
factory_walkthrough.ocel.json
factory_walkthrough.receipts.jsonl
factory_walkthrough.projection_manifest.json
factory_walkthrough.ue4_rows/
factory_walkthrough.report.json
factory_walkthrough.report.md
```

Required simulation events:

```text
EnterFactory
VisitFrameAssembly
GenerateFrame
VisitSocketTopology
GenerateSocketTopology
VisitArmorSkinStation
GenerateArmorPanels
GenerateSkinLayers
VisitRigMotionStation
GenerateMotionFamily
ValidateMotionClearance
VisitVerificationGate
RunFactoryVerification
VisitReceiptTerminal
EmitFactoryReceipt
ExitOrLoop
```

Required variants:

```text
happy_path
refused_missing_socket
refused_blocked_clearance
refused_skin_hides_vent
repair_then_admit
```

---

# Geometry Surrogate

No UE4 required.

Represent geometry as deterministic metadata.

Required structures:

```text
AABB bounds
socket mounts
clearance zones
station coordinates
walkthrough path coordinates
required semantic features
LOD preservation features
```

Required checks:

```text
weapon mount requires socket
armor cannot block motion clearance
vent cannot be hidden by skin
walkthrough path cannot cross blocked geometry
CROWN features survive LOD
projection rows map to geometry IDs
```

---

# Motion Surrogate

No animation clips required.

Represent motion as phase law.

Required motion families:

```text
Walk
Turn
Inspect
Assemble
Brace
FireWeapon
Repair
Recover
FactoryWalkthrough
```

Required checks:

```text
Inspect before Certify
Repair before Revalidate
PlantFeet before FireWeapon
motion cannot require missing socket
damaged leg changes gait class
walkthrough movement advances route state
motion rows map to UE4 projection commands
```

---

# Skin / Material Surrogate

Skins are semantic projection, not paint.

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
thermal zone binds to heat authority
damage mask binds to damage authority
repair residue binds to repair receipt
sponsor livery cannot hide vent
LOD texture preserves CROWN/PRIMARY features
skin rows map to UE4 material projection commands
```

---

# UE4 Alignment Contract

The MUD must export a UE4-aligned package.

Do not build UE4 in this milestone unless already trivial.

But the MUD must produce all rows UE4 will need.

Required export:

```bash
cargo run -p mech_factory_mud -- export-ue4 --out generated/mech_factory_mud/ue4
```

Required UE4-aligned outputs:

```text
DataTables/FactoryStations.csv
DataTables/WalkthroughRoute.csv
DataTables/PartFamilies.csv
DataTables/SocketTopology.csv
DataTables/SkinLayers.csv
DataTables/MotionFamilies.csv
DataTables/SemanticLOD.csv
DataTables/ProjectionCommands.csv
Headers/MechFactoryMudSteps.h
Headers/MechFactoryMudAuthority.h
Headers/MechFactoryMudProjection.h
ProjectionManifest.json
ReceiptManifest.json
```

Required fields for every UE4 projection row:

```text
projection_id
object_id
station_id
route_node_id
source_process_step
source_receipt
authority_inputs
lod_class
projection_type
ue4_target_surface
admission_status
```

No projection row without source receipt.

No CROWN row without authority reason.

No station without coordinates.

No route node without projection marker.

---

# OCEL / Receipt Requirements

Emit object-centric evidence.

Minimum object types:

```text
Factory
Station
WalkthroughRun
Mech
Frame
Socket
ArmorPanel
SkinLayer
MotionFamily
GeometryEnvelope
ProjectionRow
Receipt
VerifierGate
```

Minimum event types:

```text
RouteNodeEntered
StationVisited
PartGenerated
AuthorityUpdated
SurrogateValidated
ProjectionRowEmitted
VerifierGatePassed
VerifierGateRefused
ReceiptEmitted
RepairApplied
```

Receipt chain must be tamper-evident.

Required receipt fields:

```json
{
  "sequence": 1,
  "event_type": "...",
  "surface": "mech_factory_mud",
  "objects": ["..."],
  "input_hash": "...",
  "output_hash": "...",
  "prev_hash": "...",
  "receipt": "...",
  "status": "ADMITTED|REFUSED|RESIDUAL",
  "residuals": []
}
```

Do not say unforgeable.

Correct phrase:

```text
tamper-evident receipt chain
```

---

# Required Tests

Follow the ladder:

```text
unit
→ integration
→ e2e
→ chaos
→ stress
→ benchmark
→ verifier report
```

## Unit Tests

Required:

```text
authority byte range checks
branchless transition table checks
walkthrough topology checks
station input/output checks
geometry surrogate checks
motion surrogate checks
skin surrogate checks
projection row checks
receipt chain checks
OCEL object/event checks
```

## Integration Tests

Required:

```text
ggen output → Rust crate compile
generated authority → MUD world load
MUD simulation → receipts
MUD simulation → projection manifest
projection manifest → UE4 CSV/header export
receipt chain → replay verifier
```

## E2E Tests

Required:

```text
factory_walkthrough happy_path
factory_walkthrough refused_missing_socket
factory_walkthrough refused_blocked_clearance
factory_walkthrough refused_skin_hides_vent
factory_walkthrough repair_then_admit
```

## Chaos Tests

Required mutations:

```text
remove route node
break route edge
delete station coordinate
weapon mount without socket
skin hides vent
motion requires missing socket
projection row lacks receipt
receipt prev_hash broken
receipt event mutated
generated header disagrees with CSV
authority byte out of range
LOD demotes CROWN without reason
```

Each must fail for expected reason.

## Stress Tests

Minimum:

```text
1 factory
10 stations
100 parts
1,000 projection rows
10,000 authority cells
```

If feasible:

```text
100,000 authority cells
```

Benchmark:

```text
walkthrough step transition
station process execution
authority update
projection manifest validation
receipt replay
UE4 export generation
```

Do not overclaim performance.

Report machine, compiler, target, sample size, and residuals.

---

# Required Reports

Generate:

```text
~/rocket-craft/VERIFIER_REPORT_GC_MECH_FACTORY_MUD_001.md
~/rocket-craft/VERIFIER_REPORT_GC_MECH_FACTORY_MUD_001.json
```

Required sections:

```text
Milestone
Scope
Standing Inherited
ggen Source Capability Audit
Repository Boundaries
Generated Artifacts
Rust Crate
Factory World Model
Walkthrough Route
Stations
Authority Byte Fields
Branchless Transition Law
SIMD/SIMDe Equivalence
Geometry Surrogate
Motion Surrogate
Skin Surrogate
Projection Manifest
UE4 Alignment Contract
OCEL Events
Receipt Chain
Tests
Chaos Tests
Stress / Benchmarks
Agent Jidoka Events
Residuals
Next Falsifier
Final Status
```

---

# Agent Jidoka Must Fire If

```text
agent guesses ggen command
agent relies on stale docs without source audit
agent writes final authority Rust by hand
agent writes final C++ headers by hand
ggen emits branchy hot-path code
ggen emits String/Vec-heavy hot-path authority code
ggen emits iter().position() precedence checks
generated artifact lacks provenance
generated artifact lacks hash
projection row lacks receipt
route node unreachable
station has no coordinate
weapon mount lacks socket
skin hides vent
motion requires missing part
receipt chain accepts mutation
header and CSV disagree
UE4 alignment fields missing
tests pass without chaos tests
report omits residuals
```

Every Jidoka event must include:

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

# Required Final Status Logic

Set:

```text
MECH_FACTORY_MUD_ALIVE_UNDER_SCOPE
```

only if all are true:

```text
ggen source audit complete
true ggen command used
generated artifacts emitted
generated artifacts have provenance
Rust MUD crate compiles
Rust MUD simulation runs happy path
refused scenarios refuse correctly
repair scenario admits after repair
walk walkthrough route validates
all stations validate
authority transitions validate
geometry surrogate validates
motion surrogate validates
skin surrogate validates
projection manifest validates
UE4 export package exists
OCEL log emitted
receipt chain validates
mutation breaks receipt verification
chaos tests fail correctly
stress tests complete
benchmark report emitted
markdown and JSON verifier reports emitted
residuals published
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

After this milestone:

```text
GC-MECH-FACTORY-MUD-002:
MUD_TO_NUXT_SUPABASE_UE4_BRIDGE
```

That next milestone must prove:

```text
MUD simulation
→ Supabase stored session/receipts
→ Nuxt UI receipt drawer
→ UE4 bridge receives projection commands
→ Playwright observes visual delta
→ evidence uploaded
→ receipt chain replay
```

Do not start 002 until 001 emits reports and residuals.

---

# Final Agent Response Format

Respond only with:

```text
Milestone:
Status:
Scoped status:
Commands run:
Files changed:
ggen command verified from:
Generated artifacts:
Rust crate path:
MUD simulation outputs:
UE4 alignment outputs:
Tests passed:
Tests failed:
Chaos tests:
Benchmarks:
OCEL files:
Receipt files:
Agent Jidoka events:
Residuals:
Next falsifier:
```
