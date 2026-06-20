# SPR: DMEDI OPERATING MANUAL

## Core Doctrine
The factory is governed by the DMEDI (Define, Measure, Explore, Develop, Implement) methodology for new-product/new-process generation. This is the formal operating system for flagship AAA UE4 mech manufacturing.

## Current Official State
- `DMEDI_DEFINE`: PARTIAL_ALIVE / acceptable
- `DMEDI_MEASURE`: ALIVE / TEST_READY
- `DMEDI_EXPLORE`: PARTIAL_ALIVE
- `DMEDI_DEVELOP`: BLOCKED_ON_MODULAR_IDENTITY_SMOKE
- `DMEDI_IMPLEMENT`: NOT_STARTED

## Required Artifacts per Phase

### 1. DEFINE
- `charter.md`: Manufacture flagship/cinematic AAA UE4 mechs non-humanly.
- `MGPP.md`: Mission, Goals, Plans, Problems.
- `risk_register.md`: false admission, toy/proxy output, IP risk, duplicate files.
- `communication_plan.md`

### 2. MEASURE
- `VOC.md`: Customer demands F1-grade UE4 mech, not skin, not toy.
- `QFD_matrix.csv`: CTQs (modular USD, PBR, rig/sockets, destruction, loadouts, UE4 cook, Vision Judge, IP distance, receipts).
- `CTQ_tree.json`
- `target_cost_model.md`: Collapse $2M–$5M human cost/time.
- `scorecard.json`
- `MSA_report.md`: Verify the verifier wall; binary disposition validation.
- `process_capability_baseline.md`: Currently UNKNOWN.

### 3. EXPLORE
- `TRIZ_contradictions.md`: Detail vs UE4 performance, modularity vs coherence.
- `concept_matrix.csv`
- `Pugh_matrix.csv`
- `AHP_weights.json`: Select generator families (layered_swept_feather_panel, etc.).
- `design_FMEA.md`: Forecast defects (foreign prim leakage, socket smuggling).
- `Monte_Carlo_plan.md`

### 4. DEVELOP (Current Phase)
- `MODULAR_IDENTITY_SMOKE_REPORT` (Blocks DOE)
- `DOE_factor_matrix.json`
- `candidate_dispositions.jsonl`
- `pareto_failure_report.md`
- `transfer_function_report.md`
- `next_patch_priority_report.md`
- `robust_design_report.md`

### 5. IMPLEMENT
- `pilot_report.md`
- `control_plan.md`: Control charts on defect rates (`REFUSE_MODULAR_USD`, etc.).
- `replay_report.md`
- `capstone_verifier_report.md`
