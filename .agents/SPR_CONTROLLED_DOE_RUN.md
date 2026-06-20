# SPR: CONTROLLED DOE RUN (GC-FLAGSHIP-UE4-MECH-001)

## Core Doctrine
The measurement harness is frozen. The next phase is a controlled combinatorial Design of Experiments (DOE) against the verifier wall.
The goal is a measured failure map (Pareto chart, Transfer Function) to mathematically drive the next source-law patches.

## Flow Discipline (The Funnel)
Do not burn UE4 cook cycles on upstream failures. The line stops at the first critical defect.
`source-law candidate -> static artifact checks -> modular USD checks -> PBR manifest checks -> rig/socket checks -> render checks -> AI Vision Judge -> UE4 cook eligibility -> UE4 cook -> final disposition`

## Pre-Flight Negative Fixture Gates
Before launching the 100-seed batch, the pipeline MUST prove it automatically refuses these fixtures:
1. `old blob/line-wing candidate` -> `REFUSE_NON_FLAGSHIP`
2. `duplicate USD part candidate` -> `REFUSE_MODULAR_USD`
3. `missing PBR manifest candidate` -> `REFUSE_PBR_INCOMPLETE`
4. `static mesh with no sockets` -> `REFUSE_RIG_SOCKET`
5. `fake AI judge score field` -> `SCHEMA_REFUSE`
6. `PASS_VISUAL_FLAGSHIP without upstream passes` -> `SCHEMA_REFUSE`

## Source Law Factors (X)
- `X_geometry`, `X_surface`, `X_rig`, `X_engine` parameters governing topology, materials, and engine integration.

## Required DOE Output Artifacts
The DOE Enumeration Cell must emit the following formal reports:
1. `DOE_FACTOR_MATRIX.csv/json`: The matrix of X parameters for all 100+ candidates.
2. `CANDIDATE_DISPOSITIONS.jsonl`: The final aggregate disposition, defects, and first-failure station per candidate.
3. `PARETO_FAILURE_REPORT.md`: Where does the line fail first? Which defects dominate?
4. `TRANSFER_FUNCTION_REPORT.md`: Which X factors appear to drive which Y failures?
5. `NEXT_PATCH_PRIORITY_REPORT.md`: What source-law patch gives the highest expected yield increase?
6. `OCEL manufacturing log`
7. `receipt chain`
