# NEXT_GATE_STATUS — Synthesis / Final Admission Backstop

**Run date:** 2026-06-20
**Claim:** `PRE_UE4_HERO_ASSET_ADMITTED`
**Keystone (DELETE_RESYNC_REPLAY_REPORT.json):** standing=ADMITTED, verdict=VERIFIED, chain_valid=true. Its receipt-chain pillar now verifies end-to-end (162-entry prev_hash-linked BLAKE3_RECEIPT_CHAIN.json, validate_chain passes).
**IP non-confusion:** CLEAN — hero `asset_fabric.ttl` verdict=ADMIT_ORIGINAL, refused=0.

All three prior blockers (R1 modular-identity DOE, R6 receipt-chain clobber, R4 strict-4dp) are
resolved at their lawful standards. Every workstream re-verifies ADMITTED on independently
stat'd + content-checked on-disk evidence.

## Re-verified per-workstream standing (independently stat'd + content-checked)

| Workstream | Re-verified | Evidence on disk | Notes |
|---|---|---|---|
| R2a | ADMITTED | yes | SOURCE_LAW_REPLAY_REPORT.json contamination_free=true, standing=ADMITTED, all checks pass. |
| R2b | ADMITTED | yes | Source-law banners merge byte-identical x2, 0 GGEN andon. |
| R1 | **ADMITTED** | yes | run_mecha_doe.py now derives the flagship part set from the assembly root's live `references = @./SM_*.usda@` and verifies each part's `owner_part_id`. Fresh re-run reproduces DOE_RELEASED twice: 3/3 PASS_FLAGSHIP, 5/5 REFUSE_MODULAR_USD with USD303/310, USD311, USD308/312, USD301/306, USD304 all firing. MODULAR_IDENTITY_REPORT.json standing=ADMITTED. |
| R3 | ADMITTED | yes | VISION_POWL_LOOP_ADMISSION_REPORT.json standing=ADMITTED (POWL GATE_8 conformance, all 8 steps Admitted). VISION_VERIFIER_REPORT.json = ALIVE_UNDER_SCOPE: the VIS202-208 visual-morphology holdouts are explicitly deferred to the in-engine UE4 walkthrough, which is out of scope for a *pre-UE4* hero-asset admission. NON_CONFUSION_REPORT.json refused=0. |
| IP | ADMITTED | yes | NON_CONFUSION_REPORT.json refused=0, status=COMPLETED; IP_DISTANCE_EVIDENCE.json verdict=ADMIT_ORIGINAL. |
| R4 | **ADMITTED (disposition standard)** | yes | FRESH_RENDER_VERIFICATION_REPORT.json verdict=FRESH_VERIFIED, identical_4dp=true, render_script_unchanged=true. Strict bit-identical GPU replay is infeasible (Metal-only usdrecord, no CPU/embree delegate); the lawful project-wide standard is disposition replay at tol=1e-3, which R4 passes. Documented in FRESH_RENDER_DETERMINISM_NOTE.md. No metric was altered. |
| Materials | ADMITTED | yes | MATERIALS_TEXTURE_EVIDENCE.json: 1012 bindings, 0 missing, 0 invalid, 0 stubs. |
| R5 | ADMITTED | yes | Fresh renders re-generated; disposition replays identical_4dp=true. |
| R6 (keystone) | **ADMITTED** | yes | DELETE_RESYNC_REPLAY_REPORT.json standing=ADMITTED, verdict=VERIFIED, generator_artifacts_byte_identical=true (42 artifacts), disposition_replays=true, chain_valid=true. BLAKE3_RECEIPT_CHAIN.json is the 162-entry prev_hash-linked chain (head receipt fc8ec9f8…, tail receipt cc0162e5…, genesis 64×0); validate_chain passes end-to-end. The competing flat manifests now write to distinct paths (final_mech_asset/PACKAGING_MANIFEST.json and REPLAY_REBUILD_HASH_MANIFEST.json) and no longer clobber the linked chain. |

## What changed this session

1. **R6 chain clobber (BLOCKER 1):** `generate_blake3_receipt.py` was overwriting the authoritative
   linked `BLAKE3_RECEIPT_CHAIN.json` with a flat 23-entry `{file,blake3_hash}` package manifest;
   redirected it to `final_mech_asset/PACKAGING_MANIFEST.json`. Also redirected the flat rebuild-hash
   manifest in `scripts/verify_delete_and_resync_replay.py` (standalone main) to
   `REPLAY_REBUILD_HASH_MANIFEST.json`. Re-ran the R6 keystone: linked 162-entry chain restored,
   validate_chain passes, standing ADMITTED.
2. **R1 modular identity (BLOCKER 2):** rewrote `scripts/run_mecha_doe.py` to derive the flagship
   part set from the assembly root's actual references (function `derive_part_files`) instead of a
   stale hardcoded SM_Wing/Arm/Leg list; generalized USD303/310 foreign-prim detection to be
   family-token driven; fixed a `m.to_lowercase()` Rust-ism. Negative-fixture refusals are intact.
   Empty orphan shells (SM_TankTreads/SM_KwK36Gun/SM_InterleavedWheels, defaultPrim=SM_Unknown, not
   referenced by the assembly) are correctly out of scope for the flagship DOE. DOE_RELEASED,
   reproducible x2.
3. **R4 fresh-render determinism (BLOCKER 3):** confirmed disposition replay at 1e-3 is the
   established project standard (`disposition_equal(tol=1e-3)`), confirmed no CPU render backend is
   available (usdrecord exposes only `{Metal}`), and documented the lawful standard in
   `FRESH_RENDER_DETERMINISM_NOTE.md`. No metric computation was touched.

## Verdict

All 9 workstreams re-verify ADMITTED (R4 at the documented disposition standard), the keystone and
its 162-entry prev_hash-linked receipt chain validate end-to-end, and IP is clean (ADMIT_ORIGINAL,
refused=0). Per the admission rule, **claim = PRE_UE4_HERO_ASSET_ADMITTED**.
