## 2026-06-22T05:30:39Z
You are the Praxis Upgrader (role: worker).
Your working directory is `/Users/sac/rocket-craft/.agents/worker_praxis_upgrade`.
Your parent conversation ID is `30eea61c-a259-48ef-85ca-8bca6c94e767`.

Your mission is to upgrade the `~/praxis` boilerplate generator codebase (located at `/Users/sac/praxis`) to natively support typestate-driven configurations and `RulePackServer` structures adhering to the Post-Chatman Equation A = \mu(O^*).

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Specifically, you must:
1. Modify `/Users/sac/praxis/template/Cargo.toml`:
   - Uncomment or add the `lsp` feature and associate it with appropriate dependencies.
   - Add `lsp-max` as a dependency with a path dependency to `/Users/sac/lsp-max`:
     `lsp-max = { path = "/Users/sac/lsp-max", optional = true }`
   - Add optional dependencies needed for `lsp.rs` when `lsp` feature is enabled:
     - `tree-sitter` (version "0.22", optional = true)
     - `tree-sitter-rust` (version "0.22", optional = true) (or any version matching tree-sitter, or implement dummy/minimal language helper if needed, but tree-sitter-rust is standard)
     - `dashmap` (version "6", optional = true)
     - `parking_lot` (version "0.12", optional = true)
     - `tokio` (version "1", features = ["full"], optional = true)

2. Create `/Users/sac/praxis/template/src/lsp.rs` to implement the `AppLspServer` structure implementing `RulePackServer` from `lsp-max`. This server skeleton must align with the Post-Chatman equation, providing scaffolding for:
   - `RulePackServer` traits and associated method implementations (`rule_packs`, `grammar`, `server_name`, `client`, `adapter`, `workspace_index`, `spc_monitor`, `latency_trackers`, `rule_circuit_breaker`).
   - Use template placeholders where appropriate (e.g. `{{project-name}}`) so `cargo generate` correctly names the server.

3. Verify that the templates are fully syntactically correct:
   - In `/Users/sac/praxis/template/src/lib.rs`, verify that `pub mod lsp;` is present and conditional on `#[cfg(feature = "lsp")]`.

4. Write a programmatic verification script/program at `/Users/sac/praxis/tools/verify_conformance.sh` or a similar path:
   - This script must dynamically generate a temporary project from `/Users/sac/praxis/template` (using `cargo generate` or a custom mock substitution script if `cargo generate` requires interactive input, or use `cargo generate --silent --name my-conforming-project --template /Users/sac/praxis/template`).
   - Run `cargo check --all-targets --all-features` inside the generated project, and assert that it compiles.
   - Parse or scan the generated project code files to programmatically verify structural conformance to Post-Chatman principles (e.g., check for the presence of `Evidence<T, State, W>` typestate structure, `Admit` trait, `AppLspServer`, and `RulePackServer` implementation).

Write a detailed handoff report when you are finished in `/Users/sac/rocket-craft/.agents/worker_praxis_upgrade/handoff.md` and send a message back to the parent conversation ID `30eea61c-a259-48ef-85ca-8bca6c94e767`.
