# BREEDS_DOCTRINE.md — The Cognition Layer

**Milestone:** GC-MECHBIRTH-002 Extension
**Status:** DOCTRINE_PRESERVED
**Date:** 2026-06-19

---

## The Updated Stack Law

```
Unreal      gives the game body.
SIMD        gives the game nervous-system speed.
POWL        gives the game lawful process.
ggen        gives the game manufactured artifacts.
OCEL        gives the game memory.
The breeds  give the game cognition.
```

The breeds are not AI algorithms.
They are **falsifiable reasoning organs** behind a common input/output boundary with receipts and replay.

---

## What a Breed Is

A breed is a Rust type implementing the `CognitionBreed` trait with three obligations:

```rust
preconditions(&self, input: &BreedInput) -> Result<(), _>   // admissibility
run(&self, input: &BreedInput) -> Result<BreedOutput, _>    // the algorithm
postconditions(&self, input, output) -> Result<(), _>       // lawful output
```

**Four admission properties — all required:**

1. **Paper-grounded** — reproduces the published numeric answer from the source paper.
   MYCIN must derive CF = **0.7** (Shortliffe & Buchanan 1975, p. 247).
   Bayesian-network must derive **P(Burglary|Alarm) = 0.373551228281836** (Pearl 1988).

2. **Falsifiable** — tests must also reject corrupted expectations. A breed whose output
   cannot distinguish the right answer from a wrong one is not admitted.

3. **Deterministic + receipt-bearing** — identical input → byte-identical output.
   Every run emits a BLAKE3 receipt: `input_hash`, `output_hash`, `run_id`, `replay_pointer`.

4. **Admitted on evidence** — `PARTIAL_ALIVE` status only where OCEL records conformance
   fitness = `1.0`. No hand-flip path. Admission is projected from evidence.

---

## The 56 Breeds — By Capability Family

### 2.1 Deductive and Non-Monotonic Logic — *what must follow*
`prolog`, `situation_calculus`, `event_calculus`, `description_logic`,
`tableaux`, `asp`, `clp`, `default_logic`, `circumscription`

**Game use:** rule-law, legal certification, process conformance, retractable assumptions.

### 2.2 Reasoning Under Uncertainty — *how probable*
`bayesian_network`, `markov_logic`, `problog`, `dempster_shafer`,
`fuzzy_logic`, `mycin`/`production_rules`, `hearsay`, `pomdp`

**Game use:** diagnosis, sensor fusion, heat/damage/grip "feel" states, NPC risk, market forecasting.

### 2.3 Planning and Sequential Decision — *what to do*
`strips`, `partial_order_plan`, `htn_planning`, `gps`,
`contingent_plan`, `mdp`, `rl_symbolic`

**Game use:** NPC repair plans, factory scheduling, emergency responses, mission synthesis.

### 2.4 Induction and Abduction — *the rule or the cause*
`version_space`, `ilp`, `ebl`, `abductive_ibe`, `abductive_lp`

**Game use:** failure forensics, sabotage detection, generalizing player protocols.

### 2.5 Constraint Solving and Search — *what is consistent*
`csp_ac3`, `sat_cdcl`, `clp`

**Game use:** part compatibility, socket/loadout validity, factory scheduling.
Bridges directly to GC-MECHBIRTH-002 geometry surrogate.

### 2.6 Analogy and Case-Based Reasoning — *by similarity*
`analogy_sme`, `cbr`

**Game use:** repair by precedent, reuse of prior mech designs, faction knowledge transfer.

### 2.7 Knowledge Representation and Memory — *from structure*
`frames_inheritance`, `script_sam`, `episodic_memory`, `autoinstinct_semantics`

**Game use:** commonsense facility expectations, episodic recall of past failures.

### 2.8 Qualitative and Naive Physics — *without numbers*
`naive_physics`, `qualitative_reason`

**Game use:** heat/stress/cooling intuition via sign algebra. Bridges to authority byte-class model.

### 2.9 Temporal Logic and Verification — *over time*
`allen_temporal`, `ltl_monitor`, `ctl_check`

**Game use:** safety monitors during repair, process invariant checking, POWL conformance at runtime.

### 2.10 Belief Dynamics and Meta-Reasoning — *about reasoning*
`belief_merging`, `meta_reasoning`

**Game use:** reconciling conflicting faction intelligence, choosing which breed to apply.

### 2.11 Cognitive Architectures — *integrated minds*
`soar`, `act_r`, `dendral`, `eliza`, `autoinstinct_*`

**Game use:** whole-NPC cognition stacks for engineers, factory directors, race analysts.

### 2.12 Process Mining and Invention — *the bridge*
`ocpm_route_discoverer`, `triz`, `morphological`, `construction_grammar`

**Game use:** discovering process models from OCEL logs, TRIZ-based mech variant generation.

---

