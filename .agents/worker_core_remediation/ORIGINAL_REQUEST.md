## 2026-06-19T01:05:41Z
You are worker_core_remediation. Your working directory is `/Users/sac/rocket-craft/.agents/worker_core_remediation`.
Please initialize your progress.md and context.md.

## Objective
Remediate the integrity violations and quality defects identified by the Forensic Auditor and Reviewer 1. You will fix the ggen compiler codebase, install the updated compiler, update the ggen.toml configuration, update core.ttl, and verify the entire system.

## Task Details
1. **Apply Compiler Code Patches**:
   - Patch `crates/ggen-core/src/validation/sparql_rules.rs` using `/Users/sac/rocket-craft/.agents/explorer_core_remediation/sparql_rules.rs.patch` (or manually adjust the code if needed).
   - Patch `crates/ggen-core/src/codegen/pipeline.rs` using `/Users/sac/rocket-craft/.agents/explorer_core_remediation/pipeline.rs.patch`.
   - Patch `crates/ggen-core/src/codegen/executor.rs` using `/Users/sac/rocket-craft/.agents/explorer_core_remediation/executor.rs.patch`.
   - All paths are relative to `/Users/sac/ggen/`.
2. **Build and Install Compiler**:
   - Compile the workspace inside `/Users/sac/ggen/` by running `cargo build`.
   - Run unit tests `cargo test --package ggen-core` to verify correctness.
   - Install the compiled binary locally to `/Users/sac/.local/bin/ggen` by running:
     `cargo install --path crates/ggen-cli --locked --root /Users/sac/.local/ --force`
3. **Update ggen.toml**:
   - Update `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` with the proposed `when` clauses for inference rules.
   - Add verification for `ue4:USceneComponent rdfs:subClassOf ue4:UActorComponent .` to validation rule `R1` in `ggen.toml`.
4. **Update core.ttl**:
   - Update `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` to explicitly declare `ue4:isComponentOf` and `ue4:isLevelOf` as `owl:ObjectProperty`.
   - Resolve redundancy by ensuring `ue4:owner` is defined as the inverse of `ue4:hasComponent`, and remove the redundant `ue4:hasOwner` property.
5. **Verify**:
   - Execute the validation script `/Users/sac/rocket-craft/validate_ontology.sh`. It must pass successfully with exit code 0.
   - Run the overall project test suite `./rocket test` if applicable (or other unit tests) and ensure they pass.

## Mandatory Integrity Warning
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.

Report completion, test logs, and handoff path to parent ID: 4f79cb22-2adb-466d-9e20-d8baef6e934d.
