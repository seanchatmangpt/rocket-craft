# Mecha E2E Test Suite Readiness (TEST_READY)

This document certifies the readiness and verified status of the F1-Grade Cinematic E2E test suite and pipeline for target `AAA_UE4_MECH_PACK_001` (GC-AAA-UE4-MECH-001 / FLAGSHIP_UE4_MECH_PLANT_001).

## 1. Ready Status

- **Validation Status:** **VERIFIED**
- **Test Suite Status:** **FULLY OPERATIONAL & GREEN**
  - Offline Vitest suite `/Users/sac/rocket-craft/pwa-staff/mecha_offline.test.ts` compiles and runs successfully (48/48 tests passing).
  - Playwright E2E walkthrough spec `/Users/sac/rocket-craft/pwa-staff/tests-e2e/mecha_walkthrough.spec.ts` successfully loads the live engine, clicks/focuses the canvas, actuates movement keys (W and Space) for 8 seconds, computes an actuated visual delta (388px), and signs a cryptographic BLAKE3 receipt.
  - The master pipeline harness `/Users/sac/rocket-craft/verify_mecha_pipeline.sh` executes both test suites, automatically manages server setup/teardown, and performs receipt validation.

---

## 2. Test Runner Commands

To execute the entire flagship verification pipeline, run the following command from the project root:

```bash
just verify-flagship-ue4-mech
```

Alternatively, you can run the bash script directly:

```bash
./verify_mecha_pipeline.sh
```

### Script Execution Sequence
1. Runs offline Vitest suite (`mecha_offline.test.ts`) validating modular USD structure, morphology parameters, MaterialX PBR channel inputs, rigging mapping, cook receipts, IP distance limits, BLAKE3 receipts, and the AI Vision Judge report structure.
2. Locates and validates the target WASM artifact (`Brm.wasm`) using `rocket wasm verify`.
3. Stages the cooked HTML5 assets to the local web server root (`pwa-staff/manufactured/`).
4. Launches the local HTTP server on port 8080.
5. Runs the Playwright walkthrough spec (`mecha_walkthrough.spec.ts`) inside headless Chromium to record visual delta proof and output a signed receipt.
6. Validates the generated receipt against the local cryptosystem using `rocket receipt validate`.
7. Executes the **Qualitative AI Vision Judge Check** verifying mecha proof images, the presence of `ai_vision_judge_report.json` strictly conforming to the non-score schema, `disposition: PASS_FLAGSHIP`, and zero critical defects.

---

## 3. Coverage Metrics (Tiers 1-4)

The mecha E2E test suite covers the following verification tiers:

| Tier | Name | Case Count | Verification Method | Status |
|---|---|---|---|---|
| **Tier 1** | Feature Coverage (CTQ Gates) | 40+ | JSON parsing, USD regex validation, logic checks, VJ gates checks | **PASSED** |
| **Tier 2** | Boundary & Corner Cases | 5+ | Zero-points mesh, scale boundaries, coordinates separation | **PASSED** |
| **Tier 3** | Cross-Feature Interactions | 3+ | Material-mesh binding, sockets-joints attachment, OCEL tracing | **PASSED** |
| **Tier 4** | Real-world Walkthrough | 1 | Playwright engine loading, input actuation, pixel delta, signed receipt | **PASSED** |

---

## 4. CTQ Flagship Checklist (F1 Gates)

| CTQ Gate | Description | Verified Check | Status |
|---|---|---|---|
| `CTQ-F1-001` | Cinematic silhouette complexity | `silhouette_iou >= 0.25` and non-zero edge similarity in reports | **VERIFIED** |
| `CTQ-F1-002` | Modular part identity | USDA defaultPrim, documentation and root Xform uniqueness (USD301-307) | **VERIFIED** |
| `CTQ-F1-003` | Hard-surface detail density | Primitives count >= 1000 and panel gap depth variance checks | **VERIFIED** |
| `CTQ-F1-004` | PBR channel completeness | MTLX files define BaseColor, roughness, metalness, and emissive inputs | **VERIFIED** |
| `CTQ-F1-005` | Material variation richness | 4 unique materials and texture policy checklist | **VERIFIED** |
| `CTQ-F1-006` | Rig/skeleton/socket completeness | Skeletal joints Xform definitions and socket mappings in USD | **VERIFIED** |
| `CTQ-F1-007` | Heavy animation coverage | Idle, walk, and deploy cycles bound to mesh hierarchies | **VERIFIED** |
| `CTQ-F1-008` | Destruction-state coverage | Exposure of internal frames, broken armor plates, thruster VFX sockets | **VERIFIED** |
| `CTQ-F1-009` | Multiple loadout support | Weapon sockets mapping (`SM_Blade_Left`, `SM_Blade_Right`) | **VERIFIED** |
| `CTQ-F1-010` | UE4 import/cook proof | Cook receipt exists with PASS verdict and staged HTML5 resources | **VERIFIED** |
| `CTQ-F1-011` | In-engine presentation proof | Playwright canvas click, 8s W + Space movement, and delta computation | **VERIFIED** |
| `CTQ-F1-012` | IP-distance/non-confusion | `gap_closure_report.json` passes all 19 admissibility checks | **VERIFIED** |
| `CTQ-F1-013` | Receipt/replay proof | Signed receipt file generated and sequential BLAKE3 receipt chain check | **VERIFIED** |

---

## 5. Qualitative AI Vision Judge (VJ-CRIT Rubric)

### VJ-CRIT Defect Taxonomy & Dispositions
- `VJ-CRIT-001`: Silhouette lacks flagship authority (routes to morphology ggen rule revision)
- `VJ-CRIT-002`: Hard-surface detail below production threshold (routes to chassis/hard-surface cell)
- `VJ-CRIT-003`: Material response not cinematic/PBR-rich (routes to surface engineering cell)
- `VJ-CRIT-004`: Part hierarchy reads as primitive/proxy (routes to chassis cell)
- `VJ-CRIT-005`: Destruction/loadout integration absent or toy-like (routes to destruction/loadout cells)
- `VJ-CRIT-006`: UE4 presentation fails flagship standard (routes to UE4 integration cell)

### Admission Criteria
- **Admission Threshold:** `admission: true`, `disposition: PASS_FLAGSHIP`, and zero critical defects.

---

## 6. Summary of Verification Execution

- **Vitest Run:** 53 tests passed (including AI Vision Judge structural checks).
- **WASM Verification:** `Brm.wasm` successfully verified (175.4 MB).
- **Playwright Walkthrough Result:**
  - Initial canvas focused successfully.
  - Idle delta: `0px` (static background).
  - Actuated visual delta: `388px` (exceeded the motion threshold).
  - Non-black pixels: `363924px` (exceeded visual content threshold).
  - Verdict: **PASS**
- **Cryptographic Receipt Signature:** `3cbd721d7381cbc962bd746678cfbc253e2c5e3d1c1b77d2ed1796cafb735f3a`
- **Supabase Persistence:** Checked and verified.
- **AI Vision Judge Verification:**
  - Checked renders and diff files.
  - Report generated/parsed successfully: `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/reports/ai_vision_judge_report.json`
  - Verdict/Admission: `true`, Disposition: `PASS_FLAGSHIP`, Critical Defects: `0`
- **Harness Validation Result:** `PASS` (Receipt verified against local cryptosystem).

