## 2026-06-19T18:26:07Z

You are the Worker subagent for milestone GC-GUNDAM-FACTORY-001.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_gundam_factory_002/`.
Please create this directory first.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Please execute the following tasks:

Phase 1: Copy C++ Header Files
1. Copy `/Users/sac/rocket-craft/generated/gundam_factory/GundamFactorySteps.h` into `/Users/sac/rocket-craft/versions/v4_27_0/Source/Brm/`. Verify it was successfully copied.

Phase 2: Run UE4 HTML5 Packaging
1. Execute `/Users/sac/rocket-craft/package-brm-html5.sh`.
   Check the console output and logs to ensure the packaging succeeds and outputs the WASM package (Brm.wasm, Brm.js, Brm.html, Brm.data) to `/tmp/brm-html5-archive/HTML5` or `/tmp/brm-html5-archive`.
   Verify that the `.wasm` magic bytes are `0061736d` and the file size is >100 MB.

Phase 3: Stage HTML5 and generated deliverables
1. Copy the packaged HTML5 files from the archive directory into `pwa-staff/manufactured/`.
2. Create directory `pwa-staff/manufactured/generated/gundam_factory/` and copy all 13 generated files from `/Users/sac/rocket-craft/generated/gundam_factory/` into it, so they are served by the web server.

Phase 4: Implement Playwright Test and Config
1. Create `pwa-staff/tests-e2e/gundam_factory_walkthrough_projection.spec.ts`.
   It must:
   - Load the URL `/Brm.html`.
   - Log console output from the browser.
   - Wait for engine readiness signal (either window.Module.calledMain is true, window.UE4_EngineReady is true, or canvas dimension is >0).
   - Capture a baseline screenshot.
   - Click the canvas to focus.
   - Inject movement input: press and hold the 'W' and 'Space' keys for 8 seconds, then release.
   - Capture post-input screenshot.
   - Use `pixelmatch` to compute the visual delta (number of different pixels).
   - Assert that the visual delta exceeds the threshold (idle background delta + 50 pixels) and that the canvas is not blank (non-black pixels > 1000).
   - Calculate BLAKE3/SHA256 hashes of the screenshots.
   - Calculate the BLAKE3 hash of the compiled `Brm.wasm` file as `output_hash`.
   - Write a JSON receipt named `gundam-factory-playwright-receipt.json` under `pwa-staff/test-results/` with details: timestamp, screenshots (base64-encoded), consoleLogs, inputTrace, visualDelta, verdict, and output_hash.
2. Create `pwa-staff/playwright.gundam.config.ts` by copying `playwright.html5.config.ts` but setting `testMatch: '**/gundam_factory_walkthrough_projection.spec.ts'`.

Phase 5: Run Playwright Actuation Test
1. Create a script `/Users/sac/rocket-craft/verify_gundam_pipeline.sh` (modeled after `verify_html5_pipeline.sh`) to automatically serve on port 8080 and run `npx playwright test --config playwright.gundam.config.ts --reporter=list` inside `pwa-staff/`.
2. Execute this script and ensure the E2E verification completes successfully with a PASS verdict.

Phase 6: Run Pre-UE4 Benchmarks
1. Run `cargo bench -p rocket-preue4-verifier` or run the benchmark binary to measure the performance of authority update, Semantic LOD classification, walkthrough topology validation, projection manifest validation, and receipt replay. Record the sample sizes and timing results.

Phase 7: Generate Final Report and Receipt Chain
1. Combine the receipts from the verifier CLI, ggen, and Playwright screenshots to build the final tamper-evident receipt chain JSON:
   Create `/Users/sac/rocket-craft/VERIFIER_REPORT_GC_GUNDAM_FACTORY_001.json` following the required schema (sequence, event_type, surface, input_hash, output_hash, prev_hash, receipt, status, residuals).
2. Create `/Users/sac/rocket-craft/VERIFIER_REPORT_GC_GUNDAM_FACTORY_001.md` containing the required sections: Milestone, Scope, Repository Boundaries, Inputs, Generated Artifacts, Headless Rust Verification, ggen Manufacturing, UE4/WASM Projection, Playwright Visual Actuation, Receipt Chain, Agent Jidoka Events, Testing Ladder, Benchmark Results, Residuals, Next Falsifier, and Final Status.
   Set Scoped Status to: `GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE` and Final Status to: `GUNDAM_FACTORY_WALKTHROUGH_ALIVE_UNDER_SCOPE`.

Provide a detailed handoff report (`handoff.md`) in your working directory and message the parent when done.
