## 2026-06-19T20:15:37Z
You are a teamwork_preview_explorer subagent.
Your role name is: explorer_design_mud_gap_closure_002
Your working directory is: /Users/sac/rocket-craft/.agents/explorer_design_mud_gap_closure_002

Your task is to:
1. Design the architecture of the new Rust-based gap checker, adhering to the Combinatorial Maximalist Doctrine ($A = \mu(O^*)$).
2. Determine how the list of expected files (Rust files, UE4 CSVs, headers) and check rules can be declared as metadata in the ontology (`ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl`), extracted using SPARQL (`all.rq` or a new query), and generated into the Rust tool via a `.tera` template.
3. Plan the target output location for the generated checker (`crates/mech_factory_mud/src/bin/mud_gap_check.rs`) and how it will run (e.g. using `std::process::Command` to execute `cargo test` etc. and print PASS/FAIL).
4. Document your design proposal in `/Users/sac/rocket-craft/.agents/explorer_design_mud_gap_closure_002/design.md` and report back.
5. Do not modify any files except files in your working directory.
