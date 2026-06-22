# Handoff Report — Praxis Upgrade Remediation

This report details the observations, logic chain, decisions, and verification steps performed to remediate the defects identified in `/Users/sac/praxis`.

## 1. Observation
We observed the following:
- In `/Users/sac/praxis/template/Cargo.toml`, the package name was hardcoded as `praxis_template`, the description was `Praxis template upgrade`, and the URLs used `praxis_template`.
- In `/Users/sac/praxis/template/Cargo.workspace.toml`, the example package name was `#   name = "praxis_template-core"`. The repository and homepage URLs also had hardcoded `praxis_template`.
- In `/Users/sac/praxis/template/src/types.rs`, the documentation and doc-tests used `{{project-name}}` with a hyphen (e.g. `use {{project-name}}::Blake3Hash;`), which does not compile under doctests when replaced with a project name containing a hyphen (hyphens are invalid in Rust identifiers).
- Crate name placeholders in various other files (`chain.rs`, `error.rs`, `verbs/verifier.rs`, `cli.rs`) used inconsistent placeholders like `{{project-name}}` or `{{crate_name}}`.
- The crate root `lib.rs` was not re-exporting all domain/typestate types from `types.rs`, leading to compilation failures in downstream doc-tests referencing types like `ProfileId`.
- The optional `tokio` dependency was not constrained for binary targets like `mcp_server`, leading to compile issues when `mcp` feature was not selected.
- The `lsp` feature mapping was not activating the `lsp-max/proposed` feature, which is required for trait signatures to match exactly.

Verification commands run:
- `/Users/sac/praxis/tools/verify_conformance.sh` was updated to generate `/tmp/my-conforming-project` and run hollow-gate checks.
- Clean compilation checked via `cargo check --all-targets --all-features` inside `/tmp/my-conforming-project`.
- Test verification run via `cargo test` and `cargo test --all-features` using a custom target directory `/Users/sac/cargo_target_conforming` to avoid macOS temp directory cleanup issues.

Outputs of the verification commands:
- Conformance verdict was `[CONFORMANCE SUCCESS] Generated project successfully verified!` and `=== Conformance Verification Verdict: VERIFIED ===`.
- Default tests: `test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` and doc-tests: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.75s`.
- All features tests: `test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` and doc-tests: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.89s`.

## 2. Logic Chain
- Reverting the hardcoded package name, description, and URLs back to `{{project-name}}` and `{{description}}` makes the templates generic and ready for substitution by templating tools.
- Changing `{{project-name}}` to `{{project_name}}` inside Rust code examples and doctests ensures that cargo-generate (or mock substitutes) will populate the placeholder using snake_case, allowing doc-tests to compile cleanly under `cargo test`.
- Standardizing the placeholders across all other Rust source files to `{{project_name}}` guarantees consistent behavior when the template is instantiated.
- Modifying `lib.rs` to re-export all domain and typestate types (`Blake3Hash`, `ObjectRef`, `canonical_bytes`, `ProfileId`, `Evidence`, `Admit`, `Raw`, `Validated`, `Admitted`, `RawEvidence`, `ValidatedEvidence`, `AdmittedEvidence`, `AdmittedReceipt`) from `types.rs` resolves doc-test and crate-root import failures.
- Declaring `mcp_server` target explicitly in `Cargo.toml` with `required-features = ["mcp"]` ensures it is only built when `tokio` (pulled by the `mcp` feature) is active, fixing optional dependency failures.
- Adding `"lsp-max/proposed"` to the `lsp` feature list matches the expected traits in the dependencies.

## 3. Caveats
- Long-running builds (such as compiling RocksDB for all features) on macOS `/tmp` directories can trigger the system cleanup daemon, resulting in missing files during compiler linkage (e.g. `ranlib` errors). Setting `CARGO_TARGET_DIR` to a non-temp path resolves this.
- Unit testing was executed on the dynamically generated project under `/tmp/my-conforming-project` and verified to work correctly.

## 4. Conclusion
All identified defects have been remediated: name placeholders have been reverted and standardized to snake_case in source files, typestates and receipts are fully re-exported in the crate root, the optional server dependency is properly mapped, and all features align. The template conforming verification passes.

## 5. Verification Method
To independently verify the changes:
1. Run the conformance script:
   ```bash
   /Users/sac/praxis/tools/verify_conformance.sh
   ```
2. Navigate to the conforming project directory:
   ```bash
   cd /tmp/my-conforming-project
   ```
3. Run the clean build and check:
   ```bash
   cargo clean && cargo check --all-targets --all-features
   ```
4. Run default tests:
   ```bash
   cargo test
   ```
5. Run all-features tests:
   ```bash
   CARGO_TARGET_DIR=/Users/sac/cargo_target_conforming cargo test --all-features
   ```
All compilation checks, tests, and doc-tests should compile cleanly and pass.