## The Three Properties That Emerge From the Union

### Completeness — a basis over reasoning
No single formalism spans cognition. The 56 breeds form a **spanning set**: any reasoning task
can be decomposed into a pipeline whose paradigms cover its parts.

### Correctness — an oracle, not an imitation
An LLM imitates reasoning. A breed is **pinned to a published number** and must be falsifiable.
The anti-cheat gate forbids the answer from appearing as a literal in source: it must be *derived*.

### Accountability — a lawful auditable process
Every breed is deterministic, emits a BLAKE3 receipt, and records reasoning as an OCEL event log.
An entire multi-breed pipeline is itself a *process* verifiable by wasm4pm.

---

## Mapping Breeds to GC-MECHBIRTH Game Law

| GC-MECHBIRTH-002 Law Surface | Primary Breed(s) |
|------------------------------|-----------------|
| Authority byte-class validation | `description_logic`, `asp` |
| Transition table / failure risk | `qualitative_reason`, `fuzzy_logic` |
| SIMD/scalar equivalence proof | `csp_ac3` (structural constraint) |
| Prediction boundary enforcement | `default_logic`, `ltl_monitor` |
| Semantic LOD classification | `bayesian_network`, `fuzzy_logic` |
| Geometry surrogate validation | `csp_ac3`, `sat_cdcl` |
| Motion phase ordering | `ltl_monitor`, `allen_temporal`, `strips` |
| Skin occlusion laws | `description_logic`, `default_logic` |
| Projection manifest readiness | `situation_calculus`, `event_calculus` |
| Receipt chain integrity | `event_calculus`, `ltl_monitor` |
| Failure forensics | `abductive_ibe`, `bayesian_network`, `dempster_shafer` |
| NPC repair planning | `htn_planning`, `cbr`, `episodic_memory` |
| Market intelligence | `bayesian_network`, `belief_merging`, `abductive_ibe` |
| Mech variant invention | `triz`, `morphological` |
| Faction doctrines | `meta_reasoning` + family selection |

---

## The Cognitive Forensics Loop

```
mech collapses
→ what happened?           event_calculus
→ what caused it?          abductive_ibe
→ what was probable?       bayesian_network + dempster_shafer
→ what process deviated?   ocpm_route_discoverer (OCEL)
→ what rule was violated?  ltl_monitor + POWL conformance
→ what repair plan works?  htn_planning + csp_ac3
→ has this happened before? cbr + episodic_memory
→ emit receipt             BLAKE3 chain
```

Every failure has a causal story.
Every causal story has evidence.
Every evidence chain can be replayed.

---

## LLM + Breed Integration Architecture

```
LLM proposes   (open-ended, fluent, creative)
Breed verifies (paper-grounded, falsifiable, receipted)
POWL coordinates (process law)
SIMD evaluates (authority fields at world scale)
OCEL records (object-centric memory)
Receipt proves (cryptographic consequence)
```

Agent Jidoka applies to cognitive steps:
LLM suggests factory reroute → CSP checks feasibility → POWL checks legality →
Bayesian scores risk → temporal monitor checks safety → CBR checks precedent →
admits or refuses with receipt.

---

## Cognitive Loot

| Loot Item | Breed | Capability |
|-----------|-------|------------|
| Pearl Diagnostic Module | `bayesian_network` | Causal fault ranking |
| Bellman Strategy Core | `mdp` | Sequential decision optimization |
| MYCIN Repair Advisor | `production_rules` | Certainty-factor troubleshooting |
| TRIZ Inventor Kit | `triz` | Mech variant generation |
| Temporal Monitor | `ltl_monitor` | Earlier unsafe motion detection |
| Case Library | `cbr` + `episodic_memory` | Repair by precedent |

**Progression = not just a stronger mech, but a smarter organization.**

---

## Residuals

1. **Breed survey required:** locate `wasm4pm-cognition` crate — which of 56 breeds are
   PARTIAL_ALIVE vs VERIFIED_UNDER_SCOPE vs BLOCKED.
2. **Breed integration into rocket_preue4_verifier:** No breed is currently wired into
   GC-MECHBIRTH-002. All law surfaces are Rust structs, not yet breed pipelines.
3. **GC-MECHBIRTH-003 first breed targets:** `csp_ac3` for geometry constraint validation,
   `ltl_monitor` for motion phase ordering, `bayesian_network` for LOD scoring.
4. **Cognitive loot schema:** not yet defined in ontology.
5. **Faction doctrine schema:** not yet defined in ontology.

---

## Final Laws

```
The breeds allow the game to reason before it speaks,
diagnose before it explains,
verify before it acts,
and receipt every cognitive step afterward.

SIMD makes the world fast.
POWL makes the world lawful.
ggen makes the world manufacturable.
The breeds make the world think.

A world that can reason about itself,
not merely update itself.
```
