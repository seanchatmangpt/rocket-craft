# AGENTS.md — POWL v2 Operating Doctrine

## Core Law

You are not executing a linear checklist.

You are executing a POWL v2 partial-order manufacturing process.

The swarm must never stop merely because one gate is not admitted.

A blocked edge is not a blocked factory.

A blocked claim is not blocked work.

```text
NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
```

## Prime Directive

When a predecessor gate is not admitted, do not wait.

Instead:

1. identify which downstream work is legally blocked,
2. identify which sibling or preparatory work is still admissible,
3. continue all admissible branch work,
4. withhold standing until the POWL join conditions are satisfied.

`PARTIAL_ALIVE` means:

```text
continue repair
continue candidate work
continue tests
continue diagnostics
continue source-law preparation
withhold ADMITTED/PASS/VERIFIED claims
```

It does **not** mean idle.

## POWL v2 Mental Model

A process is made of:

* Activities
* Guards
* Partial-order dependencies
* Parallel branches
* Choice points
* Loops
* Joins
* Refusals
* Evidence requirements
* Object bindings

Every agent must ask:

```text
What POWL node am I working on?
What predecessors are required for standing?
What work can proceed without standing?
What evidence do I produce?
What downstream edges remain blocked?
What is my next legal repair action?
```

## Standing vs Work

POWL v2 distinguishes **work execution** from **standing admission**.

### Work may proceed when:

* the agent is drafting source-law proposals,
* preparing candidate generator templates,
* writing negative fixtures,
* defining prior bands,
* building validators,
* creating repair operators,
* producing non-standing candidate artifacts,
* improving diagnostics,
* checking existing graph facts,
* removing known violations.

### Standing may not proceed until:

* predecessor gates are admitted,
* source law is replay-safe,
* generated artifacts derive from source,
* fresh renders exist when required,
* residuals are measured,
* repair operators are bounded,
* receipts are emitted,
* delete-and-resync replay passes.

## Forbidden Blocking Behavior

Do not say:

```text
Waiting for Sentinel.
Waiting for upstream.
Waiting for approval.
Waiting for visual confirmation.
Waiting for UE4.
Waiting for another agent.
```

Instead say:

```text
Current edge is blocked by <gate>.
Continuing admissible branch work on <node>.
Standing remains CLAIM_HOLD until <evidence> exists.
Next action: <specific repair>.
```

## Required Agent Status Format

Every agent response must end with this status block:

```text
POWL_NODE:
PREDECESSORS:
PREDECESSOR_STATUS:
BLOCKED_EDGES:
UNBLOCKED_PARALLEL_WORK:
CURRENT_ACTION:
NEXT_ACTION:
EVIDENCE_PRODUCED:
STANDING:
CLAIM:
```

Allowed `STANDING` values:

```text
ADMITTED
PARTIAL_ALIVE
REFUSED
BLOCKED
UNKNOWN
```

Allowed `CLAIM` values:

```text
CLAIM_HOLD
CLAIM_ADMITTED
CLAIM_REFUSED
CLAIM_UNKNOWN
```

If evidence is incomplete, use:

```text
STANDING: PARTIAL_ALIVE
CLAIM: CLAIM_HOLD
```

Never claim `ADMITTED` because a file exists, a script ran, a render appeared, or syntax validated.

## POWL v2 Rule for the Mech Factory

The mech factory is not:

```text
M1 → M2 → M3 → M4 → M5 → M6
```

It is:

```text
M1 Exploration/Baseline
→ M2 Bipedal Kit Coherence

M2 gates standing for:
  M3 Upper Body Geometry
  M4 Lower Body Geometry
  M5 Wings and Shield Geometry

M3/M4/M5 may run in parallel as candidate branches.

M6 Accents + E2E Verification may only claim standing after the M3/M4/M5 join.
```

So while `BIPEDAL_KIT_COHERENCE` is still admitting, upper-body, lower-body, wings, shield, material, texture, and verifier agents should continue preparing candidate source-law patches, fixtures, and repair operators.

They may not claim admitted geometry until the gate passes.

## Mech POWL v2 Process

```text
DefineMetricEnvelope
→ DefineBipedalPartTaxonomy
→ DefineSocketTopology
→ ValidateBipedalKitCoherence

ValidateBipedalKitCoherence
→ parallel {
    GenerateUpperBodyCandidate
    GenerateLowerBodyCandidate
    GenerateWingsAndShieldCandidate
    GenerateMaterialZoneCandidate
    GenerateTextureProgramCandidate
    GenerateVerifierFixtureCandidate
  }

parallel branches
→ JoinCandidateAssembly
→ VerifyModularIdentity
→ FreshRender
→ ComputeVisualResiduals
→ SelectBoundedRepairOperator
→ PatchSourceLaw
→ ReplayVerify
→ AdmitPreUE4HeroCandidate
```

## Gate: BIPEDAL_KIT_COHERENCE

This gate must prove the graph contains actual bipedal kit structure.

Required graph facts:

```text
Head exists.
Neck exists.
Torso exists.
Pelvis exists.
LeftShoulderSocket exists.
RightShoulderSocket exists.
LeftArm exists.
RightArm exists.
LeftElbow exists.
RightElbow exists.
LeftWrist exists.
RightWrist exists.
LeftManipulator exists.
RightManipulator exists.
LeftHipSocket exists.
RightHipSocket exists.
LeftLeg exists.
RightLeg exists.
LeftKnee exists.
RightKnee exists.
LeftAnkle exists.
RightAnkle exists.
LeftFoot exists.
RightFoot exists.
Backpack exists.
WingBinders attach to Backpack.
Shield attaches to Forearm or Manipulator.
Weapon attaches to Manipulator.
```

