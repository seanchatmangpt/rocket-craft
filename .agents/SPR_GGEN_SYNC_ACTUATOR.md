# SPR: GGEN SYNC (Manufacturing Actuator)

## Core Doctrine
`ggen sync` is the deterministic manufacturing actuator, not an artist or judge. It is the law-to-assets synchronizer.
It does not freewheel. It synchronizes the generated artifact graph with the 100-file Source Law ontology and the active control plan.

## The 8-Stage Pipeline

### 1. Load and Actuate Source Law (100 TTL Files)
- Parses the 100 `*.ttl` files representing the DMEDI governance, plant cells, verifier gates, and receipts.
- Validates the graph for contradictions or dangling references via SHACL.
- Emits the `LAW_SYNC_REPORT.json` (Acceptance Test).

### 2. Resolve Manufacturing Rows (SPARQL)
Runs strict extraction queries to produce typed rows.
*Invariant:* Queries must strictly filter by ownership (`WHERE owner_part_id = "torso"`), not globally.

### 3. Dispatch Generator Cells
Routes rows to specialized generator cells:
- **Geometry Cell:** Emits rigidly scoped part files (e.g., `SM_Torso.usda`).
- **Surface Cell:** Emits MaterialX (`M_Armor_Primary.mtlx`).
- **Texture Cell:** Emits PBR stack and `texture_manifest.json`.
- **Rig/Socket Cell:** Emits `skeleton.usda`, `sockets.json`, limits.
- **Destruction/Loadout Cell:** Emits damage zones, armor breaks.
- **UE4 Projection Cell:** Emits import/cook manifests.

### 4. Deterministic Output Tree
Writes assets to a stable output tree:
`generated/flagship_ue4_mechs/pack_001/ {usd, materialx, textures, rig, destruction, ue4_export, verifier, evidence}`

### 5. Execute Sequential Verifier Funnel
Runs the strict Andon line:
`MODULAR_IDENTITY` → `PBR_MANIFEST` → `RIG_SOCKET` → `PRECOOK_RENDER` → `AI_VISION_PRECOOK` → `UE4_COOK_ELIGIBILITY` → `UE4_COOK_IMPORT` → `AI_VISION_ENGINE_FINAL` → `AGGREGATE_DISPOSITION` → `RECEIPT_SEALING`.
*Flow Discipline:* If a candidate fails at any gate, it dies immediately. No downstream waste.

### 6. Record Dispositions
Every candidate gets a typed disposition. No scalar vibe scoring.

### 7. Emit Reports
Emits phase-appropriate reports based on the control plan.

### 8. Emit OCEL & Receipts
Hashes all emitted artifacts via BLAKE3. Writes `asset_manufacturing.ocel.json` and `asset_receipts.jsonl`. Proves replay determinism.

## The 100 TTL Sync Acceptance Test
Before `ggen sync` fires generator cells, it must emit `LAW_SYNC_REPORT.json` proving graph coherence and control plan awareness:
```json
{
  "ttl_files_loaded": 100,
  "ttl_files_parsed": 100,
  "unresolved_references": 0,
  "contradictions": 0,
  "required_ctqs_present": true,
  "required_gates_present": true,
  "required_dispositions_present": true,
  "current_phase": "DEVELOP",
  "current_gate": "MODULAR_IDENTITY_SMOKE",
  "required_next_artifact": "MODULAR_IDENTITY_SMOKE_REPORT",
  "forbidden_artifacts": [
    "DOE_FACTOR_MATRIX",
    "PARETO_FAILURE_REPORT"
  ],
  "release_decision": "DOE_HELD"
}
```

## Control Plan Enforcement
If the current gate is `MODULAR_IDENTITY_SMOKE`, `ggen sync` ONLY runs 3 seeds, checks USD identity gates, writes the smoke report, and evaluates `DOE_RELEASED` vs `DOE_HELD`. It strictly refuses to touch PBR, Vision, or UE4 Cook.
