## 2026-06-19T18:13:06Z

You are the Worker subagent for GC-GUNDAM-FACTORY-001.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_gundam_factory_001/`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Please execute the following tasks:

Phase 1: POWL Model and Trace Generation
1. In `/Users/sac/powlv2lsp/samples/`, write a new POWL file named `GundamFactory.powl` representing the Gundam Factory walkthrough process. It must define a ChoiceGraph with 9 sequential activities (Spawn, FactoryEntrance, FrameAssembly, SocketTopology, ArmorSkinStation, RigMotionStation, VerificationGate, ReceiptTerminal, ExitOrLoop) and transition edges.
2. In `/Users/sac/powlv2lsp/`, run `npm run compile` to build the TypeScript files.
3. Use the simulator command:
   `node out/src/bin/sim.js samples/GundamFactory.powl gundam_factory_trace.json`
   to generate the OCEL trace file `gundam_factory_trace.json` in `/Users/sac/powlv2lsp/`. Verify it exists and is valid JSON.

Phase 2: Rust Pre-UE4 Verifier Parameterization & Tests
1. In `crates/rocket_preue4_verifier/src/ocel.rs`, refactor `OcelLog::from_powlv2lsp_trace` to parse relationships and dynamic `objectId` values instead of hardcoding `case-mechbirth-001`. Fallback to `case-gundam-factory-001` if no objects are present.
2. In `crates/rocket_preue4_verifier/src/bin/rocket_preue4_verify.rs`, refactor the CLI so that if `--milestone GC-GUNDAM-FACTORY-001` is passed:
   - It validates the 9 Gundam Factory walkthrough steps.
   - It expects the 13 Gundam Factory deliverables under `generated/gundam_factory/`.
   - The fallback/defaults for `--powl` and `--trace` point to `GundamFactory.powl` and `gundam_factory_trace.json`.
   - The reported milestone name is `GC-GUNDAM-FACTORY-001`.
   - It outputs the correct residuals list for `GC-GUNDAM-FACTORY-001`.
3. Create `crates/rocket_preue4_verifier/tests/integration_gundam_factory.rs` and `crates/rocket_preue4_verifier/tests/chaos_gundam_factory.rs` to verify the new walkthrough trace and chaos mutations for Gundam Factory.
4. Run `cargo test -p rocket-preue4-verifier` and ensure all tests compile and pass.

Phase 3: `ggen` Code Generation Mappings
1. In `ggen-validation-tests/ggen.toml`, add generation rules (`[[generation.rules]]`) to procedurally lower the ontologies (using SPARQL select/construct queries and Tera templates) into the 13 required deliverables under `/Users/sac/rocket-craft/generated/gundam_factory/`. Ensure these templates are robust and generate valid C++ and Rust code/CSVs/JSONs.
2. Run `/Users/sac/.local/bin/ggen sync --manifest /Users/sac/rocket-craft/ggen-validation-tests/ggen.toml` (or similar validate commands) to confirm validation and successful code generation.

Provide your handoff report (`handoff.md`) in your working directory and message the parent when done.
