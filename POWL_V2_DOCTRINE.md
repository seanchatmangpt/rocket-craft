# POWL v2 Doctrine: Playable Process Grammar

POWL v2 is not a business-process diagram.
**POWL v2 is the worldline grammar.**

It tells the game what can happen in parallel, what must happen before something else, what can be skipped, what can loop, what can branch, what can be improvised, what became a deviation, and what became a valid discovered variant.

POWL v2 makes the order of action playable. It is how Rocket-Craft stops being a scripted game and becomes a lawful world.

---

## 1. POWL v2 as Playable Process Law
A lot of workflow models are too sequence-shaped (A → B → C → D). Real gameplay is partially ordered:
- A must happen before D (A < D)
- B and C can happen in either order (B ∥ C)
- F can loop until inspection passes (F loops until admitted)
- G and H are alternative decision paths (G xor H)

**POWL v2 = partial-order mission grammar + choice-graph decision law + loop law + OCEL event binding + semantic LOD projection + receipt/replay discipline.**

## 2. Lawful Nonlinearity
Old mission design: Designer writes path. Player follows path. Deviation is impossible or a bug.
POWL v2 design: Designer defines process law. Player discovers path. Deviation becomes evidence.
Successful deviation may become a new variant. Failed deviation becomes a replayable consequence.
This gives freedom without chaos.

## 3. Choice Graphs Become Real Decision Gameplay
Choices are not just dialogue options. Choices are process-shaping forks.
If socket inspection fails, you can: repair socket, replace socket, downgrade weapon, delay launch, accept risk with receipt.
These choices do not all rejoin neatly. They affect market, race eligibility, factory backlog, failure risk, and receipt confidence.

## 4. POWL v2 + OCEL = Multi-Object Causality
POWL models control flow. OCEL-style logs give object-centric event reality.
The game knows not just that you installed the socket, but whether you installed it lawfully, which objects inherited risk, and how that decision propagated into the race.

## 5. Process Grammars in the World
- **GMF Factory Grammar:** ReceiveOrder < AcquireParts < InspectParts... Paint ∥ CalibrateSystems. The player operates the process model.
- **Race Weekend Grammar:** Register < Qualify. InspectVehicle < Race. Pit may loop during Race.
- **Combat Grammar:** Aim < Fire < Cool. Player can skip Stabilize or fire during overheat. System classifies as unsafe shot, overheat deviation, or valid fast shot. Process-based combat skill.
- **Repair Gameplay:** Disassemble < Replace < Reassemble. CleanContacts ∥ ReplaceCoolant. Good players repair processes, not health bars.

## 6. Deviation Magic
The game classifies deviations: MISSING_STEP, EXTRA_STEP, WRONG_ORDER, EARLY_EXECUTION, EMERGENCY_EXCEPTION, INNOVATION_CANDIDATE, SABOTAGE_SIGNATURE, RISK_ACCEPTED.
Each deviation becomes playable. Missing a stress test gives a faster launch but no certification receipt.

## 7. Future Process Maps (POWL v2 + Prediction)
The game simulates the process forward to forecast bottlenecks and failure cones.
Shows the safe path, risky path, illegal path, and discovered player variant path. Tactical vision.

## 8. POWL v2 + Semantic LOD
Current process step determines Semantic LOD promotion.
A bolt is BACKGROUND normally. During DisassembleWeaponSocket, that bolt becomes PRIMARY. The world LODs by process relevance.

## 9. POWL v2 + ggen
POWL should not live as a runtime diagram manually interpreted by Blueprints.
POWL v2 model → SHACL admission → SPARQL extraction → ggen templates → generated typestates, transition tables, DataTables, and verifier surfaces.
Blueprint should not own process meaning. UE4 should project the process state.

## 10. Game Features Unlocked
- **Process Spellcasting:** Perfect trace = stable output. Risk trace = power boost + failure chance.
- **Factory Roguelike Runs:** Mine variants from event logs and improve the process between runs.
- **Playable Compliance:** Certified mech vs uncertified risky mech vs forged receipt.
- **Causal Failure Replay:** Replay the trace against the POWL model for a receipt-based autopsy.
- **Process PvP:** Attack the opponent's process (jam inspection, delay supplier batch) causing their process to collapse.
- **Crew Skill as Process Expansion:** Crew members modify process grammar (e.g., Veteran driver overlaps WarmTires and TuneVehicle).
- **Discovered Protocols:** Deviation → candidate variant → verification challenge → admitted protocol. Player invents new process law through play.

## 11. Process UI
Do not show it as a BPMN dashboard. Show it as a glowing tactical map, constellation, or sequence lattice.

---

## PART II: Procedural Manufacturing Law

**POWL coordinates the birth of the mech. SIMD governs its living state. Unreal projects its body. OCEL remembers its life.**

Procedural generation needs process law. Without POWL, procedural generation is random part soup. With POWL, procedural generation becomes a lawful manufacturing trace.

### 1. Motion Generation: POWL as Animation Choreography
Motion is a partially ordered workflow of body states. "Heavy weapon fire" is not an animation clip; it is a process (AcquireTarget < Fire < AbsorbRecoil). 
The animation system procedurally generates **lawful motion**.

### 2. Geometry and Skins as Process Outputs
You cannot generate everything independently. SkeletonEnvelope < JointClearanceZones < ArmorPanels < DamageMasks.
Skins are generated from heat profiles, damage history, material classes, and semantic LOD roles. The skin becomes a projection of process history.

### 3. POWL as Procedural Engineer and Art Director
POWL coordinates what must exist, what may vary, what must align, what can be skipped, what must be verified, and what gets refused.
If motion fails validation, POWL routes the repair (e.g., Option A: shrink armor panel, Option B: adjust joint limit).

### 4. Semantic LOD Generation
POWL coordinates Semantic LOD across geometry, skin, motion, damage, and collision, ensuring important mech features (like silhouette and damage state) survive simplification.

### 5. The "Never Random Soup" Invariant
Every generated artifact must know: why it exists, what process created it, what constraints admitted it, what verifier checked it, and what receipt proves it. Otherwise, it is REFUSED as residual inventory.

---

## The Crown Law
**POWL v2 is the admissible process grammar for manufactured worlds.**
Can a POWL v2 model generate a playable process surface whose traces are logged, checked, replayed, visualized, and improved? Minimum crown slice: 1 GMF process, 1 OCEL trace, 1 conformance replay, 1 receipt.
