# Counterfeit Artifact Audit Project Plan

This plan outlines the milestones and steps required to scan the `rocket-craft` project to identify, catalogue, and report all LLM-generated fake, cheat, or mock artifacts that falsely claim completion of the genuine Combinatorial Maximalist requirements.

## Milestones

### Milestone 1: Source Scanning & Audit Identification (Exploration)
- **Objective**: Identify all Python wrappers, simulated engines, mock Three.js viewers, and placeholder/stub outputs in the repository.
- **Tasks**:
  - Spawn an Explorer to scan the entire workspace for files matching the counterfeit criteria.
  - Specifically search for:
    - Simulated engines: Python files, shell scripts, or other binaries (e.g., simulating `ue4-sim`, engine runs, or bypasses of Unreal Engine compilation).
    - Mock projections: HTML/JS/TS files containing Three.js, Canvas, WebGL mocks, or simulated visual outputs.
    - Stubs: Tiny wasm files, mock t3d generators, or hardcoded json generators (e.g., `spec.json`, `map.t3d`).
  - Document all candidate file paths and details.
- **Verification**: Handoff containing list of potential counterfeit files.

### Milestone 2: Report Generation (Worker)
- **Objective**: Synthesize the explorer's findings and write a comprehensive markdown report.
- **Tasks**:
  - Spawn a Worker to create `counterfeit_artifacts_report.md` in the project root.
  - Catalog every counterfeit file found with its path and type.
  - Explain why each violates the CM doctrine (referencing GEMINI.md).
- **Verification**: Report is successfully written to `/Users/sac/rocket-craft/counterfeit_artifacts_report.md`.

### Milestone 3: Verification & Review (Reviewer)
- **Objective**: Review the report for completeness, accuracy, and compliance with the CM doctrine.
- **Tasks**:
  - Spawn a Reviewer to verify that all requirements of the user request are met.
  - Check that the report has the correct layout, covers all identified fakes, and references GEMINI.md correctly.
- **Verification**: Reviewer handoff confirms report validity and correctness.
