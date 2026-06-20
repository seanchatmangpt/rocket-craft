# Scope: GC-MECH-FACTORY-MUD-002

## Architecture
- The new Rust gap checker tool (to be named `mud_gap_check`) will be implemented as a binary in the `mech_factory_mud` crate (specifically `src/bin/mud_gap_check.rs`).
- The checker will execute cargo commands and check file existence/contents to verify that all gaps are resolved.
- To comply with the Combinatorial Maximalist Doctrine, the lists of expected files and configurations will be defined in the ontology and extracted via SPARQL into the generated code of the gap checker.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Audit & Extraction | Inspect Python script, outline all check functions and design the Rust CLI structure | None | PLANNED |
| 2 | Ontology & Templates | Add expected files and test configurations to `mech_factory_mud.ttl` and create the `.rq` query and `.tera` template for generating the Rust gap checker | M1 | PLANNED |
| 3 | Rust Implementation | Generate `src/bin/mud_gap_check.rs` via `ggen sync` and implement any supporting Rust logic | M2 | PLANNED |
| 4 | Verification & Audit | Run the new Rust gap checker, verify it yields identical results to Python, and run full test suites | M3 | PLANNED |

## Interface Contracts
### `mud_gap_check` CLI ↔ workspace
- Execution: `cargo run --bin mud_gap_check`
- Outputs: `generated/mech_factory_mud/gap_closure_report.json` and `generated/mech_factory_mud/gap_closure_report.md`
- Return status: exit code 0 if all requirements pass, non-zero if any fail.