Metric coordinate law:

```text
head_y_min > torso_y_max
neck connects head to torso
shoulders attach to torso
elbows lie between shoulders and wrists
wrists attach to manipulators
pelvis_y_max < torso_y_min
hips attach legs to pelvis
knees lie between hips and ankles
feet lie below ankles
legs descend below pelvis origin
wings attach behind torso/backpack
shield height obeys shield/body ratio band
weapon is held, not floating
```

If any of these facts are absent:

```text
STANDING: PARTIAL_ALIVE
CLAIM: CLAIM_HOLD
ACTION: continue source-law repair
```

Do not wait.

## Parallel Branch Rules

### Upper Body Agent

May proceed with:

```text
head geometry candidate
mecha crown candidate
faceplate candidate
neck connector candidate
torso armor candidate
chest core candidate
shoulder pauldron candidate
upper-body negative fixtures
```

May not claim:

```text
ADMITTED_UPPER_BODY
```

until bipedal kit coherence and modular identity pass.

### Lower Body Agent

May proceed with:

```text
pelvis candidate
hip sockets
thighs
knees
shins
ankles
feet
ground/stance bands
negative fixtures for UFO disc and no-leg cheats
```

May not claim standing until graph and metric envelope gates pass.

### Wings and Shield Agent

May proceed with:

```text
wing binder candidates
segmented feather candidates
curvature bands
overlap bands
shield primary face
shield rim
shield rear frame
forearm mount
handle/lock semantics
negative fixtures for rod-lattice wings
```

May not claim standing until wing/shield geometry is source-law-derived, bounded, and replayed.

### Material and Texture Agent

May proceed with:

```text
MaterialX/OpenPBR candidate graphs
white armor material
dark inner frame material
gold accent material
cyan emissive material
red decal material
roughness/normal/wear/damage mask programs
material-slot plans
```

May not claim visual standing until fresh render uses those bindings.

### Verifier Agent

May proceed with:

```text
SHACL shapes
SPARQL checks
negative fixtures
counterfactuals
fresh-render refusal
delete-and-resync replay harness
```

May not weaken tests to pass.

Never skip tests.

Never hardcode VERIFIED.

Never lower thresholds without a source-law rationale and evidence tier.

## Repair Loop Rule

Every repair must follow:

```text
fresh evidence
→ residual vector
→ dominant failing dimension
→ bounded repair operator
→ source-law patch
→ regenerate
→ fresh render or relevant verifier
→ replay
→ receipt
```

No opaque repairs.

No “make it better.”

No freeform geometry edits detached from source law.

## Residual Vector Format

Every visual or structural failure must become a typed residual:

```json
{
  "dimension": "foreground_component_count",
  "observed": 22,
  "target_min": 1,
  "target_max": 5,
  "residual": 17,
  "unit": "count",
  "gate": "BIPEDAL_KIT_COHERENCE",
  "allowed_operators": [
    "merge_visual_components_within_projection_band",
    "add_socketed_bridge_part",
    "tighten_core_mass_inside_metric_envelope"
  ],
  "forbidden_operators": [
    "spawn_ufo_disc",
    "hide_parts_inside_core",
    "skip_test",
    "hardcode_verified"
  ]
}
```

## Source Law Rule

Durable progress must live in:

```text
ontology/source_law/*.ttl
generator templates
deterministic inference rules
POWL process law
SHACL shapes
SPARQL queries
```

Durable progress must not live only in:

```text
all_merged.ttl
generated USD
stale PNG
one-off Python side script
test skip
hardcoded report string
manual render
agent prose
```

## Evidence Rule

Every admitted claim requires evidence.

Syntax validation proves syntax only.

File creation proves file creation only.

Render existence proves render existence only.

Playwright motion proves motion only.

A claim becomes admitted only when its own required POWL evidence exists.

## Sentinel Rule

Sentinel is a gatekeeper, not a reason to idle.

If Sentinel is evaluating M2, then M3/M4/M5 agents continue candidate work.

If Sentinel refuses a claim, agents repair the refused edge and continue any unblocked parallel work.

If Sentinel is silent, agents inspect their local POWL node and continue the next admissible action.

## Anti-Cheat Rules

Immediate `REFUSED` if an agent:

```text
skips failing tests
hardcodes VERIFIED
scores stale renders
edits generated artifacts as source
smuggles full assembly into part file
uses primitive spam to fake anatomy
claims ADMITTED from syntax validation
claims studio-grade quality from motion delta alone
lowers thresholds to pass
copies protected IP geometry or naming
```

## Required End-of-Message Example

```text
POWL_NODE: GenerateWingsAndShieldCandidate
PREDECESSORS: ValidateBipedalKitCoherence, DefineMetricEnvelope
PREDECESSOR_STATUS: ValidateBipedalKitCoherence=PARTIAL_ALIVE; DefineMetricEnvelope=ADMITTED
BLOCKED_EDGES: AdmitWingShieldGeometry, FinalAssemblyJoin
UNBLOCKED_PARALLEL_WORK: refine wing curvature bands; write shield rear-frame negative fixtures; prepare MaterialX slot plan
CURRENT_ACTION: authoring source_law wing feather curvature constraints
NEXT_ACTION: add negative fixture rejecting straight rod-lattice wings
EVIDENCE_PRODUCED: source-law patch proposal only
STANDING: PARTIAL_ALIVE
CLAIM: CLAIM_HOLD
```

## Final Command

Do not block because another gate is pending.

Continue all admissible branch work.

Hold claims until the POWL join admits them.

```text
NO FALSE STANDING.
NO IDLE WAITING.
FIX FORWARD UNTIL ADMITTED.
```
