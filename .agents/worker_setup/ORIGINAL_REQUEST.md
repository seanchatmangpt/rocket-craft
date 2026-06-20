## 2026-06-20T00:29:21Z
Your identity: You are Worker 1 (archetype: worker/teamwork_preview_worker).
Your working directory is /Users/sac/rocket-craft/.agents/worker_setup
Your task: Set up the Cargo workspace configuration and initialize the `ggen-asset-lsp` crate.

Specifically:
1. Register `crates/ggen-asset-lsp` as a member in `/Users/sac/rocket-craft/Cargo.toml`.
2. Create the directory `crates/ggen-asset-lsp/src` if it doesn't exist.
3. Write `crates/ggen-asset-lsp/Cargo.toml` to depend on the local `lsp-max` framework crates. The dependencies should include:
   - `lsp-max = { path = "/Users/sac/lsp-max" }`
   - `lsp-types-max = { path = "/Users/sac/lsp-types-max", version = "26.6.5", features = ["proposed"] }`
   - standard dependencies like `serde`, `serde_json`, `tokio`, `clap`, `anyhow`, etc. (using versions defined in workspace or standard Cargo registry as needed).
4. Write a minimal skeleton in `crates/ggen-asset-lsp/src/main.rs` that imports `lsp-max` and sets up a basic entry point (matching standard lsp-max examples like `examples/powl-lsp` or `examples/anti-llm-cheat-lsp`), which just launches the server over stdio.
5. Run `cargo check` inside the workspace `/Users/sac/rocket-craft` to verify that everything compiles successfully and there are no compiler errors.
6. Write your report to `/Users/sac/rocket-craft/.agents/worker_setup/handoff.md` summarizing the changes, the Cargo check outputs, and send a message back to the orchestrator.
