## 2026-06-21T22:47:56Z
You are the Praxis Upgrader (role: worker).
Your working directory is `/Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation`.
Your parent conversation ID is `30eea61c-a259-48ef-85ca-8bca6c94e767`.

Your mission is to remediate the defects in `/Users/sac/praxis` identified by the reviews.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Please execute the following steps exactly:
1. Revert Hardcoded Project Names to Placeholders:
   - In `/Users/sac/praxis/template/Cargo.toml`, change `name = "praxis_template"` back to `name = "{{project-name}}"`, change `description = "Praxis template upgrade"` back to `description = "{{description}}"`, and revert all repository, homepage, and documentation URLs to use `{{project-name}}`.
   - In `/Users/sac/praxis/template/Cargo.workspace.toml`, revert `name = "praxis_template-core"` to `name = "{{project-name}}-core"` and the repository/homepage URLs to use `{{project-name}}`.
   - In `/Users/sac/praxis/template/src/lsp.rs`, revert `praxis_template` to `{{project-name}}` on line 1, line 37, and line 68.
   - In `/Users/sac/praxis/template/src/main.rs`, revert `praxis_template` to `{{project-name}}` on line 1.
   - In `/Users/sac/praxis/template/src/types.rs`, revert `praxis_template` to `{{project_name}}` in all doctests/code examples (lines 18, 81, 140, 238, 437, 719, 1028, 1042) and to `{{project-name}}` in the module documentation header (line 1).

2. Standardize Crate Name Placeholders in Rust Source Files:
   - In `/Users/sac/praxis/template/src/chain.rs` line 46, change the doctest import `use {{project-name}}::chain::ChainAssembler;` to `use {{project_name}}::chain::ChainAssembler;` (using snake_case so it compiles when substituted).
   - In `/Users/sac/praxis/template/src/error.rs` line 112, change `use {{project-name}}::error::{AppError, ValidationChain};` to `use {{project_name}}::error::{AppError, ValidationChain};`.
   - In `/Users/sac/praxis/template/src/verbs/verifier.rs` line 66, change `use {{project-name}}::verbs::verify::{VerifyGuard, VerifyMetrics};` to `use {{project_name}}::verbs::verify::{VerifyGuard, VerifyMetrics};`.
   - In `/Users/sac/praxis/template/src/cli.rs` line 84, change `use {{crate_name}}::cli::{collect_tools_from_cmd, Cli};` to `use {{project_name}}::cli::{collect_tools_from_cmd, Cli};`.

3. Re-export All Domain and Typestate Types in Crate Root:
   - In `/Users/sac/praxis/template/src/lib.rs`, modify the re-export line (around line 22) to export all the following types from `types.rs`:
     `Blake3Hash`, `ObjectRef`, `canonical_bytes`, `ProfileId`, `Evidence`, `Admit`, `Raw`, `Validated`, `Admitted`, `RawEvidence`, `ValidatedEvidence`, `AdmittedEvidence`, `AdmittedReceipt`.
     This will resolve the doctest import failures for `ProfileId`.

4. Fix Optional Tokio Dependency for MCS Server Binary Target:
   - In `/Users/sac/praxis/template/Cargo.toml`, explicitly declare the `mcp_server` binary target and restrict it to require the `mcp` feature (which pulls in `tokio`):
     ```toml
     [[bin]]
     name = "mcp_server"
     path = "src/bin/mcp_server.rs"
     required-features = ["mcp"]
     ```

5. Map `lsp-max/proposed` Feature Dependency:
   - In `/Users/sac/praxis/template/Cargo.toml`, update the `lsp` feature mapping to activate `lsp-max/proposed` so the trait implementations match:
     `lsp = ["dep:lsp-max", "lsp-max/proposed", "dep:tree-sitter", ...]`

6. Verification:
   - Run `/Users/sac/praxis/tools/verify_conformance.sh`. Make sure it runs and passes.
   - Go to `/tmp/my-conforming-project` and run `cargo clean && cargo check --all-targets --all-features` to ensure it compiles cleanly from scratch.
   - Run `cargo test` to ensure all tests (including default tests) pass.
   - Run `cargo test --all-features` to ensure all tests with features enabled pass.

Write a detailed handoff report when you are finished in `/Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation/handoff.md` and send a message back to the parent conversation ID `30eea61c-a259-48ef-85ca-8bca6c94e767`.
