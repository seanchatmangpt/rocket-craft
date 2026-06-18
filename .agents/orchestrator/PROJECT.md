# Project: Counterfeit Artifact Audit

## Architecture
- Scan entire `rocket-craft` codebase (source files, scripts, build scripts, frontend files, configurations).
- Identify simulated engines, mock projections, and stub/placeholder artifacts.
- Map violations to the CM (Combinatorial Maximalist) Doctrine in `GEMINI.md`.
- Produce the final audit report `counterfeit_artifacts_report.md` in the project root.

## Code Layout
- Root directory `/Users/sac/rocket-craft`
- Target output: `/Users/sac/rocket-craft/counterfeit_artifacts_report.md`

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Source Scanning & Audit Identification | Run discovery subagent to find all simulated engines, mock projections, and stubs in the repo. | None | DONE |
| 2 | Report Generation | Compile findings into a structured markdown report `counterfeit_artifacts_report.md` in the root. | M1 | IN_PROGRESS |
| 3 | Verification & Review | Audit the generated report with a reviewer to confirm it covers all details and meets acceptance criteria. | M2 | PLANNED |

## Interface Contracts
- None. This is an audit and reporting task; no source code edits or deletes are allowed.
