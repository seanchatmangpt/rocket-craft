# Scope: E2E Testing Track for target FLAGSHIP_UE4_MECH_PLANT_001
(GC-AAA-UE4-MECH-001)

## Architecture
The E2E testing framework serves as the validation engine for the flagship cinematic mecha assets and their integration into the Unreal Engine 4 HTML5/WASM environment.
It includes:
- Offline tests (Vitest) validating static USD files, morphology metrics, MaterialX lookdev, skeletal structures, heavy animation sets, destruction states, multiple loadouts, 4K/8K texture mapping policy, IP-admissibility, AI Vision Judge binary JSON report, and receipts.
- Online tests (Playwright) loading the WASM world, injecting keystrokes (focus, movement, deploy, destruction), calculating visual delta, and confirming telemetry.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Test Infra Document | Author TEST_INFRA.md detailing 4-tier structure, F1 Gates (CTQ-F1-001 to CTQ-F1-013), and AI Vision Judge critical defects VJ-CRIT-001 to VJ-CRIT-006. | None | DONE |
| 2 | Offline Test Suite | Write Vitest specs verifying CTQ-F1-001 to CTQ-F1-013, destruction states, animations, loadouts, PBR 4K/8K, VJ-CRIT-001 to VJ-CRIT-006, and receipts. | None | DONE |
| 3 | Online Test Suite | Implement Playwright spec verifying WASM boot, canvas focus, heavy animation actuation, and visual delta. | None | DONE |
| 4 | Unified Test Runner | Create verify_mecha_pipeline.sh wrapping Vitest, Playwright, and VJ report binary evaluation. | Milestones 2, 3 | DONE |
| 5 | Publication | Publish TEST_READY.md showing E2E coverage and invocation commands (especially `just verify-flagship-ue4-mech`). | Milestone 4 | DONE |

## Interface Contracts
- **Test Results**: All offline tests output JUnit/Vitest reports.
- **Playwright Receipts**: Online test outputs tps-dflss-receipt.json containing visual delta, before/after screenshots, console logs, and BLAKE3 hash of Brm.wasm.
- **AI Vision Judge Report**: JSON report at `reports/ai_vision_judge_report.json` containing `disposition` and `critical_defects`.
- **TEST_READY.md**: Signal document at project root.
