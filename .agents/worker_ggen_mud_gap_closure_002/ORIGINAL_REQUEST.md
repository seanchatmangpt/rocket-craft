## 2026-06-19T20:21:08Z

You are a teamwork_preview_worker subagent.
Your role name is: worker_ggen_mud_gap_closure_002
Your working directory is: /Users/sac/rocket-craft/.agents/worker_ggen_mud_gap_closure_002

Your task is to:
1. Update `ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl` with:
   - Metadata definitions and properties for `mud:ExpectedFile` and `mud:GapCheckRule`.
   - All expected Rust files, UE4 CSVs, and headers as `ExpectedFile` instances.
   - All check rules (e.g., Ggen Sync, Cargo tests, falsify/counterfactual cases, replay, verify) as `GapCheckRule` instances.
   - Use labels and comments conforming to the schema standard.
2. Save the SPARQL extraction query as `ontology/ggen-packs/mech_factory_mud/queries/gap_check.rq`. Ensure it is a target-oriented query extracting the ExpectedFiles, GapCheckRules, Stations, and RouteNodes, and uses deterministic `ORDER BY`.
3. Save the Tera template as `ontology/ggen-packs/mech_factory_mud/templates/rust/mud_gap_check.rs.tera` which generates `crates/mech_factory_mud/src/bin/mud_gap_check.rs`. Make sure it uses a split-based stdout parser for cargo test metrics to avoid requiring the `regex` library dependency.
4. Add the generation rule block to `ontology/ggen-packs/mech_factory_mud/ggen.toml`:
   ```toml
   [[generation.rules]]
   name = "rust-gap-checker"
   query = { file = "queries/gap_check.rq" }
   template = { file = "templates/rust/mud_gap_check.rs.tera" }
   output_file = "crates/mech_factory_mud/src/bin/mud_gap_check.rs"
   mode = "Overwrite"
   ```
5. Run `ggen sync --manifest ontology/ggen-packs/mech_factory_mud/ggen.toml` to execute the generation pipeline.
6. Verify the generated file compiles and runs successfully using `cargo run --bin mud_gap_check`. It should produce `generated/mech_factory_mud/gap_closure_report.json` and `gap_closure_report.md` with `PASSED` status.
7. Write a detailed handoff report in `/Users/sac/rocket-craft/.agents/worker_ggen_mud_gap_closure_002/handoff.md` and report back.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
