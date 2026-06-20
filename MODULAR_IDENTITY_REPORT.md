# MODULAR IDENTITY REPORT (R1) — FRESH RE-RUN

Generated: 2026-06-20T22:12:56.888165Z
Regenerator: `scripts/run_mecha_doe.py` (runnable)

## Verdict: PARTIAL_ALIVE — fresh re-run does NOT reproduce DOE_RELEASED

| Source | release_decision | smoke PASS | neg REFUSE |
|---|---|---|---|
| Committed static evidence (HEAD) | DOE_RELEASED | 3/3 | 5/5 |
| **Fresh re-run (this session)** | **DOE_HELD** | **0/3** | 5/5 |

## Gap / Root Cause
`run_mecha_doe.py` hardcodes a stale `part_files` list:
`SM_Wing_Left/Right`, `SM_Arm_Left/Right`, `SM_Leg_Left/Right` — none of which ggen emits today.
Current canonical part files are: `SM_WingArray_Left/Right`, `SM_Limb_Left/Right`, plus tank parts
`SM_TankTreads`, `SM_KwK36Gun`, `SM_InterleavedWheels`, `SM_Loadout`.

Result: every smoke seed emits 6 spurious `USD304 expected part root missing` errors → no PASS_FLAGSHIP → DOE_HELD.
All 5 negative fixtures still REFUSE with their intended diagnostics, but contaminated by the same 6 false USD304s.

## Fresh geometry-level modular identity (ad-hoc, NOT named verifier)
- Part files present: 12
- Unique fingerprints: 12 (all unique)
- owner_part_id complete: yes
- Torso foreign geometry: none
- Socket mesh payload: none

Geometry modular identity HOLDS for current output, but the named DOE verifier cannot confirm it until repaired.

## Next action
Repair `scripts/run_mecha_doe.py` to DERIVE the part list from the actual `SM_*.usda` roots
(via each file's `custom string owner_part_id`, excluding `ASSET_ReferenceFabric_001.usda`),
re-run, reproduce DOE_RELEASED, and delete-and-resync replay twice.
