## 2026-06-19T18:02:55Z

You are the Explorer subagent for GC-GUNDAM-FACTORY-001.
Your working directory is `/Users/sac/rocket-craft/.agents/explorer_gundam_factory_001/`.

Your tasks:
1. Locate and examine the pre-UE4 verifier codebase (`crates/rocket_preue4_verifier`) and identify all places that refer to the previous milestone `GC-MECHBIRTH-002` or `MechBirth` assets/steps, and list what changes/extensions are needed to support `GC-GUNDAM-FACTORY-001` (Gundam Factory walkthrough).
2. Analyze how `ggen` is used in this repository. Identify the exact configuration file (e.g., `ggen.toml` or similar) that needs to be updated or created to procedurally generate the required Gundam Factory artifacts under `~/rocket-craft/generated/gundam_factory/` from the ontology `ontology/gundam_nexus.ttl` or `ggen-validation-tests/`.
3. Investigate the UE4 HTML5 packaging flow. What uproject is used, how are generated C++ headers / DataTables / manifests imported into it, and how does the packaging script compile and produce the final WASM and data files?
4. Look at the Playwright test setup. Identify where e2e tests are run, what configuration they use, and how to implement the required `gundam_factory_walkthrough_projection.spec.ts` test to load the WASM page, wait for readiness, inject input, and compute the visual screenshot delta.
5. Create a detailed Gap Analysis Report (`analysis.md`) in your working directory listing your findings, and write a self-contained handoff (`handoff.md`) summarizing the exact next steps for the implementation phase. Send a message to parent when done.
