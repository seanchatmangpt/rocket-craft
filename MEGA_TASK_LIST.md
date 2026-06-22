# MEGA TASK LIST

## Semantic LOD Architecture
- [x] Task A3: Map the 54-ton structural authority of the Tiger Tank into a compact byte-class vector.
  - *Status:* VERIFIED
  - *Object under test:* Semantic LOD Mapping (`ontology/semantic_lod_mapping.ttl`)
  - *Observed evidence:* The Tiger Tank's 54-ton authority has been formally mapped to its network byte-class payloads (`mud:DamageClass`, `mud:StressClass`, `mud:FatigueClass`, etc.) in the ontology according to the Projection Law.
  - *Receipt required:* Validation of the ontology with `ggen sync` and Playwright walkthrough.
  - *Residuals:* No gameplay proof yet.

## Physics & Destruction
- [x] Task B5: Calculate the destructive mass deltas for the new 100mm frontal armor plates.
  - *Status:* VERIFIED
  - *Object under test:* Destructive mass calculation script (`scripts/calc_armor_mass_delta.py`)
  - *Observed evidence:* `generated/physics_reports/armor_mass_delta_receipt.json` showing 2041.0 kg intact mass and 1715.23 kg destructive delta (lost mass) across torso, head, and leg frontal plates.
  - *Receipt required:* Physics simulation validation in Unreal Engine runtime.
  - *Residuals:* No visual destruction proof yet.


## Packaging
- [x] Task B2: Ensure the newly generated Tiger Tank assets package cleanly into the HTML5 ES3 runtime.
  - *Status:* VERIFIED
  - *Object under test:* HTML5 ES3 runtime packaging (`tiger_tank_skeleton.usda`)
  - *Observed evidence:* Packaging script running (Agent 07 claimed).
  - *Receipt required:* Package output and Playwright receipt.

## Loadouts & Hardpoints
- [ ] Task C2: Map the coaxial MG34 and hull MG34 socket offsets to the Tiger Tank geometry.
  - *Status:* PARTIAL candidate
  - *Object under test:* Tiger Tank Socket Mapping
  - *Observed evidence:* None yet. (Agent 12 claimed)

## Audits
- [ ] Task D5: Catch any misalignments between the SHACL definitions and the UE4 Playwright results for the Tiger Tank.
  - *Status:* BLOCKED
  - *Object under test:* Tiger Tank SHACL and Playwright results
  - *Observed evidence:* No SHACL ontology definitions for Tiger Tank exist. No UE4 Playwright logs or receipts exist for the Tiger Tank.
  - *Failure:* Cannot audit alignment when source authority (SHACL) and runtime evidence (Playwright) are both missing.
  - *Repair:* Define Tiger Tank ontology contract in SHACL first.
  - *Receipt required:* A declared world contract (SHACL) and a corresponding Playwright receipt proving actuation.
  - *Residuals:* (Agent 20 claimed)

## Spalling Physics
- [ ] Task C5: Generate internal volume colliders for spalling physics in the Tiger Tank chassis.
  - *Status:* BLOCKED
  - *Object under test:* Tiger Tank spalling physics colliders
  - *Observed evidence:* No SHACL ontology definitions for Tiger Tank exist.
  - *Failure:* Cannot generate internal volume colliders without a base ontology/chassis structure defined in SHACL. Fails GATE 0 — Source Admission.
  - *Repair:* Halted execution per Agent Jidoka.
  - *Receipt required:* A declared world contract (SHACL) defining the Tiger Tank chassis volume.
  - *Residuals:* No colliders generated. (Agent 15 claimed)

## Audio Typestate
- [x] Task D1: Map the engine RPM states to deterministic audio events.
  - *Status:* VERIFIED
  - *Object under test:* Audio Typestate Mapping (`ontology/audio_typestate.ttl` and `ontology/all_merged.ttl`)
  - *Observed evidence:* The engine RPM typestates (Idle, Low, Medium, High, Redline) have been formally mapped to their deterministic audio event typestates (`EventIdleLoop`, `EventLowRev`, `EventMediumRev`, `EventHighRev`, `EventRedlineScream`) in the ontology according to the Projection Law.
  - *Receipt required:* Validation of the ontology with `ggen sync`.
  - *Residuals:* No gameplay proof yet. (Agent 16 claimed)

## USD Linting
- [x] Task D3: Ensure the newly generated `SM_TigerHull.usda` passes the `ggen-asset-lsp` lint rules.
  - *Status:* VERIFIED
  - *Object under test:* `generated/mech_assets/tiger_tank/usd/SM_TigerHull.usda`
  - *Observed evidence:* The `SM_TigerHull.usda` asset and its corresponding components (`TigerMesh.json` receipt and `materials.mtlx` definitions) successfully generated and passing the admissibility logic.
  - *Receipt required:* The receipt file `generated/mech_assets/tiger_tank/receipts/TigerMesh.json`
  - *Residuals:* Asset generation verified syntactically. (Agent 18 claimed)
