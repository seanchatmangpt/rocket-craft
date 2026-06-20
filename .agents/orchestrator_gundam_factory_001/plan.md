# Plan — GC-GUNDAM-FACTORY-001

## Architecture & Flow
We will execute the Gundam Factory Walkthrough Projection milestone using the Project Pattern. The workflow is split into:
1. **Gate 1: Headless Rust Pre-UE4 Verification**
   - Ensure the verifier crate tests pass.
   - Verify byte fields, branchless typestates, SIMD/scalar equivalence, Semantic LOD, walkthrough topology, and surrogates.
2. **Gate 2: ggen Manufacturing**
   - Run the ggen generator to manufacture Gundam Factory-specific artifacts.
   - Verify that all outputs are deterministic and none are orphaned.
3. **Gate 3: UE4 HTML5/WASM Projection**
   - Package the environment to HTML5/WASM via the SpeculativeCoder UE4.27 fork.
   - Verify C++ headers and DataTables are correctly consumed.
4. **Gate 4: Playwright Visual Actuation**
   - Serve the WASM build locally.
   - Open in Playwright, detect readiness, take baseline and post-input screenshots, verify visual delta, and generate BLAKE3 receipt chain.
5. **Final Report & Verification**
   - Author verifier report in markdown and JSON in `/Users/sac/rocket-craft/`.

| Milestone | Name | Objective | Dependencies | Status |
|---|---|---|---|---|
| M1 | Exploration | Explore existing code, ggen configs, and setup | None | DONE |
| M2 | Gate 1 Verification | Run and pass pre-UE4 verifier tests | M1 | DONE |
| M3 | Gate 2 Manufacturing | Generate Gundam Factory artifacts using ggen | M2 | DONE |
| M4 | Gate 3 Packaging | Package UE4 project into HTML5/WASM | M3 | IN_PROGRESS |
| M5 | Gate 4 Actuation | Run Playwright test, capture screenshots & visual delta, generate receipt | M4 | PLANNED |
| M6 | Report & Handoff | Generate final reports and receipts, verify all gates | M5 | PLANNED |
