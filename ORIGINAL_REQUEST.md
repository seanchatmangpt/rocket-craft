# Original User Request

## Initial Request — 2026-06-19T17:23:43-07:00

# Teamwork Project Prompt: Asset Manufacturing LSP (ggen-asset-lsp)

Goal: Implement the Asset Manufacturing LSP (`ggen-asset-lsp`) using `~/lsp-max`.
Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Mission
You are tasked with building `ggen-asset-lsp`, a 3D Asset Manufacturing Language Server. This is not a standard code LSP. It treats USD, MaterialX, textures, rigs, renders, and receipts as a living, diagnosable compiler surface. It uses the framework at `/Users/sac/lsp-max`.

## Requirements

### R1. Crate Initialization
Create `crates/ggen-asset-lsp` inside the `rocket-craft` workspace. It must depend on the LSP framework crates located at `/Users/sac/lsp-max`. 

### R2. USD/MaterialX Diagnostic Authority (GC-ASSET-LSP-001)
Implement the LSP server to index the directory `generated/mech_assets/reference_fabric_001/`. It must parse `.usda` and `.mtlx` files to detect missing payloads, missing material bindings, and unreceipted USD prims.

### R3. Visual Proof Routing
The LSP must parse headless render outputs (`visual_gap_report.json`) and `usdchecker` logs, projecting them as `PublishDiagnostics` errors directly onto the USDA text in the editor (e.g., highlighting `def Mesh` if the silhouette IOU falls below threshold).

### R4. Code Actions for Source Law
Provide LSP Code Actions that target the **generator parameter source** (the SPARQL query, Tera template, or Rust parameter row), NOT the generated USD output. Asset instances are immutable; the source law is what must be repaired.

### R5. OCEL Integration
Emit an OCEL event for the LSP diagnostic lifecycle whenever the LSP runs a validation pass or a repair action.

## Acceptance Criteria

### Implementation
- [ ] `crates/ggen-asset-lsp` compiles successfully against the local `~/lsp-max` path.
- [ ] The binary can launch and respond to `initialize` and `textDocument/didOpen` requests.

### Diagnostics & Routing
- [ ] A missing `payload = @mesh.usd@` in a parsed USDA file correctly triggers an LSP Diagnostic.
- [ ] Failures in `visual_gap_report.json` are successfully mapped to diagnostics on the root `Xform` or `Mesh` in the USDA.

## Follow-up — 2026-06-20T00:46:45Z

EMERGENCY CORRECTION: The asset you verified was a false-positive admitted by a weak whole-image metric. The pipeline is now moving to **GC-MECH-ASSET-FABRIC-001B** (Part-Aware Morphology Convergence).

Update your diagnostic engine to support the new `VIS200` series taxonomy for morphology failures.
Do NOT use franchise-specific language.

New diagnostics to implement:
- VIS201 ERROR: part-graph similarity below threshold.
- VIS202 ERROR: wing morphology mismatch.
- VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates.
- VIS204 ERROR: core body massing exceeds compactness bound.
- VIS205 ERROR: blade placement/angle mismatch.
- VIS206 ERROR: armor segmentation density below threshold.
- VIS207 ERROR: edge-density distribution mismatch.
- VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate.

See `/Users/sac/rocket-craft/.agents/SPR_MORPHOLOGY_CONVERGENCE.md` for the full spec. The LSP must surface these errors when the new gap report includes per-component morphology residuals.

## Follow-up — 2026-06-20T00:49:38Z

EMERGENCY CORRECTION: The USD output you are diagnosing failed modularity constraints. Every file was a duplicate of the full assembly. 

Add the following `USD300` diagnostics to the Asset LSP immediately:
- USD301 ERROR: duplicate USD geometry fingerprint.
- USD302 ERROR: part file renders full assembly.
- USD303 ERROR: part-local file contains foreign component prims.
- USD304 ERROR: expected part root missing.
- USD305 ERROR: mirrored part lacks mirror transform proof.
- USD306 ERROR: generated USD files share identical source template expansion.
- USD307 ERROR: part bounding box overlaps full-asset bounds.

See `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`. The LSP must flag these if multiple `.usda` files share the exact same primitive composition or if `SM_Head.usda` contains a Torso mesh.

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

## 2026-06-20T23:55:45Z

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

